use std::{collections::HashMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use data_layer::Metric;
use serde::Deserialize;
use tokio::{sync::Mutex, time};

use super::MetricAdapter;

pub struct PrometheusMetricAdapter {
    metric: Metric,
    last_timestamp: Arc<Mutex<f64>>,
    last_value: Arc<Mutex<f64>>,
}

impl PrometheusMetricAdapter {
    pub const METRIC_KIND: &'static str = "prometheus";
    pub fn new(metric: Metric) -> Self {
        PrometheusMetricAdapter {
            metric: metric,
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
        let mut interval = time::interval(Duration::from_millis(1000));

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
                println!("url_with_query: {}", url_with_query);
                let response = reqwest::get(url_with_query)
                    .await
                    .unwrap()
                    .json::<serde_json::Value>()
                    .await;

                if let Ok(response) = response {
                    let timestamp = &response["data"]["result"][0]["value"][0];
                    let mut shared_timestamp = shared_timestamp.lock().await;
                    *shared_timestamp = timestamp.as_f64().unwrap();

                    let value = &response["data"]["result"][0]["value"][1];
                    let mut shared_value = shared_value.lock().await;
                    *shared_value = value.as_str().unwrap().parse::<f64>().unwrap();
                    println!("b: {:?}", shared_value);
                }

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
