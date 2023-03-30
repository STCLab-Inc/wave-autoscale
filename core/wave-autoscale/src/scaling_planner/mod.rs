use crate::metric_adapter::MetricStore;
use data_layer::ScalingPlanDefinition;
use serde_json::Value;
use std::{collections::HashMap, time::Duration};
use tokio::{sync::RwLockReadGuard, time};

pub struct ScalingPlanner {
    definition: ScalingPlanDefinition,
    metric_store: MetricStore,
}

// Get a context with the metric store values set as global variables
fn get_context_with_metric_store(
    metric_store: &RwLockReadGuard<HashMap<String, Value>>,
) -> quick_js::Context {
    let context = quick_js::Context::new().unwrap();
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
    context
}

impl<'a> ScalingPlanner {
    pub fn new(definition: ScalingPlanDefinition, metric_store: MetricStore) -> Self {
        ScalingPlanner {
            definition,
            metric_store,
        }
    }
    pub async fn run(&self) {
        let mut plans = self.definition.plans.clone();
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

        tokio::spawn(async move {
            let shared_metric_store = shared_metric_store.read().await;
            loop {
                for plan in plans.iter() {
                    if let Some(expression) = plan["expression_bool"].as_str() {
                        println!("Expression: {}", expression);
                        let context = get_context_with_metric_store(&shared_metric_store);
                        match context.eval(expression) {
                            Ok(value) => {
                                println!("Value: {:?}", value);
                            }
                            Err(error) => {
                                println!("Error: {:?}", error);
                            }
                        }
                    } else {
                        println!("No expression found")
                    }
                }
                // Wait for the next interval.
                interval.tick().await;
            }
        });
    }
}
