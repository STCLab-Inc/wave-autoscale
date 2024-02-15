pub mod scaling_planner_manager;
mod js_functions;

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
use rquickjs::async_with;
use serde_json::{json, Value};
use std::{collections::HashMap, str::FromStr, sync::Arc, time::Duration};
use tokio::{sync::RwLock, task::JoinHandle, time};
use tracing::{debug, error, info};
use js_functions::get_in_js;


/**
 * ExressionResult
 */

#[derive(Debug, Clone)]
struct ExressionResult {
    result: bool,
    error: bool,
    message: Option<String>,
}

/**
 * Parse action from DataLayer
 */
fn parse_action(action: serde_json::Value) -> Result<(String, String)> {
    let action = action.as_object();
    if action.is_none() {
        return Err(anyhow::anyhow!("Failed to parse action"));
    }
    let action = action.unwrap();

    let plan_id = action.get("plan_id").and_then(serde_json::Value::as_str);
    if plan_id.is_none() {
        return Err(anyhow::anyhow!("Failed to parse plan_id"));
    }
    let plan_id = plan_id.unwrap();

    let plan_item_id = action
        .get("plan_item_id")
        .and_then(serde_json::Value::as_str);
    if plan_item_id.is_none() {
        return Err(anyhow::anyhow!("Failed to parse plan_item_id"));
    }
    let plan_item_id = plan_item_id.unwrap();

    Ok((plan_id.to_string(), plan_item_id.to_string()))
}

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

/**
Create a AutoscalingHistoryDefinition
- plan_db_id
- plan_id
- plan_item_json
- metric_values_json
- metadata_values_json
*/
async fn create_autoscaling_history(
    // plan_db_id, paln
    data_layer: &Arc<DataLayer>,
    plan_db_id: String,
    plan_id: String,
    plan_item: &PlanItemDefinition,
    expression_value_map: Option<&Vec<HashMap<String, Option<f64>>>>,
    scaling_components_metadata: Option<&Value>,
    fail_message: Option<String>,
) {
    let metric_values_json = if let Some(expression_value_map) = expression_value_map {
        json!(expression_value_map).to_string()
    } else {
        "".to_string()
    };
    let metadata_values_json =
        if let Some(scaling_components_metadata) = scaling_components_metadata {
            json!(scaling_components_metadata).to_string()
        } else {
            "".to_string()
        };
    let autoscaling_history: AutoscalingHistoryDefinition = AutoscalingHistoryDefinition::new(
        plan_db_id,
        plan_id,
        json!(plan_item).to_string(),
        metric_values_json,
        metadata_values_json,
        fail_message,
    );
    debug!(
        "[ScalingPlanner] autoscaling_history - {:?}",
        autoscaling_history
    );
    let _ = data_layer
        .add_autoscaling_history(autoscaling_history)
        .await;
}

pub struct ScalingPlanner {
    definition: ScalingPlanDefinition,
    metric_updater: SharedMetricUpdater,
    scaling_component_manager: SharedScalingComponentManager,
    last_plan_item_id: Arc<RwLock<String>>,
    last_plan_timestamp: Arc<RwLock<Option<DateTime<Utc>>>>,
    last_cool_down: Arc<RwLock<u64>>,
    data_layer: Arc<DataLayer>,
    task: Option<JoinHandle<()>>,
    // For instant action
    action_task: Option<JoinHandle<()>>,
    last_plan_item_id_by_action: Arc<RwLock<String>>,
    last_plan_timestamp_by_action: Arc<RwLock<Option<DateTime<Utc>>>>,
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
            last_plan_item_id: Arc::new(RwLock::new(String::new())),
            last_plan_timestamp: Arc::new(RwLock::new(None)),
            last_cool_down: Arc::new(RwLock::new(0)),
            data_layer,
            task: None,
            action_task: None,
            last_plan_item_id_by_action: Arc::new(RwLock::new(String::new())),
            last_plan_timestamp_by_action: Arc::new(RwLock::new(None)),
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
        let _shared_metric_updater = self.metric_updater.clone();
        let shared_scaling_component_manager = self.scaling_component_manager.clone();
        let shared_last_run = self.last_plan_item_id.clone();
        let shared_last_plan_timestamp = self.last_plan_timestamp.clone();
        let shared_last_cool_down = self.last_cool_down.clone();
        let data_layer: Arc<DataLayer> = self.data_layer.clone();

        // PlanDefinition
        let scaling_plan_definition = self.definition.clone();
        let plan_id = scaling_plan_definition.id.clone();
        let plan_db_id = scaling_plan_definition.db_id.clone();
        let plan_metadata = scaling_plan_definition.metadata.clone();
        let plan_variables = scaling_plan_definition.variables.clone();

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

        let plan_items = self.sort_plan_by_priority();

        let mut interval = time::interval(Duration::from_millis(plan_interval as u64));

