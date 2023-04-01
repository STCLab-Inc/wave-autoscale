use super::MetricAdapter;
use crate::metric_store::MetricStore;
use async_trait::async_trait;
use data_layer::MetricDefinition;
use serde_json::Value;
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, task::JoinHandle, time};

// This is a metric adapter for prometheus.
pub struct PrometheusMetricAdapter {
    task: Option<JoinHandle<()>>,
    metric: MetricDefinition,
    metric_store: MetricStore,
}

impl PrometheusMetricAdapter {
    pub const METRIC_KIND: &'static str = "prometheus";

    // Functions
    pub fn new(metric: MetricDefinition, metric_store: MetricStore) -> Self {
        PrometheusMetricAdapter {
            task: None,
            metric,
            metric_store,
        }
    }
}

const DEFAULT_POLLING_INTERVAL: u64 = 1000;

#[async_trait]
impl MetricAdapter for PrometheusMetricAdapter {
    fn get_metric_kind(&self) -> &str {
        PrometheusMetricAdapter::METRIC_KIND
    }
    fn get_id(&self) -> &str {
        &self.metric.id
    }
    async fn run(&mut self) {
        self.stop();

        let metadata = self.metric.metadata.clone();

        let polling_interval: u64 = metadata["polling_interval"]
            .as_u64()
            .unwrap_or(DEFAULT_POLLING_INTERVAL);
        let mut interval = time::interval(Duration::from_millis(polling_interval));

        // Concurrency
        let shared_metric_store = self.metric_store.clone();
        let metric_id = self.get_id().to_string();
        let task = tokio::spawn(async move {
            loop {
                // Every 1 second, get the metric value from prometheus using reqwest.
                // Generate a url to call a prometheus query.
                if let (Some(Value::String(url)), Some(Value::String(query))) =
                    (metadata.get("endpoint"), metadata.get("query"))
                {
                    // TODO: validate url and query.
                    let url = format!("{}/api/v1/query", url);
                    let client = reqwest::Client::new();
                    let params = vec![("query", query)];

                    let response = client.get(url).query(&params).send().await;

                    // Update the shared value.
                    match response {
                        Ok(response) => {
                            let json = response.json::<serde_json::Value>().await;
                            match json {
                                Ok(json) => {
                                    // Update the metric store.
                                    let mut shared_metric_store = shared_metric_store.write().await;
                                    let value = json["data"]["result"].as_array();

                                    if let Some(value) = value {
                                        let value = &value[0]["value"][1];
                                        shared_metric_store
                                            .insert(metric_id.clone(), value.clone());
                                    }
                                }
                                Err(e) => {
                                    println!("Error: {:?}", e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("Error: {:?}", e);
                        }
                    }
                } else {
                    println!("Error: missing endpoint or query in metadata.");
                }
                // Wait for the next interval.
                interval.tick().await;
            }
        });
        self.task = Some(task);
    }
    fn stop(&mut self) {
        if let Some(task) = &self.task {
            task.abort();
            self.task = None;
        }
    }
}
