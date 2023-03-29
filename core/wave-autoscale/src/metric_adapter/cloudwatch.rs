use async_trait::async_trait;
use data_layer::Metric;

use super::MetricAdapter;

pub struct CloudWatchMetricAdapter {
    metric: Metric,
}

impl CloudWatchMetricAdapter {
    pub const METRIC_KIND: &'static str = "cloudwatch";
    pub fn new(metric: Metric) -> Self {
        CloudWatchMetricAdapter { metric }
    }
}

#[async_trait]
impl MetricAdapter for CloudWatchMetricAdapter {
    fn get_metric_kind(&self) -> &str {
        CloudWatchMetricAdapter::METRIC_KIND
    }
    fn get_id(&self) -> &str {
        &self.metric.id
    }
    async fn run(&self) {}
    async fn get_value(&self) -> f64 {
        0.0
    }
    async fn get_multiple_values(&self) -> Vec<f64> {
        vec![]
    }
    async fn get_timestamp(&self) -> f64 {
        0.0
    }
}
