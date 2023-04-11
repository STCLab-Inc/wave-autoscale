use super::MetricAdapter;
use crate::{
    metric_store::MetricStore,
    util::{aws_region::get_aws_region_static_str, string::make_ascii_titlecase},
};
use async_trait::async_trait;
use aws_config::SdkConfig;
use aws_sdk_cloudwatch::{
    types::{Dimension, StandardUnit, Statistic},
    Client as CloudWatchClient,
};
use aws_smithy_types::DateTime;
use aws_smithy_types_convert::date_time::DateTimeExt;
use chrono::Utc;
use data_layer::MetricDefinition;
use serde_json::Value;
use std::{mem::ManuallyDrop, time::Duration};
use tokio::{task::JoinHandle, time};
// This is a metric adapter for AWS CloudWatch Metrics.
pub struct CloudWatchMetricAdapter {
    task: Option<JoinHandle<()>>,
    metric: MetricDefinition,
    metric_store: MetricStore,
}

impl CloudWatchMetricAdapter {
    pub const METRIC_KIND: &'static str = "cloudwatch";

    // Functions
    pub fn new(metric: MetricDefinition, metric_store: MetricStore) -> Self {
        CloudWatchMetricAdapter {
            task: None,
            metric,
            metric_store,
        }
    }
}

const DEFAULT_POLLING_INTERVAL: u64 = 1000;

#[async_trait]
impl MetricAdapter for CloudWatchMetricAdapter {
    fn get_metric_kind(&self) -> &str {
        CloudWatchMetricAdapter::METRIC_KIND
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

        // println!("CloudWatchMetricAdapter::run() - shared_config: {:?}", shared_config);

        let task = tokio::spawn(async move {
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
                let mut metric_statistics_builder = cw_client.get_metric_statistics();
                if let Some(Value::String(namespace)) = metadata.get("namespace") {
                    metric_statistics_builder = metric_statistics_builder.namespace(namespace);
                }
                if let Some(Value::String(metric_name)) = metadata.get("metric_name") {
                    metric_statistics_builder = metric_statistics_builder.metric_name(metric_name);
                }
                if let Some(Value::Array(dimensions)) = metadata.get("dimensions") {
                    let mut dimension_list = Vec::new();
                    for dimension in dimensions {
                        if let (Some(Value::String(name)), Some(Value::String(value))) =
                            (dimension.get("name"), dimension.get("value"))
                        {
                            let dimension_builder = Dimension::builder()
                                .name(name.to_string())
                                .value(value.to_string())
                                .build();
                            dimension_list.push(dimension_builder);
                        }
                    }
                    metric_statistics_builder =
                        metric_statistics_builder.set_dimensions(Some(dimension_list));
                }
                if let Some(Value::String(statistic)) = metadata.get("statistic") {
                    let statistic = make_ascii_titlecase(statistic);
                    let statistic_str: &str = statistic.as_str();
                    metric_statistics_builder =
                        metric_statistics_builder.statistics(Statistic::from(statistic_str));
                }
                if let Some(period) = metadata.get("period").and_then(Value::as_i64) {
                    metric_statistics_builder = metric_statistics_builder.period(period as i32);
                }
                if let Some(Value::String(unit)) = metadata.get("unit") {
                    let unit = make_ascii_titlecase(unit);
                    let unit_str: &str = unit.as_str();
                    metric_statistics_builder =
                        metric_statistics_builder.unit(StandardUnit::from(unit_str));
                }
                if let Some(duration_seconds) =
                    metadata.get("duration_seconds").and_then(Value::as_i64)
                {
                    let end_time = Utc::now();
                    let start_time: DateTime = DateTime::from_chrono_utc(
                        end_time - chrono::Duration::seconds(duration_seconds),
                    );
                    let end_time = DateTime::from_chrono_utc(end_time);
                    metric_statistics_builder = metric_statistics_builder.start_time(start_time);
                    metric_statistics_builder = metric_statistics_builder.end_time(end_time);
                }

                let response = metric_statistics_builder.send().await;

                if let Ok(response) = response {
                    if let Some(datapoints) = response.datapoints {
                        if let Some(last_datapoint) = datapoints.last() {
                            let values = [
                                last_datapoint.sample_count,
                                last_datapoint.average,
                                last_datapoint.sum,
                                last_datapoint.minimum,
                                last_datapoint.maximum,
                            ];
                            for value in &values {
                                if let Some(value) = value {
                                    let mut shared_metric_store = shared_metric_store.write().await;
                                    shared_metric_store
                                        .insert(metric_id.clone(), Value::from(*value));
                                    break;
                                }
                            }
                        }
                    }
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
