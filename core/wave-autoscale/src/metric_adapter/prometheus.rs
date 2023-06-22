use super::MetricAdapter;
use crate::metric_store::SharedMetricStore;
use async_trait::async_trait;
use data_layer::MetricDefinition;
use serde_json::Value;
use std::time::Duration;
use tokio::{task::JoinHandle, time};

use log::{debug, error, info};

// This is a metric adapter for prometheus.
pub struct PrometheusMetricAdapter {
    task: Option<JoinHandle<()>>,
    metric: MetricDefinition,
    metric_store: SharedMetricStore,
}

impl PrometheusMetricAdapter {
    pub const METRIC_KIND: &'static str = "prometheus";

    // Functions
    pub fn new(metric: MetricDefinition, metric_store: SharedMetricStore) -> Self {
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
    fn run(&mut self) {
        self.stop();

        let metadata = self.metric.metadata.clone();
        debug!("metric: {:?}", self.metric);
        debug!("metadata: {:?}", metadata);
        // TODO: validate metadata.
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

                    info!("Calling prometheus: {}", url);
                    let response = client.get(url).query(&params).send().await;

                    // Update the shared value.
                    match response {
                        Ok(response) => {
                            let json = response.json::<serde_json::Value>().await;
                            match json {
                                Ok(json) => {
                                    // Update the metric store.
                                    let value = json["data"]["result"].as_array();
                                    if let Some(value) = value {
                                        let value = &value[0]["value"][1];
                                        let shared_metric_store = shared_metric_store.try_write();
                                        if let Ok(mut shared_metric_store) = shared_metric_store {
                                            shared_metric_store
                                                .insert(metric_id.clone(), value.clone());
                                        } else {
                                            error!("Error: failed to lock metric store.");
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Error: {:?}", e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Error: {:?}", e);
                        }
                    }
                    info!("done");
                } else {
                    error!("Error: missing endpoint or query in metadata.");
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