        let task = tokio::spawn(async move {
            // Initialize the runtime and context to evaluate the scaling plan expressions
            // TODO: Support Python and other languages
            let Ok(runtime) = rquickjs::AsyncRuntime::new() else {
                error!("[ScalingPlanner] Error creating runtime");
                return;
            };
            let Ok(context) = rquickjs::AsyncContext::full(&runtime).await else {
                error!("[ScalingPlanner] Error creating context");
                return;
            };

            // let mut scaling_plan_cool_down: Option<u64> = None;

            // Run the loop every interval
            loop {
                /*
                 * Cool Down Stage
                 */
                {
                    let mut shared_last_cool_down: tokio::sync::RwLockWriteGuard<'_, u64> =
                        shared_last_cool_down.write().await;
                    let cool_down = *shared_last_cool_down;
                    // apply cool down
                    if let Some(last_plan_timestamp) = *shared_last_plan_timestamp.read().await {
                        let now = Utc::now();
                        let cool_down_duration = chrono::Duration::seconds(cool_down as i64);
                        let time_left = last_plan_timestamp + cool_down_duration - now;
                        if time_left.num_milliseconds() > 0 {
                            debug!(
                                "[ScalingPlanner] Cooling down. Skip the plan. {} seconds left.",
                                time_left.num_seconds()
                            );
                            interval.tick().await;
                            continue;
                        } else {
                            *shared_last_cool_down = 0;
                        }
                    }
                }
                {
                    // Prepare the context to evaluate the scaling plan expressions that are written in JavaScript
                    // Set the get function to get the metric values
                    async_with!(context => |ctx| {
                        let _ = ctx.globals().set(
                            "get",
                            rquickjs::prelude::Func::new("get", get_in_js),
                        );
                    })
                    .await;

                    // Evaluate "variables" in the scaling plan
                    for (key, value) in plan_variables.clone().iter() {
                        // If the value is a string, it should be assigned without quotes.
                        let value_for_set = if value.is_string() {
                            let value = value.as_str();
                            if value.is_none() {
                                error!("[ScalingPlanner] Failed to evaluate the variable: {}", key);
                                continue;
                            }
                            value.unwrap().to_string()
                        } else {
                            let value = serde_json::to_string(value);
                            if value.is_err() {
                                error!("[ScalingPlanner] Failed to evaluate the variable: {}", key);
                                continue;
                            }
                            value.unwrap()
                        };

                        // Evaluate the variable and set the value to the context with "$key"
                        async_with!(context => |ctx| {
                            let expression_format = format!("var ${} = {};", key, value_for_set);
                            println!("{}", expression_format);
                            let expression = expression_format;
                            let result = ctx.eval::<(), _>(expression);
                            if result.is_err() {
                                error!(
                                    "[ScalingPlanner] Failed to set the variable: {} - {}",
                                    key,
                                    result.err().unwrap()
                                );
                            }

                            // // Confirm the variable
                            // let result = ctx.eval::<f64, _>(format!("${}", key));
                            // if result.is_err() {
                            //     error!(
                            //         "[ScalingPlanner] Failed to confirm the variable: {}",
                            //         key
                            //     );
                            // }
                            // let result = result.unwrap();
                        })
                        .await;
                    }

                    let mut excuted = false;

                    /*
                     * Find the plan to execute
                     * 1. Cron Expression (if it's not yet reached, skip the plan)
                     * 2. JS Expression (if it's false, skip the plan)
                     * 3. Execute the plan
                     */
                    for plan_item in plan_items.iter() {
                        if plan_item.cron_expression.is_none() && plan_item.expression.is_none() {
                            error!(
                                "[ScalingPlanner] Both cron_expression and expression are empty"
                            );
                            // Skip this plan
                            continue;
                        }
                        /*
                         * 1. Cron Expression
                         */
                        if let Some(cron_expression) = plan_item.cron_expression.as_ref() {
                            if cron_expression.is_empty() {
                                error!("[ScalingPlanner] cron_expression is empty");
                                // Skip this plan
                                continue;
                            }
                            debug!("[ScalingPlanner] cron_expression - {}", cron_expression);
                            let schedule = cron::Schedule::from_str(cron_expression.as_str());
                            if schedule.is_err() {
                                error!(
                                    "[ScalingPlanner] Error parsing cron expression: {}",
                                    cron_expression
                                );
                                // If the expression is invalid, create a AutoscalingHistoryDefinition for the error
                                create_autoscaling_history(
                                    &data_layer.clone(),
                                    plan_db_id.clone(),
                                    plan_id.clone(),
                                    plan_item,
                                    None,
                                    None,
                                    Some("Failed to parse cron expression".to_string()),
                                )
                                .await;
                                // Skip this plan
                                continue;
                            }
                            let schedule = schedule.unwrap();
                            let now = chrono::Utc::now();
                            let datetime = schedule.upcoming(chrono::Utc).take(1).next();
                            if datetime.is_none() {
                                error!("[ScalingPlanner] Error getting next datetime for cron expression: {}", cron_expression);
                                // Skip this plan
                                continue;
                            }
                            let datetime = datetime.unwrap();
                            let duration = datetime - now;
                            let duration = duration.num_milliseconds();
                            if duration < 0 || duration > (DEFAULT_PLAN_INTERVAL as i64) {
                                info!("[ScalingPlanner] The datetime is not yet reached for cron expression: {}", cron_expression);
                                // Skip this plan
                                continue;
                            }
                            // It's confirmed that the cron expression is valid and the datetime is reached
                        }

                        /*
                         * 2. JS Expression
                         */
                        let mut expression_value_map_for_history: Vec<
                            HashMap<String, Option<f64>>,
                        > = Vec::new();

                        if let Some(expression) = plan_item.expression.as_ref() {
                            if expression.is_empty() {
                                error!("[ScalingPlanner] expression is empty");
                                // Skip this plan
                                continue;
                            }
                            debug!("[ScalingPlanner] expression\n{}", expression);
                            // Evaluate the expression.
                            let expression_result =
                                async_with!(context => |ctx| {
                                let result = ctx.eval::<bool, _>(expression.clone());
                                if result.is_err() {
                                    let message = result.err().unwrap().to_string();
                                    error!("[ScalingPlanner] Failed to evaluate expression\n{}\n\n{}", expression, message);
                                    let message = format!("Failed to evaluate expression\n{}", expression);
                                    return ExressionResult {
                                        result: false,
                                        error: true,
                                        message: Some(message),
                                    };
                                }
                                let result = result.unwrap();
                                ExressionResult {
                                    result,
                                    error: false,
                                    message: None,
                                }
                            }).await;

                            debug!(
                                "[ScalingPlanner] expression result - {:?}",
                                expression_result
                            );

                            // expression get value (for history)
                            let expression_map =
                                expression_get_value(expression.clone(), context.clone()).await;
                            expression_value_map_for_history.append(&mut expression_map.clone());

                            // If the expression is false, move to the next plan
                            if !expression_result.result {
                                // If the expression is invalid, create a AutoscalingHistoryDefinition for the error
                                if expression_result.error {
                                    create_autoscaling_history(
                                        &data_layer.clone(),
                                        plan_db_id.clone(),
                                        plan_id.clone(),
                                        plan_item,
                                        None,
                                        None,
                                        expression_result.message,
                                    )
                                    .await;
                                }
                                // Skip this plan
                                continue;
                            }
                        }

                        let results =
                            run_plan_item(plan_item, &shared_scaling_component_manager).await;

                        // update last plan timestamp
                        if !results.is_empty() {
                            let mut shared_last_plan_timestamp =
                                shared_last_plan_timestamp.write().await;
                            *shared_last_plan_timestamp = Some(Utc::now());

                            {
                                let mut shared_last_cool_down = shared_last_cool_down.write().await;
                                // save sub cool down
                                if let Some(sub_cool_down) = plan_item.cool_down {
                                    *shared_last_cool_down = sub_cool_down;
                                // save plan cool down
                                } else if let Some(cool_down) = plan_metadata.get("cool_down") {
                                    let Some(cool_down) = cool_down.as_u64() else {
                                        error!("Failed to get cool_down (none) - {:?}", cool_down);
                                        return;
                                    };
                                    *shared_last_cool_down = cool_down;
                                } else {
                                    *shared_last_cool_down = 0;
                                }
                            }
                        }

                        // Update the last run
                        {
                            let mut shared_last_run = shared_last_run.write().await;
                            let scaling_plan_id = &plan_item.id;
                            *shared_last_run = scaling_plan_id.clone();
                            info!("[ScalingPlanner] Applied scaling plan: {}", scaling_plan_id);
                        }

                        // Add the result of the scaling plan to the history
                        for (index, result) in results.iter().enumerate() {
                            let fail_message: Option<String> = match result {
                                Ok(_) => None,
                                Err(error) => Some(error.to_string()),
                            };
                            let scaling_components_metadata = &plan_item.scaling_components;

                            // Create a AutoscalingHistoryDefinition
                            create_autoscaling_history(
                                &data_layer,
                                scaling_plan_definition.db_id.clone(),
                                scaling_plan_definition.id.clone(),
                                plan_item,
                                Some(&expression_value_map_for_history),
                                Some(&scaling_components_metadata[index]),
                                fail_message,
                            )
                            .await;
                        }
                        // Stop the loop. We only want to execute one plan per interval.
                        excuted = true;
                        break;
                    }

                    // If no plan was executed
                    if !excuted {
                        debug!("[ScalingPlanner] No scaling plan was executed");
                    }
                }
                // Wait for the next interval.
                interval.tick().await;
            }
        });
        self.task = Some(task);

