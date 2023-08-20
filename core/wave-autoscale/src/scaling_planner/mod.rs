pub mod scaling_planner_manager;
use crate::{
    metric_updater::SharedMetricUpdater, scaling_component::SharedScalingComponentManager,
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use data_layer::{
    data_layer::DataLayer,
    types::{
        autoscaling_history_definition::AutoscalingHistoryDefinition,
        plan_item_definition::PlanItemDefinition, scaling_plan_definition::DEFAULT_PLAN_INTERVAL,
    },
    ScalingPlanDefinition,
};
use log::{debug, error};
use rquickjs::async_with;
use serde_json::{json, Value};
use std::str::FromStr;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{sync::RwLock, task::JoinHandle, time};

async fn apply_scaling_components(
    scaling_components_metadata: &[Value],
    shared_scaling_component_manager: &SharedScalingComponentManager,
) -> Vec<Result<()>> {
    let mut scaling_results: Vec<Result<()>> = Vec::new();
    for metadata in scaling_components_metadata.iter() {
        let scaling_component_id = metadata["component_id"].as_str().unwrap();

        let params = metadata
            .as_object()
            .unwrap()
            .iter()
            .map(|(key, value)| (key.to_string(), value.clone()))
            .collect::<HashMap<String, Value>>();

        {
            let shared_scaling_component_manager = shared_scaling_component_manager.read().await;
            let result = shared_scaling_component_manager
                .apply_to(scaling_component_id, params)
                .await;
            scaling_results.push(result);
        }
    }
    scaling_results
}

pub struct ScalingPlanner {
    definition: ScalingPlanDefinition,
    metric_updater: SharedMetricUpdater,
    scaling_component_manager: SharedScalingComponentManager,
    last_plan_id: Arc<RwLock<String>>,
    last_plan_timestamp: Arc<RwLock<Option<DateTime<Utc>>>>,
    data_layer: Arc<DataLayer>,
    task: Option<JoinHandle<()>>,
}

impl<'a> ScalingPlanner {
    pub fn new(
        definition: ScalingPlanDefinition,
        metric_updater: SharedMetricUpdater,
        scaling_component_manager: SharedScalingComponentManager,
        data_layer: Arc<DataLayer>,
    ) -> Self {
        ScalingPlanner {
            definition,
            metric_updater,
            scaling_component_manager,
            last_plan_id: Arc::new(RwLock::new(String::new())),
            last_plan_timestamp: Arc::new(RwLock::new(None)),
            data_layer,
            task: None,
        }
    }
    fn sort_plan_by_priority(&self) -> Vec<PlanItemDefinition> {
        let mut plans = self.definition.plans.clone();
        plans.sort_by(|a, b| a.priority.cmp(&b.priority).reverse());
        plans
    }

    pub fn get_id(&self) -> String {
        self.definition.id.clone()
    }

    pub fn run(&mut self) {
        let shared_metric_updater = self.metric_updater.clone();
        let shared_scaling_component_manager = self.scaling_component_manager.clone();
        let shared_last_run = self.last_plan_id.clone();
        let shared_last_plan_timestamp = self.last_plan_timestamp.clone();
        let scaling_plan_definition = self.definition.clone();
        let data_layer = self.data_layer.clone();

        // metadata
        let plan_metadata = scaling_plan_definition.metadata;

        // For plan_interval
        let plan_interval: u16 = plan_metadata
            .get("interval")
            .unwrap_or(&json!(DEFAULT_PLAN_INTERVAL))
            .as_u64()
            .unwrap_or(DEFAULT_PLAN_INTERVAL as u64) as u16;
        // plan_interval should be at least DEFAULT_PLAN_INTERVAL
        let plan_interval = if plan_interval < DEFAULT_PLAN_INTERVAL {
            DEFAULT_PLAN_INTERVAL
        } else {
            plan_interval
        };
        // If there is a cron_expression in plans then plan_interval should be set to 1 second to check the cron_expression every second
        let plan_interval = if scaling_plan_definition
            .plans
            .iter()
            .any(|plan| plan.cron_expression.is_some())
        {
            DEFAULT_PLAN_INTERVAL
        } else {
            plan_interval
        };

        let plans = self.sort_plan_by_priority();

        let mut interval = time::interval(Duration::from_millis(plan_interval as u64));

        let task = tokio::spawn(async move {
            // Initialize the runtime and context to evaluate the scaling plan expressions
            // TODO: Support Python and other languages
            let Ok(runtime) = rquickjs::AsyncRuntime::new() else {
                error!("Error creating runtime");
                return;
            };
            let Ok(context) = rquickjs::AsyncContext::full(&runtime).await else {
                error!("Error creating context");
                return;
            };

            // Run the loop every interval
            loop {
                if let Some(cool_down) = plan_metadata.get("cool_down") {
                    debug!("Cool down is set to {:?}", cool_down);

                    // apply cool down
                    if let Some(last_plan_timestamp) = *shared_last_plan_timestamp.read().await {
                        let now = Utc::now();

                        if let Some(cool_down_seconds) = cool_down.as_u64() {
                            let cool_down_duration =
                                chrono::Duration::seconds(cool_down_seconds as i64);
                            if now - last_plan_timestamp < cool_down_duration {
                                debug!("Cool down is not over yet");
                                interval.tick().await;
                                continue;
                            }
                        }
                    }
                }
                {
                    // Get the metric values from the MetricUpdater. MetricUpdater fetches and keeps the values every interval.
                    let metric_values: HashMap<String, Value> = {
                        let shared_metric_updater = shared_metric_updater.read().await;
                        match shared_metric_updater.get_metric_values().await {
                            Ok(metric_values_from_updater) => metric_values_from_updater,
                            Err(_) => {
                                error!("Error getting metric values");
                                continue;
                            }
                        }
                    };
                    let metric_values_for_get = metric_values.clone();
                    // Prepare "get" function for JavaScript. This function will be used to get the metric values from the JavaScript code.
                    async_with!(context => |ctx| {
                        let _ = ctx.globals().set(
                            "get",
                            rquickjs::prelude::Func::new("get", move |args: rquickjs::Object| -> Result<f64, rquickjs::Error> {
                                let metric_id = args.get::<String, String>("metric_id".to_string()).map_err(|_| rquickjs::Error::Exception)?;
                                let name = args.get::<String, String>("name".to_string()).map_err(|_| rquickjs::Error::Exception)?;
                                let tags = args.get::<String, HashMap<String, String>>("tags".to_string()).map_err(|_| rquickjs::Error::Exception)?;
                                let json_values = metric_values_for_get.get(&metric_id).ok_or(rquickjs::Error::Exception)?;
                                let json_values = json_values.as_str().ok_or(rquickjs::Error::Exception)?;
                                let json_values: Value = serde_json::from_str(json_values).map_err(|_| rquickjs::Error::Exception)?;
                                let json_values = json_values.as_array().ok_or(rquickjs::Error::Exception)?;
                                let value = json_values.iter().find(|value| {
                                    value.as_object().and_then(|value| value.get("name").and_then(Value::as_str)) == Some(name.as_str())
                                        && (tags.is_empty() || {
                                            value
                                                .get("tags")
                                                .and_then(Value::as_object)
                                                .map_or(false, |value_tags| {
                                                    tags.iter().all(|(key, value)| {
                                                        value_tags.get(key).and_then(Value::as_str) == Some(value.as_str())
                                                    })
                                                })
                                        })
                                }).ok_or(rquickjs::Error::Exception)?;

                                let value = value.as_object().ok_or(rquickjs::Error::Exception)?;
                                let value: f64 = value
                                    .get("value")
                                    .and_then(Value::as_f64)
                                    .ok_or(rquickjs::Error::Exception)?;
                                Ok(value)
                            }),
                        );
                    })
                    .await;

                    let mut excuted = false;
                    // Find the plan that matches the expression
                    for plan in plans.iter() {
                        if plan.cron_expression.is_none() && plan.expression.is_none() {
                            error!("Both cron_expression and expression are empty");
                            continue;
                        }
                        // 1. Cron Expression
                        if let Some(cron_expression) = plan.cron_expression.as_ref() {
                            if !cron_expression.is_empty() {
                                let schedule = cron::Schedule::from_str(cron_expression.as_str());
                                if schedule.is_err() {
                                    error!("Error parsing cron expression: {}", cron_expression);
                                    continue;
                                }
                                let schedule = schedule.unwrap();
                                let now = chrono::Utc::now();
                                let datetime = schedule.upcoming(chrono::Utc).take(1).next();
                                if datetime.is_none() {
                                    error!(
                                        "Error getting next datetime for cron expression: {}",
                                        cron_expression
                                    );
                                    continue;
                                }
                                let datetime = datetime.unwrap();
                                let duration = datetime - now;
                                let duration = duration.num_milliseconds();
                                if duration < 0 || duration > DEFAULT_PLAN_INTERVAL as i64 {
                                    error!(
                                        "The datetime is not yet reached for cron expression: {}",
                                        cron_expression
                                    );
                                    continue;
                                }
                                // It is time to execute the plan. Move on.
                            }
                        }

                        // 2. JS Expression
                        if let Some(expression) = plan.expression.as_ref() {
                            if !expression.is_empty() {
                                // Evaluate the expression
                                let result = async_with!(context => |ctx| {
                                    let Ok(result) = ctx.eval::<bool, _>(expression.clone()) else {
                                        return false;
                                    };
                                    result
                                })
                                .await;

                                // If the expression is false, move to the next plan
                                if !result {
                                    continue;
                                }
                            }
                        }

                        // TODO: Stabilization window(time)
                        // Check if the plan has already been executed
                        let scaling_plan_id = &plan.id;

                        // Apply the scaling components
                        let scaling_components_metadata = &plan.scaling_components;
                        let results = apply_scaling_components(
                            scaling_components_metadata,
                            &shared_scaling_component_manager,
                        )
                        .await;

                        debug!("results - {:?}", results);

                        // update last plan timestamp
                        if !results.is_empty() {
                            let mut shared_last_plan_timestamp =
                                shared_last_plan_timestamp.write().await;
                            *shared_last_plan_timestamp = Some(Utc::now());
                            println!(" >> update last plan timestamp");
                        }

                        // Update the last run
                        {
                            let mut shared_last_run = shared_last_run.write().await;
                            *shared_last_run = scaling_plan_id.clone();
                        }
                        debug!("Applied scaling plan: {}", scaling_plan_id);

                        // Add the result of the scaling plan to the history
                        for (index, result) in results.iter().enumerate() {
                            let fail_message: Option<String> = match result {
                                Ok(_) => None,
                                Err(error) => Some(error.to_string()),
                            };
                            let autoscaling_history: AutoscalingHistoryDefinition =
                                AutoscalingHistoryDefinition::new(
                                    scaling_plan_definition.db_id.clone(),
                                    scaling_plan_definition.id.clone(),
                                    json!(plan).to_string(),
                                    json!(metric_values.clone()).to_string(),
                                    json!(scaling_components_metadata[index].clone()).to_string(),
                                    fail_message,
                                );
                            debug!("autoscaling_history - {:?}", autoscaling_history);
                            let _ = data_layer
                                .add_autoscaling_history(autoscaling_history)
                                .await;
                        }
                        // Stop the loop. We only want to execute one plan per interval.
                        excuted = true;
                        break;
                    }

                    // If no plan was executed
                    if !excuted {
                        debug!("No scaling plan was executed");
                    }
                }
                debug!("------------Next--------------");
                // Wait for the next interval.
                interval.tick().await;
            }
        });
        self.task = Some(task);
    }
    pub fn stop(&mut self) {
        if let Some(task) = &self.task {
            task.abort();
            self.task = None;
        }
    }
    pub fn get_last_plan_id(&self) -> Arc<RwLock<String>> {
        self.last_plan_id.clone()
    }
    pub fn get_last_plan_timestamp(&self) -> Arc<RwLock<Option<DateTime<Utc>>>> {
        self.last_plan_timestamp.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metric_updater::MetricUpdater;
    use crate::scaling_component::ScalingComponentManager;
    use data_layer::data_layer::DataLayer;
    use data_layer::types::object_kind::ObjectKind;
    use data_layer::MetricDefinition;
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    const COLLECTOR: &str = "vector";
    const METRIC_DEFINTION_ID: &str = "metric1";

    async fn get_scaling_planner(
        plans: Vec<PlanItemDefinition>,
    ) -> (Arc<DataLayer>, ScalingPlanner) {
        // Initialize DataLayer
        let data_layer = Arc::new(DataLayer::new("", "").await);
        // Create a MetricDefinition
        let metric_definitions = vec![MetricDefinition {
            id: METRIC_DEFINTION_ID.to_string(),
            metadata: HashMap::new(),
            kind: ObjectKind::Metric,
            db_id: "".to_string(),
            collector: COLLECTOR.to_string(),
            metric_kind: "prometheus".to_string(),
        }];
        let _ = data_layer.add_metrics(metric_definitions).await;

        // To fetch the metrics, we need to start the MetricUpdater
        let mut metric_updater = MetricUpdater::new(data_layer.clone(), 1000);
        metric_updater.run().await;
        let shared_metric_updater = Arc::new(RwLock::new(metric_updater));

        // Create a ScalingComponentManager
        let scaling_component_manager = ScalingComponentManager::new_shared();

        // Create a ScalingPlanner
        let scaling_plan_definition = ScalingPlanDefinition {
            id: "test".to_string(),
            db_id: "".to_string(),
            kind: ObjectKind::ScalingPlan,
            metadata: HashMap::new(),
            plans,
        };

        let scaling_planner = ScalingPlanner::new(
            scaling_plan_definition,
            shared_metric_updater,
            scaling_component_manager,
            data_layer.clone(),
        );
        (data_layer, scaling_planner)
    }

    #[test]
    fn test_utc_calculated() {
        let now = Utc::now();
        let before_2_seconds = now - chrono::Duration::seconds(2);

        assert!(now - before_2_seconds == chrono::Duration::seconds(2));
    }

    #[tokio::test]
    async fn test_last_plan_timestamp() {
        let plan_id = uuid::Uuid::new_v4().to_string();
        // Create a ScalingPlanner
        let (_, mut scaling_planner) = get_scaling_planner(vec![PlanItemDefinition {
            id: plan_id.clone(),
            description: None,
            expression: None,
            cron_expression: Some("*/2 * * * * * *".to_string()),
            priority: 1,
            scaling_components: vec![json!({"component_id": "test_component_id"})],
            ui: None,
        }])
        .await;

        scaling_planner.run();

        // Wait for the scaling planner to execute the plan
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
        {
            scaling_planner.stop();
            let after_last_plan_timestamp = scaling_planner.get_last_plan_timestamp().clone();
            let shared_after_last_plan_timestamp = after_last_plan_timestamp.read().await;
            assert!(shared_after_last_plan_timestamp.is_some());
        }
    }

    #[tokio::test]
    async fn test_simple_expression() {
        let plan_id = uuid::Uuid::new_v4().to_string();
        // Create a ScalingPlanner
        let (data_layer, mut scaling_planner) = get_scaling_planner(vec![PlanItemDefinition {
            id: plan_id.clone(),
            description: None,
            expression: Some(
                "get({ metric_id: 'metric1', name: 'test', tags: { tag1: 'value1'}}) > 0"
                    .to_string(),
            ),
            cron_expression: None,
            priority: 1,
            scaling_components: vec![],
            ui: None,
        }])
        .await;
        scaling_planner.run();

        // Add a metric to the DataLayer
        let metric = json!([
            {
                "name": "test",
                "tags": {
                    "tag1": "value1"
                },
                "value": 1,
            }
        ])
        .to_string();
        let _ = data_layer
            .add_source_metric("vector", "metric1", metric.as_str())
            .await;

        // Wait for the scaling planner to execute the plan
        tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
        {
            let last_plan_id = scaling_planner.get_last_plan_id();
            let shared_last_plan_id = last_plan_id.read().await;
            assert_eq!(*shared_last_plan_id, plan_id);
        }
    }
    #[tokio::test]
    async fn test_cron_expression() {
        let plan_id = uuid::Uuid::new_v4().to_string();
        // Create a ScalingPlanner
        let (_, mut scaling_planner) = get_scaling_planner(vec![PlanItemDefinition {
            id: plan_id.clone(),
            description: None,
            expression: None,
            cron_expression: Some("*/2 * * * * * *".to_string()),
            priority: 1,
            scaling_components: vec![],
            ui: None,
        }])
        .await;
        scaling_planner.run();

        // Wait for the scaling planner to execute the plan
        tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
        {
            let last_plan_id = scaling_planner.get_last_plan_id();
            let shared_last_plan_id = last_plan_id.read().await;
            assert_eq!(*shared_last_plan_id, plan_id);
        }
    }
    #[tokio::test]
    async fn test_complex_expression() {
        let plan_id = uuid::Uuid::new_v4().to_string();
        // Create a ScalingPlanner
        let (data_layer, mut scaling_planner) = get_scaling_planner(vec![PlanItemDefinition {
            id: plan_id.clone(),
            description: None,
            expression: Some(
                "get({ metric_id: 'metric1', name: 'test', tags: { tag1: 'value1'}}) > 0"
                    .to_string(),
            ),
            cron_expression: Some("*/2 * * * * * *".to_string()),
            priority: 1,
            scaling_components: vec![],
            ui: None,
        }])
        .await;
        scaling_planner.run();

        // Add a metric to the DataLayer
        let metric = json!([
            {
                "name": "test",
                "tags": {
                    "tag1": "value1"
                },
                "value": 1,
            }
        ])
        .to_string();
        let _ = data_layer
            .add_source_metric("vector", "metric1", metric.as_str())
            .await;

        // Wait for the scaling planner to execute the plan
        tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
        {
            let last_plan_id = scaling_planner.get_last_plan_id();
            let shared_last_plan_id = last_plan_id.read().await;
            assert_eq!(*shared_last_plan_id, plan_id);
        }
    }
    #[tokio::test]
    async fn test_complex_expression_failed_with_value() {
        let plan_id = uuid::Uuid::new_v4().to_string();
        // Create a ScalingPlanner
        let (data_layer, mut scaling_planner) = get_scaling_planner(vec![PlanItemDefinition {
            id: plan_id.clone(),
            description: None,
            expression: Some(
                "get({ metric_id: 'metric1', name: 'test', tags: { tag1: 'value1'}}) > 0"
                    .to_string(),
            ),
            cron_expression: Some("*/2 * * * * * *".to_string()),
            priority: 1,
            scaling_components: vec![],
            ui: None,
        }])
        .await;
        scaling_planner.run();

        // Add a metric to the DataLayer
        let metric = json!([
            {
                "name": "test",
                "tags": {
                    "tag1": "value1"
                },
                "value": 0,
            }
        ])
        .to_string();
        let _ = data_layer
            .add_source_metric("vector", "metric1", metric.as_str())
            .await;

        // Wait for the scaling planner to execute the plan
        tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
        {
            let last_plan_id = scaling_planner.get_last_plan_id();
            let shared_last_plan_id = last_plan_id.read().await;
            assert_eq!(*shared_last_plan_id, "");
        }
    }
    #[tokio::test]
    async fn test_complex_expression_failed_with_cron() {
        let plan_id = uuid::Uuid::new_v4().to_string();
        // Create a ScalingPlanner
        let (data_layer, mut scaling_planner) = get_scaling_planner(vec![PlanItemDefinition {
            id: plan_id.clone(),
            description: None,
            expression: Some(
                "get({ metric_id: 'metric1', name: 'test', tags: { tag1: 'value1'}}) > 0"
                    .to_string(),
            ),
            cron_expression: Some("* * * * * * 2007".to_string()),
            priority: 1,
            scaling_components: vec![],
            ui: None,
        }])
        .await;
        scaling_planner.run();

        // Add a metric to the DataLayer
        let metric = json!([
            {
                "name": "test",
                "tags": {
                    "tag1": "value1"
                },
                "value": 1,
            }
        ])
        .to_string();
        let _ = data_layer
            .add_source_metric("vector", "metric1", metric.as_str())
            .await;

        // Wait for the scaling planner to execute the plan
        tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
        {
            let last_plan_id = scaling_planner.get_last_plan_id();
            let shared_last_plan_id = last_plan_id.read().await;
            assert_eq!(*shared_last_plan_id, "");
        }
    }
    #[tokio::test]
    async fn test_empty_expression_to_fail() {
        let plan_id = uuid::Uuid::new_v4().to_string();
        // Create a ScalingPlanner
        let (_, mut scaling_planner) = get_scaling_planner(vec![PlanItemDefinition {
            id: plan_id.clone(),
            description: None,
            expression: None,
            cron_expression: None,
            priority: 1,
            scaling_components: vec![],
            ui: None,
        }])
        .await;
        scaling_planner.run();

        // Wait for the scaling planner to execute the plan
        tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
        {
            let last_plan_id = scaling_planner.get_last_plan_id();
            let shared_last_plan_id = last_plan_id.read().await;
            assert_eq!(*shared_last_plan_id, "");
        }
    }
}
