use self::{cloudwatch::CloudWatchMetricAdapter, prometheus::PrometheusMetricAdapter};
use anyhow::Result;
use async_trait::async_trait;
use data_layer::Metric;
pub mod cloudwatch;
pub mod prometheus;

pub fn create_metric_adapter(metric: &Metric) -> Result<Box<dyn MetricAdapter>> {
    // Get a value of metric and clone it.
    let cloned_metric = metric.clone();
    match metric.metric_kind.as_str() {
        PrometheusMetricAdapter::METRIC_KIND => Ok(Box::new(PrometheusMetricAdapter::new(cloned_metric))),
        CloudWatchMetricAdapter::METRIC_KIND => Ok(Box::new(CloudWatchMetricAdapter::new(cloned_metric))),
        _ => Err(anyhow::anyhow!(
            "Metric adapter not implemented for this kind"
        )),
    }
}

#[async_trait]
pub trait MetricAdapter {
    async fn run(&self);
    async fn get_value(&self) -> f64;
    async fn get_multiple_values(&self) -> Vec<f64>;
    async fn get_timestamp(&self) -> f64;
    fn get_metric_kind(&self) -> &str;
}
