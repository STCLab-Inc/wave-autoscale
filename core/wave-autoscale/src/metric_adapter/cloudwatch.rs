use async_trait::async_trait;
use data_layer::MetricDefinition;

use super::MetricAdapter;

pub struct CloudWatchMetricAdapter {
    metric: MetricDefinition,
}

impl CloudWatchMetricAdapter {
    pub const METRIC_KIND: &'static str = "cloudwatch";
    pub fn new(metric: MetricDefinition) -> Self {
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
    async fn run(&mut self) {}
    fn stop(&mut self) {}
}
