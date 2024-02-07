pub mod scaling_planner_manager;
use crate::{
    metric_updater::SharedMetricUpdater, scaling_component::SharedScalingComponentManager,
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use data_layer::{
    data_layer::{DataLayer, SOURCE_METRICS_DATA},
    types::{
        autoscaling_history_definition::AutoscalingHistoryDefinition,
        plan_item_definition::PlanItemDefinition, scaling_plan_definition::DEFAULT_PLAN_INTERVAL,
    },
    ScalingPlanDefinition,
};
use rquickjs::async_with;
use serde_json::{json, Value};
use std::ops::Bound::Included;
use std::str::FromStr;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{sync::RwLock, task::JoinHandle, time};
use tracing::{debug, error, info};
use ulid::Ulid;

// Constants
const PLAN_EXPRESSION_PERIOD_SEC: u64 = 5 * 60;

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
 * PlanExpressionStats
 */
#[derive(Debug, Clone)]
enum PlanExpressionStats {
    Latest,
    Average,
    Sum,
    Count,
    Minimum,
    Maximum,
    LinearSlope,
    MovingAverageSlope,
}
impl std::fmt::Display for PlanExpressionStats {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PlanExpressionStats::Latest => write!(f, "latest"),
            PlanExpressionStats::Average => write!(f, "avg"),
            PlanExpressionStats::Sum => write!(f, "sum"),
            PlanExpressionStats::Count => write!(f, "count"),
            PlanExpressionStats::Minimum => write!(f, "min"),
            PlanExpressionStats::Maximum => write!(f, "max"),
            PlanExpressionStats::LinearSlope => write!(f, "linear_slope"),
            PlanExpressionStats::MovingAverageSlope => write!(f, "moving_average_slope"),
        }
    }
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

            // Run the loop every interval
            loop {
                /*
                 * Cool Down Stage
                 */
                if let Some(cool_down) = plan_metadata.get("cool_down") {
                    // apply cool down
                    if let Some(last_plan_timestamp) = *shared_last_plan_timestamp.read().await {
                        let now = Utc::now();
                        if let Some(cool_down_seconds) = cool_down.as_u64() {
                            let cool_down_duration =
                                chrono::Duration::seconds(cool_down_seconds as i64);
                            let time_left = last_plan_timestamp + cool_down_duration - now;
                            if time_left.num_milliseconds() > 0 {
                                debug!(
                                    "[ScalingPlanner] Cooling down. Skip the plan. {} seconds left.",
                                    time_left.num_seconds()
                                );
                                interval.tick().await;
                                continue;
                            }
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
                        let value = value.as_str();
                        if value.is_none() {
                            error!("[ScalingPlanner] Failed to evaluate the variable: {}", key);
                            continue;
                        }
                        let value = value.unwrap();

                        // Evaluate the variable and set the value to the context with "$key"
                        async_with!(context => |ctx| {
                            let expression = format!("var ${} = {};", key, value);
                            let result = ctx.eval::<(), _>(expression);
                            if result.is_err() {
                                error!(
                                    "[ScalingPlanner] Failed to set the variable: {} - {}",
                                    key,
                                    result.err().unwrap()
                                );
                            }
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

fn get_in_js(args: rquickjs::Object<'_>) -> Result<f64, rquickjs::Error> {
    let metric_id = args
        .get::<String, String>("metric_id".to_string())
        .map_err(|_| {
            error!("[ScalingPlan expression error] Failed to get metric_id");
            rquickjs::Error::new_loading("Failed to get metric_id")
        })?;
    let name = args.get::<String, String>("name".to_string()).ok();
    // tags, stats, period_sec is optional
    let tags = match args.get::<String, HashMap<String, String>>("tags".to_string()) {
        Ok(tags) => tags,
        Err(_) => HashMap::new(),
    };
    let stats = args
        .get::<String, String>("stats".to_string())
        .unwrap_or("latest".to_string());
    let period_sec = args
        .get::<String, u64>("period_sec".to_string())
        .unwrap_or(PLAN_EXPRESSION_PERIOD_SEC); // default 5 min

    let Ok(source_metrics_data) = SOURCE_METRICS_DATA.read() else {
        error!("[get_in_js] Failed to get source_metrics_data");
        return Err(rquickjs::Error::new_loading("Failed to get the metrics data"));
    };
    let start_time = Ulid::from_datetime(
        std::time::SystemTime::now() - Duration::from_millis(1000 * period_sec),
    );
    let end_time = Ulid::new();

    debug!(
        "[get_in_js] - metric_id: {}, name: {:?}, tags: {:?}, stats: {}, period_sec: {}",
        metric_id, name, tags, stats, period_sec
    );

    // find metric_id
    let Some(metric_values) = source_metrics_data.source_metrics.get(&metric_id) else {
        return Err(rquickjs::Error::new_loading("Failed to get metric_id from the metrics data"));
    };

    // Filtered metric values
    let mut target_value_arr: Vec<f64> = Vec::new();

    // Validate whether the start_time is before the last item in the metric_values.
    // If the start_time is after the last item, then BTreeMap will panic.
    let last_item = metric_values.iter().last();
    if last_item.is_none() {
        return Err(rquickjs::Error::new_loading(
            "Failed to get the last item in the metric_values",
        ));
    }
    let last_item = last_item.unwrap();
    let last_item_time = Ulid::from_str(last_item.0.as_str()).unwrap();
    if last_item_time < start_time {
        return Err(rquickjs::Error::new_loading(
            "The start_time is before the last item in the metric_values",
        ));
    }

    // Find the metric values between the time range (current time - period_sec, current time)
    metric_values
        .range((Included(start_time.to_string()), Included(end_time.to_string())))
        .for_each(|(_ulid, source_metrics_value)| {
            // Get the json string
            let Ok(value) = serde_json::to_value(source_metrics_value.clone()) else {
                error!(
                    "[ScalingPlan expression error] Failed to convert source_metric_data to serde value"
                );
                return;
            };
            let Some(json_value_str) = value
                .get("json_value")
                .and_then(|value| value.as_str()) else {
                return;
            };
            // Transform the json string to serde value
            let json_value_str = serde_json
                ::from_str::<Value>(json_value_str)
                .map_err(|_| {
                    error!(
                        "[ScalingPlan expression error] Failed to convert json_value to serde value"
                    );
                    rquickjs::Error::Exception
                })
                .unwrap();

            // Get the value array
            let Some(json_values_arr) = json_value_str.as_array() else {
                return;
            };

            for json_value_item in json_values_arr.iter() {
                // Check if the name
                let item_name = json_value_item.get("name").and_then(Value::as_str);
                if
                    name.is_some() &&
                    item_name.is_some() &&
                    item_name.unwrap() != name.as_ref().unwrap()
                {
                    continue;
                }

                // Check if the tags match
                let item_tags = json_value_item.get("tags").and_then(Value::as_object);
                if !tags.is_empty() {
                    // If the tags are not empty but the item_tags is None, then it means that it doesn't match
                    if item_tags.is_none() {
                        continue;
                    }
                    let item_tags = item_tags.unwrap();

                    let mut match_tags = true;
                    for (key, value) in tags.iter() {
                        let item_value = item_tags.get(key).and_then(Value::as_str);
                        if item_value.is_none() || item_value.unwrap() != value {
                            match_tags = false;
                            break;
                        }
                    }

                    // If the tags don't match, then skip
                    if !match_tags {
                        continue;
                    }
                }

                // Put the value in the target_value_arr
                let item_value = json_value_item.get("value").and_then(Value::as_f64);
                if item_value.is_some() {
                    target_value_arr.append(&mut vec![item_value.unwrap()]);
                }
            }
        });
    let metric_stats =
        match stats.to_lowercase() {
            ms if PlanExpressionStats::Latest.to_string() == ms => {
                let Some(latest_value) = target_value_arr.iter().last() else {
                return Err(rquickjs::Error::new_loading("Failed to get the value with the stats"));
            };
                Ok(latest_value.to_owned())
            }
            ms if PlanExpressionStats::Average.to_string() == ms => {
                let sum_value: f64 = target_value_arr.iter().sum();
                let ms_num: f64 = sum_value / (target_value_arr.len() as f64);
                Ok(ms_num)
            }
            ms if PlanExpressionStats::Sum.to_string() == ms => {
                let sum_value: f64 = target_value_arr.iter().sum();
                Ok(sum_value)
            }
            ms if PlanExpressionStats::Count.to_string() == ms => Ok(target_value_arr.len() as f64),
            ms if PlanExpressionStats::Minimum.to_string() == ms => {
                let min_value = target_value_arr.into_iter().reduce(f64::min).ok_or(
                    rquickjs::Error::new_loading("Failed to get the value with the stats"),
                );
                match min_value {
                    Ok(min_value) => Ok(min_value),
                    Err(_) => Err(rquickjs::Error::new_loading(
                        "Failed to get the value with the stats",
                    )),
                }
            }
            ms if PlanExpressionStats::Maximum.to_string() == ms => {
                let max_value = target_value_arr.into_iter().reduce(f64::max).ok_or(
                    rquickjs::Error::new_loading("Failed to get the value with the stats"),
                );
                match max_value {
                    Ok(max_value) => Ok(max_value),
                    Err(_) => Err(rquickjs::Error::new_loading(
                        "Failed to get the value with the stats",
                    )),
                }
            }
            // LinearSlope
            ms if PlanExpressionStats::LinearSlope.to_string() == ms => {
                let mut x: Vec<f64> = Vec::new();
                let mut y: Vec<f64> = Vec::new();
                for (index, value) in target_value_arr.iter().enumerate() {
                    x.append(&mut vec![index as f64]);
                    y.append(&mut vec![value.to_owned()]);
                }
                let x_sum: f64 = x.iter().sum();
                let y_sum: f64 = y.iter().sum();
                let x_y_sum: f64 = x.iter().zip(y.iter()).map(|(x, y)| x * y).sum();
                let x_square_sum: f64 = x.iter().map(|x| x * x).sum();
                let x_sum_square: f64 = x_sum * x_sum;
                let slope: f64 = (x.len() as f64 * x_y_sum - x_sum * y_sum)
                    / (x.len() as f64 * x_square_sum - x_sum_square);
                Ok(slope)
            }
            // MovingAverageSlope
            ms if PlanExpressionStats::MovingAverageSlope.to_string() == ms => {
                // Moving average
                let mut moving_average: Vec<f64> = Vec::new();
                for (index, _) in target_value_arr.iter().enumerate() {
                    let start_index = if index < 5 { 0 } else { index - 5 };
                    let end_index = if index > target_value_arr.len() - 5 {
                        target_value_arr.len()
                    } else {
                        index + 5
                    };
                    let average: f64 = target_value_arr[start_index..end_index].iter().sum();
                    moving_average.append(&mut vec![average / (end_index - start_index) as f64]);
                }
                let mut x: Vec<f64> = Vec::new();
                let mut y: Vec<f64> = Vec::new();
                for (index, value) in moving_average.iter().enumerate() {
                    x.append(&mut vec![index as f64]);
                    y.append(&mut vec![value.to_owned()]);
                }
                let x_sum: f64 = x.iter().sum();
                let y_sum: f64 = y.iter().sum();
                let x_y_sum: f64 = x.iter().zip(y.iter()).map(|(x, y)| x * y).sum();
                let x_square_sum: f64 = x.iter().map(|x| x * x).sum();
                let x_sum_square: f64 = x_sum * x_sum;
                let slope: f64 = (x.len() as f64 * x_y_sum - x_sum * y_sum)
                    / (x.len() as f64 * x_square_sum - x_sum_square);
                Ok(slope)
            }
            _ => {
                error!("[get_in_js] stats is valid: {}", stats);
                Err(rquickjs::Error::new_loading(
                    "Failed to get the value with the stats",
                ))
            }
        };
    debug!("[get_in_js] metric_stats: {:?}", metric_stats);
    metric_stats
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
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    const COLLECTOR: &str = "vector";
    const METRIC_DEFINTION_ID: &str = "metric1";

    async fn get_scaling_planner_with_variables(
        plans: Vec<PlanItemDefinition>,
        variables: HashMap<String, serde_json::Value>,
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
            metadata: HashMap::new(),
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
    ) -> (Arc<DataLayer>, ScalingPlanner) {
        get_scaling_planner_with_variables(plans, HashMap::new()).await
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
        let expression_moving_average_slope = "get({ metric_id: 'metric1', stats: 'moving_average_slope', period_sec: 10, name: 'test', tags: { tag1: 'value1'}}) == -0.5".to_string();

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
        let (data_layer, mut scaling_planner) = get_scaling_planner(
            vec![PlanItemDefinition {
                id: plan_id.clone(),
                description: None,
                expression: Some(
                    "get({ metric_id: 'metric1', stats: 'max', period_sec: 120, name: 'test', tags: { tag1: 'value1'}}) > 0".to_string()
                ),
                cron_expression: None,
                priority: 1,
                scaling_components: vec![],
                ui: None,
            }]
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
    async fn test_simple_expression_with_numeric_variable() {
        let plan_id = uuid::Uuid::new_v4().to_string();
        // Create a ScalingPlanner
        let (data_layer, mut scaling_planner) = get_scaling_planner_with_variables(
            vec![PlanItemDefinition {
                id: plan_id.clone(),
                description: None,
                expression: Some("$test_variable == 1".to_string()),
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
                ),
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
    async fn test_simple_expression_with_boolean_variable() {
        let plan_id = uuid::Uuid::new_v4().to_string();
        // Create a ScalingPlanner
        let (data_layer, mut scaling_planner) = get_scaling_planner_with_variables(
            vec![PlanItemDefinition {
                id: plan_id.clone(),
                description: None,
                expression: Some("$test_variable2".to_string()),
                cron_expression: None,
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
                priority: 1,
                scaling_components: vec![],
                ui: None,
            }]
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
                priority: 1,
                scaling_components: vec![],
                ui: None,
            }]
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
                priority: 1,
                scaling_components: vec![],
                ui: None,
            }]
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
            let last_plan_id = scaling_planner.get_last_plan_item_id();
            let shared_last_plan_id = last_plan_id.read().await;
            assert_eq!(*shared_last_plan_id, "");
        }
    }

    #[tokio::test]
    async fn test_run_action_receiver() {
        let plan_item_id = uuid::Uuid::new_v4().to_string();
        // Create a ScalingPlanner
        let (data_layer, mut scaling_planner) = get_scaling_planner(vec![PlanItemDefinition {
            id: plan_item_id.clone(),
            description: None,
            expression: None,
            cron_expression: None,
            priority: 1,
            scaling_components: vec![],
            ui: None,
        }])
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
}
