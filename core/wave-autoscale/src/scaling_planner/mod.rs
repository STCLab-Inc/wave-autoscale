use crate::{metric_store::MetricStore, scaling_component::ScalingComponentManager};
use ::data_layer::data_layer::DataLayer;
use anyhow::Result;
use data_layer::{
    types::{
        autoscaling_history_definition::AutoscalingHistoryDefinition,
        plan_item_definition::PlanItemDefinition,
    },
    ScalingPlanDefinition,
};
use serde_json::{json, Value};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{sync::RwLock, time};

// Get a context with the metric store values set as global variables
fn get_context_with_metric_store(
    metric_store: &HashMap<String, Value>,
) -> Result<quick_js::Context> {
    let context = quick_js::Context::new();
    match context {
        Ok(context) => {
            for (key, value) in metric_store.iter() {
                match value {
                    Value::Number(number) => {
                        context.set_global(key, number.as_f64().unwrap()).unwrap();
                    }
                    Value::String(string) => {
                        context.set_global(key, string.as_str()).unwrap();
                    }
                    Value::Array(array) => {
                        let array = array
                            .iter()
                            .map(|value| match value {
                                Value::Number(number) => number.as_f64().unwrap(),
                                _ => 0.0,
                            })
                            .collect::<Vec<_>>();
                        context.set_global(key, array).unwrap();
                    }
                    _ => {}
                }
            }
            Ok(context)
        }
        Err(error) => {
            panic!("Error creating context: {}", error);
        }
    }
}

fn get_matching_scaling_plan<'a>(
    plans: &'a [PlanItemDefinition],
    shared_metric_store: &HashMap<String, Value>,
) -> Option<&'a PlanItemDefinition> {
    for plan in plans.iter() {
        let expression = &plan.expression;
        let context = get_context_with_metric_store(shared_metric_store).unwrap();
        if context.eval_as::<bool>(expression).unwrap_or(false) {
            return Some(plan);
        }
    }
    None
}

async fn apply_scaling_components(
    scaling_components_metadata: &[Value],
    shared_scaling_component_manager: &ScalingComponentManager,
) -> Vec<Result<()>> {
    let mut scaling_results: Vec<Result<()>> = Vec::new();
    for metadata in scaling_components_metadata.iter() {
        let scaling_component_id = metadata["component_id"].as_str().unwrap();
        let shared_scaling_component_manager = shared_scaling_component_manager.read().await;

        let params = metadata
            .as_object()
            .unwrap()
            .iter()
            .map(|(key, value)| (key.to_string(), value.clone()))
            .collect::<HashMap<String, Value>>();

        let result = shared_scaling_component_manager
            .apply_to(scaling_component_id, params)
            .await;
        scaling_results.push(result);
    }
    scaling_results
}

pub struct ScalingPlanner {
    definition: ScalingPlanDefinition,
    metric_store: MetricStore,
    scaling_component_manager: ScalingComponentManager,
    last_plan_id: Arc<RwLock<String>>,
    data_layer: Arc<DataLayer>,
}

impl<'a> ScalingPlanner {
    pub fn new(
        definition: ScalingPlanDefinition,
        metric_store: MetricStore,
        scaling_component_manager: ScalingComponentManager,
        data_layer: Arc<DataLayer>,
    ) -> Self {
        ScalingPlanner {
            definition,
            metric_store,
            scaling_component_manager,
            last_plan_id: Arc::new(RwLock::new(String::new())),
            data_layer,
        }
    }
    fn sort_plan_by_priority(&self) -> Vec<PlanItemDefinition> {
        let mut plans = self.definition.plans.clone();
        plans.sort_by(|a, b| a.priority.cmp(&b.priority).reverse());
        plans
    }
    pub async fn run(&self) {
        let plans = self.sort_plan_by_priority();
        // TODO: Make this configurable
        let polling_interval: u64 = 1000;
        let mut interval = time::interval(Duration::from_millis(polling_interval));
        let shared_metric_store = self.metric_store.clone();
        let shared_scaling_component_manager = self.scaling_component_manager.clone();
        let shared_last_run = self.last_plan_id.clone();
        let scaling_plan_definition = self.definition.clone();
        let data_layer = self.data_layer.clone();

        tokio::spawn(async move {
            loop {
                // Get variables from the metric store
                let shared_metric_store: tokio::sync::RwLockReadGuard<HashMap<String, Value>> =
                    shared_metric_store.read().await;

                // Get the first plan that matches the expression
                if let Some(plan) = get_matching_scaling_plan(&plans, &shared_metric_store) {
                    let scaling_plan_id = plan.id.clone();
                    // Check if the plan has already been executed
                    let shared_last_run_read = shared_last_run.read().await;
                    if *shared_last_run_read.clone() != scaling_plan_id {
                        // Take it back to write to it(RwLock)
                        drop(shared_last_run_read);

                        let scaling_components_metadata = &plan.scaling_components;
                        // Apply the scaling components
                        let results = apply_scaling_components(
                            scaling_components_metadata,
                            &shared_scaling_component_manager,
                        )
                        .await;
                        println!("results - {:?}", results);
                        // Update the last run
                        let mut shared_last_run = shared_last_run.write().await;
                        *shared_last_run = scaling_plan_id.clone();
                        println!("Applied scaling plan: {}", scaling_plan_id);

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
                                    json!(shared_metric_store.clone()).to_string(),
                                    json!(scaling_components_metadata[index].clone()).to_string(),
                                    fail_message,
                                );
                            println!("autoscaling_history - {:?}", autoscaling_history);
                            let result = data_layer
                                .add_autoscaling_history(autoscaling_history)
                                .await;
                            println!("result - {:?}", result);
                        }
                    } else {
                        println!("Already executed");
                    }
                } else {
                    println!("No scaling components found");
                }
                println!("----------------------------------");
                // Wait for the next interval.
                interval.tick().await;
            }
        });
    }
}
