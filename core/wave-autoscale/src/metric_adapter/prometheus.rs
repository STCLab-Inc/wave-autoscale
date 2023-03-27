use super::MetricAdapter;
use async_trait::async_trait;
use data_layer::Metric;
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, time};

// This is a metric adapter for prometheus.
pub struct PrometheusMetricAdapter {
    metric: Metric,
    last_timestamp: Arc<Mutex<f64>>,
    last_value: Arc<Mutex<f64>>,
}

impl PrometheusMetricAdapter {
    // Static variables
    pub const METRIC_KIND: &'static str = "prometheus";

    // Functions
    pub fn new(metric: Metric) -> Self {
        PrometheusMetricAdapter {
            metric,
            last_value: Arc::new(Mutex::new(0.0)),
            last_timestamp: Arc::new(Mutex::new(0.0)),
        }
    }
}

#[async_trait]
impl MetricAdapter for PrometheusMetricAdapter {
    fn get_metric_kind(&self) -> &str {
        PrometheusMetricAdapter::METRIC_KIND
    }
    async fn run(&self) {
        let metadata = self.metric.metadata.clone();
        let polling_interval = metadata
            .get("polling_interval")
            .unwrap()
            .parse::<u64>()
            .unwrap();
        let mut interval = time::interval(Duration::from_millis(polling_interval));

        // Concurrency
        let shared_timestamp = self.last_timestamp.clone();
        let shared_value = self.last_value.clone();
        tokio::spawn(async move {
            loop {
                // Every 1 second, get the metric value from prometheus using reqwest.

                // Generate a url to call a prometheus query.
                let url = metadata.get("endpoint").unwrap();
                let query = metadata.get("query").unwrap();
                let url_with_query = format!("{}/api/v1/query?query={}", url, query);
                let response = reqwest::get(url_with_query)
                    .await
                    .unwrap()
                    .json::<serde_json::Value>()
                    .await;

                // Update the shared value.
                if let Ok(response) = response {
                    // Timestamp
                    let timestamp = &response["data"]["result"][0]["value"][0];
                    let mut shared_timestamp = shared_timestamp.lock().await;
                    *shared_timestamp = timestamp.as_f64().unwrap();
                    // Value
                    let value = &response["data"]["result"][0]["value"][1];
                    let mut shared_value = shared_value.lock().await;
                    *shared_value = value.as_str().unwrap().parse::<f64>().unwrap();
                }
                // Wait for the next interval.
                interval.tick().await;
            }
        });
    }
    async fn get_value(&self) -> f64 {
        let shared_value = self.last_value.clone();
        let shared_value = shared_value.lock().await;
        *shared_value
    }
    async fn get_timestamp(&self) -> f64 {
        let shared_timestamp = self.last_timestamp.clone();
        let shared_timestamp = shared_timestamp.lock().await;
        *shared_timestamp
    }
}
