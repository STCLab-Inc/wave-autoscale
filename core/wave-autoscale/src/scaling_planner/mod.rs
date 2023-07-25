pub mod scaling_planner_manager;
use crate::{
    metric_updater::SharedMetricUpdater, scaling_component::SharedScalingComponentManager,
};
use anyhow::Result;
use data_layer::{
    data_layer::DataLayer,
    types::{
        autoscaling_history_definition::AutoscalingHistoryDefinition,
        plan_item_definition::PlanItemDefinition,
    },
    ScalingPlanDefinition,
};
use log::{debug, error};
use rquickjs::async_with;
use serde_json::{json, Value};
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
        let scaling_plan_definition = self.definition.clone();
        let data_layer = self.data_layer.clone();
        let plan_interval = scaling_plan_definition.interval.unwrap_or(1000);
        let plan_interval = if plan_interval < 1000 {
            1000
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
                        let expression = &plan.expression;
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

                        // TODO: Stabilization window(time)
                        // Check if the plan has already been executed
                        let scaling_plan_id = &plan.id;
                        let res = {
                            let shared_last_run_read = shared_last_run.read().await;
                            *shared_last_run_read.clone() == *scaling_plan_id
                        };
                        if res {
                            debug!("Scaling plan {} has already been executed", scaling_plan_id);
                            continue;
                        }

                        // Apply the scaling components
                        let scaling_components_metadata = &plan.scaling_components;
                        let results = apply_scaling_components(
                            scaling_components_metadata,
                            &shared_scaling_component_manager,
                        )
                        .await;

                        debug!("results - {:?}", results);

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

    #[tokio::test]
    async fn test_run() {
        let data_layer = Arc::new(DataLayer::new("", "").await);
        let metric_definitions = vec![MetricDefinition {
            id: "metric1".to_string(),
            metadata: HashMap::new(),
            kind: ObjectKind::Metric,
            db_id: "".to_string(),
            collector: "vector".to_string(),
            metric_kind: "prometheus".to_string(),
        }];
        let _ = data_layer.add_metrics(metric_definitions).await;

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
        let metric = metric.as_str();

        let _ = data_layer
            .add_source_metric("vector", "metric1", metric)
            .await;

        let mut metric_updater = MetricUpdater::new(data_layer.clone(), 1000);
        metric_updater.run().await;

        let shared_metric_updater = Arc::new(RwLock::new(metric_updater));

        let scaling_component_manager = ScalingComponentManager::new_shared();
        let scaling_plan_definition = ScalingPlanDefinition {
            id: "test".to_string(),
            db_id: "".to_string(),
            kind: ObjectKind::ScalingPlan,
            title: "Test Scaling Plan".to_string(),
            interval: None,
            plans: vec![PlanItemDefinition {
                id: "empty".to_string(),
                description: "".to_string(),
                expression:
                    "get({ metric_id: 'metric1', name: 'test', tags: { tag1: 'value1'}}) > 0"
                        .to_string(),
                priority: 1,
                scaling_components: vec![],
                ui: None,
            }],
        };
        let mut scaling_planner = ScalingPlanner::new(
            scaling_plan_definition,
            shared_metric_updater.clone(),
            scaling_component_manager.clone(),
            data_layer.clone(),
        );

        scaling_planner.run();

        tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await;
    }
}