        // Run the action receiver
        self.run_action_receiver();
    }
    fn run_action_receiver(&mut self) {
        let mut receiver = self.data_layer.subscribe_action();
        let definition = self.definition.clone();
        let scaling_component_manager = self.scaling_component_manager.clone();
        let last_plan_id_by_action = self.last_plan_item_id_by_action.clone();
        let last_plan_timestamp_by_action = self.last_plan_timestamp_by_action.clone();
        let action_task = tokio::spawn(async move {
            while let action = receiver.recv().await {
                if action.is_err() {
                    continue;
                }
                let action = action.unwrap();

                let (plan_id, plan_item_id) = match parse_action(action) {
                    Ok((plan_id, plan_item_id)) => (plan_id, plan_item_id),
                    Err(error) => {
                        error!("Failed to parse action: {}", error);
                        continue;
                    }
                };
                if plan_id != definition.id {
                    continue;
                }
                let plan_item = definition.plans.iter().find(|plan| plan.id == plan_item_id);

                if plan_item.is_none() {
                    error!("Failed to find plan_item: {}", plan_item_id);
                    continue;
                }

                let plan_item = plan_item.unwrap();
                let _results = run_plan_item(plan_item, &scaling_component_manager).await;

                // Update the last run
                {
                    let mut shared_last_run = last_plan_id_by_action.write().await;
                    *shared_last_run = plan_item_id.clone();
                    debug!("[ScalingPlanner] Applied scaling plan: {}", plan_item_id);

                    let mut shared_last_plan_timestamp_by_action =
                        last_plan_timestamp_by_action.write().await;
                    *shared_last_plan_timestamp_by_action = Some(Utc::now());
                }
            }
        });
        self.action_task = Some(action_task);
    }
    pub fn stop(&mut self) {
        if let Some(task) = &self.task {
            task.abort();
            self.task = None;
        }
        if let Some(task) = &self.action_task {
            task.abort();
            self.action_task = None;
        }
    }
    // For testing
    #[allow(dead_code)]
    pub fn get_last_plan_item_id(&self) -> Arc<RwLock<String>> {
        self.last_plan_item_id.clone()
    }
    // For testing
    #[allow(dead_code)]
    pub fn get_last_plan_timestamp(&self) -> Arc<RwLock<Option<DateTime<Utc>>>> {
        self.last_plan_timestamp.clone()
    }
    // For testing
    #[allow(dead_code)]
    pub fn get_last_cool_down(&self) -> Arc<RwLock<u64>> {
        self.last_cool_down.clone()
    }
    // For testing
    #[allow(dead_code)]
    pub fn get_last_plan_item_id_by_action(&self) -> Arc<RwLock<String>> {
        self.last_plan_item_id_by_action.clone()
    }
    // For testing
    #[allow(dead_code)]
    pub fn get_last_plan_timestamp_by_action(&self) -> Arc<RwLock<Option<DateTime<Utc>>>> {
        self.last_plan_timestamp_by_action.clone()
    }
}

