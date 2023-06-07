use super::MetricAdapter;
use crate::{metric_store::SharedMetricStore, util::aws_region::get_aws_region_static_str};
use async_trait::async_trait;
use aws_config::SdkConfig;
use aws_sdk_cloudwatch::Client as CloudWatchClient;
use aws_smithy_types::DateTime;
use aws_smithy_types_convert::date_time::DateTimeExt;
use chrono::Utc;
use data_layer::MetricDefinition;
use serde_json::Value;
use std::time::Duration;
use tokio::{task::JoinHandle, time};
// This is a metric adapter for AWS CloudWatch Metrics.
pub struct CloudWatchDataMetricAdapter {
    task: Option<JoinHandle<()>>,
    metric: MetricDefinition,
    metric_store: SharedMetricStore,
}

impl CloudWatchDataMetricAdapter {
    pub const METRIC_KIND: &'static str = "cloudwatch-data";

    // Functions
    pub fn new(metric: MetricDefinition, metric_store: SharedMetricStore) -> Self {
        CloudWatchDataMetricAdapter {
            task: None,
            metric,
            metric_store,
        }
    }
}

const DEFAULT_POLLING_INTERVAL: u64 = 1000;

#[async_trait]
impl MetricAdapter for CloudWatchDataMetricAdapter {
    fn get_metric_kind(&self) -> &str {
        CloudWatchDataMetricAdapter::METRIC_KIND
    }
    fn get_id(&self) -> &str {
        &self.metric.id
    }
    fn run(&mut self) -> JoinHandle<()> {
        self.stop();

        let metadata = self.metric.metadata.clone();

        let polling_interval: u64 = metadata["polling_interval"]
            .as_u64()
            .unwrap_or(DEFAULT_POLLING_INTERVAL);
        let mut interval = time::interval(Duration::from_millis(polling_interval));

        // Concurrency
        let shared_metric_store = self.metric_store.clone();
        let metric_id = self.get_id().to_string();

        // println!("CloudWatchDataMetricAdapter::run() - shared_config: {:?}", shared_config);

        // self.task = Some(task);

        tokio::spawn(async move {
            let mut shared_config: SdkConfig = aws_config::from_env().load().await;
            if let (
                Some(Value::String(access_key)),
                Some(Value::String(secret_key)),
                Some(Value::String(region)),
            ) = (
                metadata.get("access_key"),
                metadata.get("secret_key"),
                metadata.get("region"),
            ) {
                // Initialize AWS CloudWatch client
                let credentials = aws_sdk_cloudwatch::config::Credentials::new(
                    access_key,
                    secret_key,
                    None,
                    None,
                    "wave-autoscale",
                );
                // aws_config needs a static region string
                let region_static: &'static str = get_aws_region_static_str(region);
                shared_config = aws_config::from_env()
                    .region(region_static)
                    .credentials_provider(credentials)
                    .load()
                    .await;
            }
            let cw_client = CloudWatchClient::new(&shared_config);

            loop {
                let mut metric_data_builder = cw_client.get_metric_data();
                if let Some(Value::String(expression)) = metadata.get("expression") {
                    // id must be unique
                    let mut uuid = uuid::Uuid::new_v4().to_string().replace('-', "");
                    // CloudWatch requires the first character of the ID to be a letter.
                    uuid.insert(0, 'i');
                    let mut metric_data_query =
                        aws_sdk_cloudwatch::types::MetricDataQuery::builder()
                            .id(uuid)
                            .expression(expression.to_string());

                    if let Some(period) = metadata.get("period").and_then(Value::as_i64) {
                        metric_data_query = metric_data_query.period(period as i32);
                    }
                    let metric_data_query = metric_data_query.build();
                    metric_data_builder =
                        metric_data_builder.metric_data_queries(metric_data_query);
                }

                if let Some(duration_seconds) =
                    metadata.get("duration_seconds").and_then(Value::as_i64)
                {
                    let end_time = Utc::now();
                    let start_time: DateTime = DateTime::from_chrono_utc(
                        end_time - chrono::Duration::seconds(duration_seconds),
                    );
                    let end_time = DateTime::from_chrono_utc(end_time);
                    metric_data_builder = metric_data_builder.start_time(start_time);
                    metric_data_builder = metric_data_builder.end_time(end_time);
                }

                let response = metric_data_builder.send().await;

                if let Ok(response) = response {
                    if let Some(data_results) = response.metric_data_results() {
                        if let Some(data_result) = data_results.first() {
                            if let Some(values) = data_result.values() {
                                if let Some(value) = values.first() {
                                    println!(
                                        "CloudWatchDataMetricAdapter::run() - value: {:?}",
                                        value
                                    );
                                    let mut shared_metric_store = shared_metric_store.write().await;
                                    shared_metric_store
                                        .insert(metric_id.clone(), Value::from(*value));
                                }
                            }
                        }
                    }
                }
                // Wait for the next interval.
                interval.tick().await;
            }
        })
    }
    fn stop(&mut self) {
        if let Some(task) = &self.task {
            task.abort();
            self.task = None;
        }
    }
}
