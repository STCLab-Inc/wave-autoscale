use super::MetricAdapter;
use async_trait::async_trait;
use data_layer::Metric;
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, time};

// This is a metric adapter for prometheus.
pub struct PrometheusMetricAdapter {
    metric: Metric,
    last_result: Arc<Mutex<serde_json::Value>>,
    // last_timestamp: Arc<Mutex<f64>>,
    // last_value: Arc<Mutex<f64>>,
}

impl PrometheusMetricAdapter {
    // Static variables
    pub const METRIC_KIND: &'static str = "prometheus";

    // Functions
    pub fn new(metric: Metric) -> Self {
        PrometheusMetricAdapter {
            metric,
            last_result: Arc::new(Mutex::new(serde_json::Value::Null)),
            // last_value: Arc::new(Mutex::new(0.0)),
            // last_timestamp: Arc::new(Mutex::new(0.0)),
        }
    }
}

#[async_trait]
impl MetricAdapter for PrometheusMetricAdapter {
    fn get_metric_kind(&self) -> &str {
        PrometheusMetricAdapter::METRIC_KIND
    }
    fn get_id(&self) -> &str {
        &self.metric.id
    }
    async fn run(&self) {
        let metadata = self.metric.metadata.clone();

        let mut polling_interval: u64 = 1000;
        if let Some(metadata_polling_interval) = metadata["polling_interval"].as_u64() {
            polling_interval = metadata_polling_interval;
        }
        println!("Polling interval: {:?}", polling_interval);
        let mut interval = time::interval(Duration::from_millis(polling_interval));

        // Concurrency
        let shared_result = self.last_result.clone();
        // let shared_timestamp = self.last_timestamp.clone();
        // let shared_value = self.last_value.clone();
        tokio::spawn(async move {
            loop {
                // Every 1 second, get the metric value from prometheus using reqwest.

                // Generate a url to call a prometheus query.
                let url = metadata["endpoint"].as_str().unwrap();
                let query = metadata["query"].as_str().unwrap();
                let url = format!("{}/api/v1/query", url);
                println!("url: {:?}", url);
                // println!("url_with_query: {:?}", url_with_query);
                let client = reqwest::Client::new();
                let params = vec![("query", query)];
                let response = client
                    .get(url)
                    .query(&params)
                    .send()
                    .await
                    .unwrap()
                    .json::<serde_json::Value>()
                    .await;

                println!("response: {:?}", response);
                // Update the shared value.
                if let Ok(response) = response {
                    let mut shared_result = shared_result.lock().await;
                    *shared_result = response["data"]["result"].clone();
                    // if let Some(result) = response["data"]["result"].as_array() {
                    //     if result.len() != 0 {
                    //         // Timestamp
                    //         let timestamp = &response["data"]["result"][0]["value"][0];
                    //         let mut shared_timestamp = shared_timestamp.lock().await;
                    //         *shared_timestamp = timestamp.as_f64().unwrap();
                    //         // Value
                    //         let value = &response["data"]["result"][0]["value"][1];
                    //         let mut shared_value = shared_value.lock().await;
                    //         *shared_value = value.as_str().unwrap().parse::<f64>().unwrap();
                    //     }
                    // }
                }
                // Wait for the next interval.
                interval.tick().await;
            }
        });
    }
    async fn get_value(&self) -> f64 {
        let shared_result = self.last_result.clone();
        let shared_result = shared_result.lock().await;
        let result = shared_result.as_array().unwrap();
        if result.len() != 0 {
            let value = &result[0]["value"][1];
            value.as_str().unwrap().parse::<f64>().unwrap()
        } else {
            0.0
        }
    }
    async fn get_multiple_values(&self) -> Vec<f64> {
        let shared_result = self.last_result.clone();
        let shared_result = shared_result.lock().await;
        let result = shared_result.as_array().unwrap();
        let mut values: Vec<f64> = Vec::new();
        for i in 0..result.len() {
            let value = &result[i]["value"][1];
            values.push(value.as_str().unwrap().parse::<f64>().unwrap());
        }
        values
    }
    async fn get_timestamp(&self) -> f64 {
        let shared_result = self.last_result.clone();
        let shared_result = shared_result.lock().await;
        let result = shared_result.as_array().unwrap();
        if result.len() != 0 {
            let timestamp = &result[0]["value"][0];
            timestamp.as_f64().unwrap()
        } else {
            0.0
        }
    }
}
