use crate::{metric_store::MetricStore, scaling_component::ScalingComponentManager};
use anyhow::Result;
use data_layer::ScalingPlanDefinition;
use serde_json::Value;
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

pub struct ScalingPlanner {
    definition: ScalingPlanDefinition,
    metric_store: MetricStore,
    scaling_component_manager: ScalingComponentManager,
    last_runs: Arc<RwLock<Vec<String>>>,
}

impl<'a> ScalingPlanner {
    pub fn new(
        definition: ScalingPlanDefinition,
        metric_store: MetricStore,
        scaling_component_manager: ScalingComponentManager,
    ) -> Self {
        ScalingPlanner {
            definition,
            metric_store,
            scaling_component_manager,
            last_runs: Arc::new(RwLock::new(Vec::new())),
        }
    }
    pub async fn run(&self) {
        let plans = &self.definition.plans;
        let mut plans = plans.clone();
        // Sort the plans by priority that is higher priority plans are executed first
        plans.sort_by(|a, b| {
            a["priority"]
                .as_u64()
                .unwrap_or(0)
                .cmp(&b["priority"].as_u64().unwrap_or(0))
                .reverse()
        });
        let polling_interval: u64 = 1000;
        let mut interval = time::interval(Duration::from_millis(polling_interval));
        let shared_metric_store = self.metric_store.clone();
        let shared_scaling_component_manager = self.scaling_component_manager.clone();
        let shared_last_runs = self.last_runs.clone();

        tokio::spawn(async move {
            let shared_metric_store = shared_metric_store.read().await;

            loop {
                let mut scaling_components_metadata: &Vec<Value> = &Vec::new();
                let mut plan_id: String = String::new();
                for plan in plans.iter() {
                    if let Some(expression) = plan["expression"].as_str() {
                        println!("Expression: {}", expression);
                        let context = get_context_with_metric_store(&shared_metric_store);
                        match context {
                            Ok(context) => match context.eval_as::<bool>(expression) {
                                Ok(value) => {
                                    println!("Value: {:?}", value);
                                    if value {
                                        scaling_components_metadata =
                                            plan["scaling_components"].as_array().unwrap();
                                        // it is enough to find one plan that matches the expression
                                        plan_id = plan["id"].as_str().unwrap().to_owned();
                                        break;
                                    }
                                }
                                Err(error) => {
                                    println!("Error: {:?}", error);
                                }
                            },
                            Err(error) => {
                                println!("Error: {:?}", error);
                            }
                        }
                    } else {
                        println!("No expression found")
                    }
                }
                println!("Scaling components: {:?}", scaling_components_metadata);
                if !scaling_components_metadata.is_empty() {
                    let shared_last_runs_read = shared_last_runs.read().await;
                    for metadata in scaling_components_metadata.iter() {
                        println!("Metadata: {:?}", metadata);
                        let scaling_component_id = metadata["id"].as_str().unwrap();
                        let cache_key = plan_id.clone() + scaling_component_id;
                        println!("Cache key: {}", cache_key);
                        if shared_last_runs_read.contains(&cache_key) {
                            println!("Already executed");
                            continue;
                        }
                        let shared_scaling_component_manager =
                            shared_scaling_component_manager.read().await;

                        println!("Scaling component id: {}", scaling_component_id);

                        let params = metadata
                            .as_object()
                            .unwrap()
                            .iter()
                            .map(|(key, value)| (key.to_string(), value.clone()))
                            .collect::<HashMap<String, Value>>();

                        shared_scaling_component_manager
                            .apply_to(scaling_component_id, params)
                            .await;

                        let mut last_runs = shared_last_runs.write().await;
                        last_runs.push(cache_key);
                    }
                } else {
                    println!("No scaling components found");
                }
                // Wait for the next interval.
                interval.tick().await;
            }
        });
    }
}