async fn run_plan_item(
    plan: &PlanItemDefinition,
    shared_scaling_component_manager: &Arc<
        RwLock<crate::scaling_component::ScalingComponentManager>,
    >,
) -> Vec<Result<()>> {
    // Apply the scaling components
    let scaling_components_metadata = &plan.scaling_components;
    let results = apply_scaling_components(
        scaling_components_metadata,
        shared_scaling_component_manager,
    )
    .await;

    debug!("[ScalingPlanner] results - {:?}", results);

    results
}


async fn expression_get_value(
    expression: String,
    context: rquickjs::AsyncContext,
) -> Vec<HashMap<String, Option<f64>>> {
    let re = regex::Regex::new(r"[get\()](.*?)[\)]").unwrap();
    let expression = expression.replace('\n', "");
    let mut expression_value_map: Vec<HashMap<String, Option<f64>>> = Vec::new();
    for cap in re.find_iter(expression.as_str()) {
        let get_result = async_with!(context => |ctx| {
            let get_value =  ctx.eval::<f64, _>(cap.as_str());
            let mut history_map = HashMap::new();
            history_map.insert(cap.as_str().to_string(), match get_value {
                Ok(value) => Some(value),
                Err(_) => None,
            });
            history_map
        })
        .await;
        expression_value_map.append(&mut vec![get_result.clone()]);
    }

    expression_value_map
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
    use tracing_test::traced_test;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    const COLLECTOR: &str = "vector";
    const METRIC_DEFINTION_ID: &str = "metric1";

    async fn get_scaling_planner_with_variables(
        plans: Vec<PlanItemDefinition>,
        variables: HashMap<String, serde_json::Value>,
        plan_metadata: HashMap<String, serde_json::Value>,
    ) -> (Arc<DataLayer>, ScalingPlanner) {
        // Initialize DataLayer
        let data_layer = DataLayer::new("", 500_000, false).await;
        data_layer.sync("").await;
        let data_layer = Arc::new(data_layer);
        // Create a MetricDefinition
        let metric_definitions = vec![MetricDefinition {
            id: METRIC_DEFINTION_ID.to_string(),
            metadata: HashMap::new(),
            kind: ObjectKind::Metric,
            db_id: "".to_string(),
            collector: COLLECTOR.to_string(),
            enabled: true,
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
            variables,
            metadata: plan_metadata,
            plans,
            enabled: true,
        };

        let scaling_planner = ScalingPlanner::new(
            scaling_plan_definition,
            shared_metric_updater,
            scaling_component_manager,
            data_layer.clone(),
        );
        (data_layer, scaling_planner)
    }
    async fn get_scaling_planner(
        plans: Vec<PlanItemDefinition>,
        plan_metadata: HashMap<String, serde_json::Value>,
    ) -> (Arc<DataLayer>, ScalingPlanner) {
        get_scaling_planner_with_variables(plans, HashMap::new(), plan_metadata).await
    }

    #[test]
    fn test_utc_calculated() {
        let now = Utc::now();
        let before_2_seconds = now - chrono::Duration::seconds(2);

        assert!(now - before_2_seconds == chrono::Duration::seconds(2));
    }

    #[tokio::test]
    async fn test_expression_get_value() {
        let Ok(runtime) = rquickjs::AsyncRuntime::new() else {
            error!("Error creating runtime");
            return;
        };
        let Ok(context) = rquickjs::AsyncContext::full(&runtime).await else {
            error!("Error creating context");
            return;
        };
        let expression =
            "get({\n  metric_id: 'cloudwatch_dynamodb_id',\n  name: 'dynamodb_capacity_usage',\n  tags: {\n    tag1: 'value1'\n  },\n  stats: 'max',\n  period_sec: 120\n}) <= 30 || get({\n  metric_id: 'cloudwatch_dynamodb_id',\n  name: 'dynamodb_capacity_usage',\n  tags: {\n    tag1: 'value1'\n  },\n  stats: 'min',\n  period_sec: 120\n}) <= 40\n";
        assert_eq!(
            expression_get_value(expression.to_string(), context)
                .await
                .len(),
            2
        );
    }

    #[tokio::test]
    async fn test_get_in_js() {
        // Initialize DataLayer
        let data_layer = DataLayer::new("", 500_000, false).await;
        data_layer.sync("").await;
        let data_layer = Arc::new(data_layer);

        // Initialize JS Engine (QuickJS)
        let Ok(runtime) = rquickjs::AsyncRuntime::new() else {
            panic!("Error creating runtime");
        };
        let Ok(context) = rquickjs::AsyncContext::full(&runtime).await else {
            panic!("Error creating context");
        };

        async_with!(context => |ctx| {
            let _ = ctx.globals().set(
                "get",
                rquickjs::prelude::Func::new("get", get_in_js),
            );
        })
        .await;

        // Create a MetricDefinition
        let timeover_json_value = json!([{"name": "test", "tags": {"tag1": "value1"}, "value": 5.0}
                                    ,{"name": "test", "tags": {"tag1": "value1"}, "value": 6.0}])
        .to_string(); // metric1

        let json_value =
            json!([{"name": "test", "tags": {"tag1": "value222222"}, "value": 1.0}
                                        ,{"name": "test", "tags": {"tag1": "value1"}, "value": 2.0}]).to_string(); // metric1
        let json_value2 =
            json!([{"name": "test", "tags": {"tag1": "value1"}, "value": 3.0}
                                        ,{"name": "test", "tags": {"tag1": "value1"}, "value": 4.0}]).to_string(); // metric1
        let json_value3 =
            json!([{"name": "test", "tags": {"tag1": "value1"}, "value": 7.0}
                                        ,{"name": "test", "tags": {"tag1": "value1"}, "value": 8.0}]).to_string(); // metric2
        let json_value4 =
            json!([{"name": "test2", "tags": {"tag1": "value1"}, "value": 7.0}]).to_string(); // metric1

        // add data to data_layer
        let _ = data_layer
            .add_source_metrics_in_data_layer("vector", "metric1", &timeover_json_value)
            .await;
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let _ = data_layer
            .add_source_metrics_in_data_layer("vector", "metric1", &json_value)
            .await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        let _ = data_layer
            .add_source_metrics_in_data_layer("vector", "metric1", &json_value2)
            .await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        let _ = data_layer
            .add_source_metrics_in_data_layer("vector", "metric2", &json_value3)
            .await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        let _ = data_layer
            .add_source_metrics_in_data_layer("vector", "metric1", &json_value4)
            .await;

        let expression_avg =
            "get({ metric_id: 'metric1', stats: 'avg', period_sec: 1, name: 'test', tags: { tag1: 'value1'}}) == 3".to_string();
        let expression_sum =
            "get({ metric_id: 'metric1', stats: 'sum', period_sec: 1, name: 'test', tags: { tag1: 'value1'}}) == 9".to_string();
        let expression_count =
            "get({ metric_id: 'metric1', stats: 'count', period_sec: 1, name: 'test', tags: { tag1: 'value1'}}) == 3".to_string();
        let expression_min =
            "get({ metric_id: 'metric1', stats: 'min', period_sec: 1, name: 'test', tags: { tag1: 'value1'}}) == 2".to_string();
        let expression_max =
            "get({ metric_id: 'metric1', stats: 'max', period_sec: 1, name: 'test', tags: { tag1: 'value1'}}) == 4".to_string();
        let expression_sum_timeover =
            "get({ metric_id: 'metric1', stats: 'sum', period_sec: 3, name: 'test', tags: { tag1: 'value1'}}) == 20".to_string();
        let expression_optional_tags =
            "get({ metric_id: 'metric1', stats: 'sum', period_sec: 3, name: 'test'}) == 21"
                .to_string();
        let expression_optional_stats =
            "get({ metric_id: 'metric1', period_sec: 3, name: 'test'}) == 4".to_string();
        let expression_optional_period_sec =
            "get({ metric_id: 'metric1', stats: 'max', name: 'test'}) == 6".to_string();
        let expression_optional_stats_period_sec =
            "get({ metric_id: 'metric1', name: 'test'}) == 4".to_string();
        let _expression_fail_name =
            "get({ metric_id: 'metric1', stats: 'avg', period_sec: 1, tags: { tag1: 'value1'}}) == 3".to_string();
        let expression_fail_metric_id =
            "get({ stats: 'avg', period_sec: 1, name: 'test', tags: { tag1: 'value1'}}) == 3"
                .to_string();

        let expression_linear_slope = "get({ metric_id: 'metric1', stats: 'linear_slope', period_sec: 10, name: 'test', tags: { tag1: 'value1'}}) == -0.5".to_string();

        let expression_moving_average_slope = "get({ metric_id: 'metric1', stats: 'moving_average_slope', period_sec: 10, name: 'test', tags: { tag1: 'value1'}}) < -0.6".to_string();

        let result_avg = check_expression(expression_avg, context.clone()).await;
        let result_sum = check_expression(expression_sum, context.clone()).await;
        let result_count = check_expression(expression_count, context.clone()).await;
        let result_min = check_expression(expression_min, context.clone()).await;
        let result_max = check_expression(expression_max, context.clone()).await;
        let result_sum_timeover = check_expression(expression_sum_timeover, context.clone()).await;
        let result_optional_tags =
            check_expression(expression_optional_tags, context.clone()).await;
        let result_optional_stats =
            check_expression(expression_optional_stats, context.clone()).await;
        let result_optional_period_sec =
            check_expression(expression_optional_period_sec, context.clone()).await;
        let result_optional_stats_period_sec =
            check_expression(expression_optional_stats_period_sec, context.clone()).await;
        let result_fail_metric_id =
            check_expression(expression_fail_metric_id, context.clone()).await;

        let result_linear_slope = check_expression(expression_linear_slope, context.clone()).await;
        let result_moving_average_slope =
            check_expression(expression_moving_average_slope, context.clone()).await;

        match result_avg {
            Ok(result) => assert!(result),
            Err(error) => panic!("Failed to get result_avg: {:?}", error),
        }

        match result_sum {
            Ok(result) => assert!(result),
            Err(error) => panic!("Failed to get result_sum: {:?}", error),
        }

        match result_count {
            Ok(result) => assert!(result),
            Err(error) => panic!("Failed to get result_count: {:?}", error),
        }

        match result_min {
            Ok(result) => assert!(result),
            Err(error) => panic!("Failed to get result_min: {:?}", error),
        }

        match result_max {
            Ok(result) => assert!(result),
            Err(error) => panic!("Failed to get result_max: {:?}", error),
        }

        match result_sum_timeover {
            Ok(result) => assert!(result),
            Err(error) => panic!("Failed to get result_sum_timeover: {:?}", error),
        }

        match result_optional_tags {
            Ok(result) => assert!(result),
            Err(error) => panic!("Failed to get result_optional_tags: {:?}", error),
        }

        match result_optional_stats {
            Ok(result) => assert!(result),
            Err(error) => panic!("Failed to get result_optional_stats: {:?}", error),
        }

        match result_optional_period_sec {
            Ok(result) => assert!(result),
            Err(error) => panic!("Failed to get result_optional_period_sec: {:?}", error),
        }

        match result_optional_stats_period_sec {
            Ok(result) => assert!(result),
            Err(error) => panic!(
                "Failed to get result_optional_stats_period_sec: {:?}",
                error
            ),
        }

        match result_fail_metric_id {
            Ok(_) => panic!(
                "Failed to get result_fail_metric_id: {:?}",
                result_fail_metric_id
            ),
            Err(_) => assert!(result_fail_metric_id.is_err()),
        }

        match result_linear_slope {
            Ok(result) => assert!(result),
            Err(error) => panic!("Failed to get result_linear_slope: {:?}", error),
        }

        match result_moving_average_slope {
            Ok(result) => assert!(result),
            Err(error) => panic!("Failed to get result_moving_average_slope: {:?}", error),
        }
    }

    async fn check_expression(expression: String, context: rquickjs::AsyncContext) -> Result<bool> {
        async_with!(context => |ctx| {
            let Ok(result) = ctx.eval::<bool, _>(expression) else {
                return Err(anyhow::anyhow!("Failed to eval expression"));
            };
            Ok(result)
        })
        .await
    }

    #[tokio::test]
    async fn test_last_plan_timestamp() {
        let plan_id = uuid::Uuid::new_v4().to_string();
        // Create a ScalingPlanner
        let (_, mut scaling_planner) = get_scaling_planner(
            vec![PlanItemDefinition {
                id: plan_id.clone(),
                description: None,
                expression: None,
                cron_expression: Some("*/2 * * * * * *".to_string()),
                cool_down: None,
                priority: 1,
                scaling_components: vec![json!({"component_id": "test_component_id"})],
                ui: None,
            }],
            HashMap::new(),
        )
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
        let (data_layer, mut scaling_planner) = get_scaling_planner(
            vec![PlanItemDefinition {
                id: plan_id.clone(),
                description: None,
                expression: Some(
                    "get({ metric_id: 'metric1', stats: 'max', period_sec: 120, name: 'test', tags: { tag1: 'value1'}}) > 0".to_string()
                ),
                cron_expression: None,
                cool_down: None,
                priority: 1,
                scaling_components: vec![],
                ui: None,
            }],
            HashMap::new(),
        ).await;
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
            .add_source_metrics_in_data_layer("vector", "metric1", metric.as_str())
            .await;

        // Wait for the scaling planner to execute the plan
        tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
        {
            let last_plan_id = scaling_planner.get_last_plan_item_id();
            let shared_last_plan_id = last_plan_id.read().await;
            assert_eq!(*shared_last_plan_id, plan_id);
        }
    }
    #[tokio::test]
    #[traced_test]
    async fn test_simple_expression_with_multiple_variable() {
        let plan_id = uuid::Uuid::new_v4().to_string();
        // Create a ScalingPlanner
        let (data_layer, mut scaling_planner) = get_scaling_planner_with_variables(
            vec![PlanItemDefinition {
                id: plan_id.clone(),
                description: None,
                expression: Some("$test_variable / $number_value == 2 && $boolean_value && $string_value == \"string\" ".to_string()),
                cron_expression: None,
                cool_down: None,
                priority: 1,
                scaling_components: vec![],
                ui: None,
            }],
            [
                // Define a numeric variable
                (
                    "test_variable".to_string(),
                    json!(
                        "get({ metric_id: 'metric1', stats: 'max', period_sec: 120, name: 'test', tags: { tag1: 'value1'}})"
                    ),
                    
                ),
                (
                    "number_value".to_string(),
                    json!(
                        5
                    ),
                ),
                (
                    "boolean_value".to_string(),
                    json!(
                        true
                    ),
                ),
                (
                    "string_value".to_string(),
                    json!(
                        "\"string\""
                    ),
                ),
            ]
                .iter()
                .cloned()
                .collect(),
            HashMap::new(),
        ).await;
        scaling_planner.run();

        // Add a metric to the DataLayer
        let metric = json!([
            {
                "name": "test",
                "tags": {
                    "tag1": "value1"
                },
                "value": 10,
            }
        ])
        .to_string();
        let _ = data_layer
            .add_source_metrics_in_data_layer("vector", "metric1", metric.as_str())
            .await;

        // Wait for the scaling planner to execute the plan
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        {
            let last_plan_id = scaling_planner.get_last_plan_item_id();
            let shared_last_plan_id = last_plan_id.read().await;
            assert_eq!(*shared_last_plan_id, plan_id);
        }
    }
    #[tokio::test]
    #[traced_test]
    async fn test_simple_expression_with_numeric_variable() {
        let plan_id = uuid::Uuid::new_v4().to_string();
        // Create a ScalingPlanner
        let (data_layer, mut scaling_planner) = get_scaling_planner_with_variables(
            vec![PlanItemDefinition {
                id: plan_id.clone(),
                description: None,
                expression: Some("$test_variable == 10".to_string()),
                cron_expression: None,
                priority: 1,
                scaling_components: vec![],
                ui: None,
            }],
            [
                // Define a numeric variable
                (
                    "test_variable".to_string(),
                    json!(
                        "get({ metric_id: 'metric1', stats: 'max', period_sec: 120, name: 'test', tags: { tag1: 'value1'}})"
                    ),
                    
                )
            ]
                .iter()
                .cloned()
                .collect()
        ).await;
        scaling_planner.run();

        // Add a metric to the DataLayer
        let metric = json!([
            {
                "name": "test",
                "tags": {
                    "tag1": "value1"
                },
                "value": 10,
            }
        ])
        .to_string();
        let _ = data_layer
            .add_source_metrics_in_data_layer("vector", "metric1", metric.as_str())
            .await;

        // Wait for the scaling planner to execute the plan
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        {
            let last_plan_id = scaling_planner.get_last_plan_item_id();
            let shared_last_plan_id = last_plan_id.read().await;
            assert_eq!(*shared_last_plan_id, plan_id);
        }
    }
    #[tokio::test]
    async fn test_simple_expression_with_boolean_variable() {
        let plan_id = uuid::Uuid::new_v4().to_string();
        // Create a ScalingPlanner
        let (data_layer, mut scaling_planner) = get_scaling_planner_with_variables(
            vec![PlanItemDefinition {
                id: plan_id.clone(),
                description: None,
                expression: Some("$test_variable2".to_string()),
                cron_expression: None,
                cool_down: None,
                priority: 1,
                scaling_components: vec![],
                ui: None,
            }],
            [
                // Define a boolean variable
                (
                    "test_variable2".to_string(),
                    json!(
                        "get({ metric_id: 'metric1', stats: 'max', period_sec: 120, name: 'test', tags: { tag1: 'value1'}}) > 0"
                    ),
                ),
            ]
                .iter()
                .cloned()
                .collect(),
            HashMap::new(),
        ).await;
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
            .add_source_metrics_in_data_layer("vector", "metric1", metric.as_str())
            .await;

        // Wait for the scaling planner to execute the plan
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        {
            let last_plan_id = scaling_planner.get_last_plan_item_id();
            let shared_last_plan_id = last_plan_id.read().await;
            assert_eq!(*shared_last_plan_id, plan_id);
        }
    }
    #[tokio::test]
    async fn test_simple_expression_with_string_variable() {
        let plan_id = uuid::Uuid::new_v4().to_string();
        // Create a ScalingPlanner
        let (data_layer, mut scaling_planner) = get_scaling_planner_with_variables(
            vec![PlanItemDefinition {
                id: plan_id.clone(),
                description: None,
                expression: Some("$test_variable3 == 'string variable'".to_string()),
                cron_expression: None,
                cool_down: None,
                priority: 1,
                scaling_components: vec![],
                ui: None,
            }],
            [
                // Define a string variable
                ("test_variable3".to_string(), json!("'string variable'")),
            ]
            .iter()
            .cloned()
            .collect(),
            HashMap::new(),
        )
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
            .add_source_metrics_in_data_layer("vector", "metric1", metric.as_str())
            .await;

        // Wait for the scaling planner to execute the plan
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        {
            let last_plan_id = scaling_planner.get_last_plan_item_id();
            let shared_last_plan_id = last_plan_id.read().await;
            assert_eq!(*shared_last_plan_id, plan_id);
        }
    }
    #[tokio::test]
    async fn test_cron_expression() {
        let plan_id = uuid::Uuid::new_v4().to_string();
        // Create a ScalingPlanner
        let (_, mut scaling_planner) = get_scaling_planner(
            vec![PlanItemDefinition {
                id: plan_id.clone(),
                description: None,
                expression: None,
                cron_expression: Some("*/2 * * * * * *".to_string()),
                cool_down: None,
                priority: 1,
                scaling_components: vec![],
                ui: None,
            }],
            HashMap::new(),
        )
        .await;
        scaling_planner.run();

        // Wait for the scaling planner to execute the plan
        tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
        {
            let last_plan_id = scaling_planner.get_last_plan_item_id();
            let shared_last_plan_id = last_plan_id.read().await;
            assert_eq!(*shared_last_plan_id, plan_id);
        }
    }
    #[tokio::test]
    async fn test_complex_expression() {
        let plan_id = uuid::Uuid::new_v4().to_string();
        // Create a ScalingPlanner
        let (data_layer, mut scaling_planner) = get_scaling_planner(
            vec![PlanItemDefinition {
                id: plan_id.clone(),
                description: None,
                expression: Some(
                    "get({ metric_id: 'metric1', stats: 'max', period_sec: 120, name: 'test', tags: { tag1: 'value1'}}) > 0".to_string()
                ),
                cron_expression: Some("*/2 * * * * * *".to_string()),
                cool_down: None,
                priority: 1,
                scaling_components: vec![],
                ui: None,
            }],
            HashMap::new(),
        ).await;
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
            .add_source_metrics_in_data_layer("vector", "metric1", metric.as_str())
            .await;

        // Wait for the scaling planner to execute the plan
        tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
        {
            let last_plan_id = scaling_planner.get_last_plan_item_id();
            let shared_last_plan_id = last_plan_id.read().await;
            assert_eq!(*shared_last_plan_id, plan_id);
        }
    }
    #[tokio::test]
    async fn test_complex_expression_failed_with_value() {
        let plan_id = uuid::Uuid::new_v4().to_string();
        // Create a ScalingPlanner
        let (data_layer, mut scaling_planner) = get_scaling_planner(
            vec![PlanItemDefinition {
                id: plan_id.clone(),
                description: None,
                expression: Some(
                    "get({ metric_id: 'metric1', stats: 'avg', period_sec: 0, name: 'test', tags: { tag1: 'value1'}}) > 0".to_string()
                ),
                cron_expression: Some("*/2 * * * * * *".to_string()),
                cool_down: None,
                priority: 1,
                scaling_components: vec![],
                ui: None,
            }],
            HashMap::new(),
        ).await;
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
            .add_source_metrics_in_data_layer("vector", "metric1", metric.as_str())
            .await;

        // Wait for the scaling planner to execute the plan
        tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
        {
            let last_plan_id = scaling_planner.get_last_plan_item_id();
            let shared_last_plan_id = last_plan_id.read().await;
            assert_eq!(*shared_last_plan_id, "");
        }
    }
    #[tokio::test]
    async fn test_complex_expression_failed_with_cron() {
        let plan_id = uuid::Uuid::new_v4().to_string();
        // Create a ScalingPlanner
        let (data_layer, mut scaling_planner) = get_scaling_planner(
            vec![PlanItemDefinition {
                id: plan_id.clone(),
                description: None,
                expression: Some(
                    "get({ metric_id: 'metric1', stats: 'avg', period_sec: 60, name: 'test', tags: { tag1: 'value1'}}) > 0".to_string()
                ),
                cron_expression: Some("* * * * * * 2007".to_string()),
                cool_down: None,
                priority: 1,
                scaling_components: vec![],
                ui: None,
            }],
            HashMap::new(),
        ).await;
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
            .add_source_metrics_in_data_layer("vector", "metric1", metric.as_str())
            .await;

        // Wait for the scaling planner to execute the plan
        tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
        {
            let last_plan_id = scaling_planner.get_last_plan_item_id();
            let shared_last_plan_id = last_plan_id.read().await;
            assert_eq!(*shared_last_plan_id, "");
        }
    }
    #[tokio::test]
    async fn test_empty_expression_to_fail() {
        let plan_id = uuid::Uuid::new_v4().to_string();
        // Create a ScalingPlanner
        let (_, mut scaling_planner) = get_scaling_planner(
            vec![PlanItemDefinition {
                id: plan_id.clone(),
                description: None,
                expression: None,
                cron_expression: None,
                cool_down: None,
                priority: 1,
                scaling_components: vec![],
                ui: None,
            }],
            HashMap::new(),
        )
        .await;
        scaling_planner.run();

        // Wait for the scaling planner to execute the plan
        tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
        {
            let last_plan_id = scaling_planner.get_last_plan_item_id();
            let shared_last_plan_id = last_plan_id.read().await;
            assert_eq!(*shared_last_plan_id, "");
        }
    }

    #[tokio::test]
    async fn test_run_action_receiver() {
        let plan_item_id = uuid::Uuid::new_v4().to_string();
        // Create a ScalingPlanner
        let (data_layer, mut scaling_planner) = get_scaling_planner(
            vec![PlanItemDefinition {
                id: plan_item_id.clone(),
                description: None,
                expression: None,
                cron_expression: None,
                cool_down: None,
                priority: 1,
                scaling_components: vec![],
                ui: None,
            }],
            HashMap::new(),
        )
        .await;
        scaling_planner.run();

        let plan_id = scaling_planner.definition.id.clone();
        let _ = data_layer.send_plan_action(plan_id, plan_item_id.clone());

        // Wait for the scaling planner to execute the plan
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        {
            let last_plan_item_id = scaling_planner.get_last_plan_item_id_by_action();
            let shared_last_plan_item_id = last_plan_item_id.read().await;
            assert_eq!(*shared_last_plan_item_id, plan_item_id);
        }
    }

    #[tokio::test]
    async fn test_sub_cool_down() {
        // plan metadata cool_down is 1
        let plan_metadata = [
            ("cool_down".to_string(), json!(1)),
            ("interval".to_string(), json!(1000)),
        ]
        .into_iter()
        .collect();
        // Create a ScalingPlanner
        let (_, mut scaling_planner) = get_scaling_planner(
            vec![PlanItemDefinition {
                id: "plan_1".to_string(),
                description: None,
                expression: Some("2>1".to_string()),
                cron_expression: None,
                cool_down: Some(2),
                priority: 1,
                scaling_components: vec![json!({"component_id": "test_component_id"})],
                ui: None,
            }],
            plan_metadata,
        )
        .await;
        scaling_planner.run();

        // sec 0: [cool_down: 0]
        // sec 1: plan_1 is executed -> [sub cool_down 2]

        let sec_0_cool_down = *scaling_planner.get_last_cool_down().read().await;
        assert_eq!(sec_0_cool_down, 0);
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let sec_1_cool_down = *scaling_planner.get_last_cool_down().read().await;
        assert_eq!(sec_1_cool_down, 2);

        scaling_planner.stop();
    }

    #[tokio::test]
    async fn test_default_cool_down() {
        // plan metadata cool_down is 1
        let plan_metadata = [
            ("cool_down".to_string(), json!(1)),
            ("interval".to_string(), json!(1000)),
        ]
        .into_iter()
        .collect();
        // Create a ScalingPlanner
        let (_, mut scaling_planner) = get_scaling_planner(
            vec![PlanItemDefinition {
                id: "plan_1".to_string(),
                description: None,
                expression: Some("2>1".to_string()),
                cron_expression: None,
                cool_down: None,
                priority: 1,
                scaling_components: vec![json!({"component_id": "test_component_id"})],
                ui: None,
            }],
            plan_metadata,
        )
        .await;
        scaling_planner.run();

        // sec 0: [cool_down: 0]
        // sec 1: plan_1 is executed -> [default cool_down 1]

        let sec_0_cool_down = *scaling_planner.get_last_cool_down().read().await;
        assert_eq!(sec_0_cool_down, 0);
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let sec_1_cool_down = *scaling_planner.get_last_cool_down().read().await;
        assert_eq!(sec_1_cool_down, 1);

        scaling_planner.stop();
    }
}
