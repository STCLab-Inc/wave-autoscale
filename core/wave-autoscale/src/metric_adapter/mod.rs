use std::collections::HashMap;

use self::{cloudwatch::CloudWatchMetricAdapter, prometheus::PrometheusMetricAdapter};
use anyhow::Result;
use async_trait::async_trait;
use data_layer::MetricDefinition;
pub mod cloudwatch;
pub mod prometheus;

pub fn create_metric_adapter(definition: &MetricDefinition) -> Result<Box<dyn MetricAdapter>> {
    // Get a value of metric and clone it.
    let cloned_definition = definition.clone();
    match definition.metric_kind.as_str() {
        PrometheusMetricAdapter::METRIC_KIND => {
            Ok(Box::new(PrometheusMetricAdapter::new(cloned_definition)))
        }
        CloudWatchMetricAdapter::METRIC_KIND => {
            Ok(Box::new(CloudWatchMetricAdapter::new(cloned_definition)))
        }
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
    fn get_id(&self) -> &str;
    fn get_metric_kind(&self) -> &str;
}

pub struct MetricAdapterManager {
    metric_adapters: HashMap<String, Box<dyn MetricAdapter>>,
}

impl MetricAdapterManager {
    pub fn new() -> Self {
        MetricAdapterManager {
            metric_adapters: HashMap::new(),
        }
    }

    pub fn add_definition(&mut self, definition: MetricDefinition) -> Result<()> {
        let metric_adapter = create_metric_adapter(&definition)?;
        self.add_metric_adapter(metric_adapter);
        Ok(())
    }

    pub fn add_definitions(&mut self, definitions: Vec<MetricDefinition>) -> Result<()> {
        for definition in definitions {
            self.add_definition(definition)?;
        }
        Ok(())
    }

    pub fn add_metric_adapter(&mut self, metric_adapter: Box<dyn MetricAdapter>) {
        self.metric_adapters
            .insert(metric_adapter.get_id().to_string(), metric_adapter);
    }

    pub async fn run(&self) {
        for metric_adapter in self.metric_adapters.values() {
            metric_adapter.run().await;
        }
    }

    pub fn get_metric_adapter(&self, id: &str) -> Option<&Box<dyn MetricAdapter>> {
        self.metric_adapters.get(id)
    }

    pub async fn get_value(&self, id: &str) -> f64 {
        if let Some(metric_adapter) = self.metric_adapters.get(id) {
            return metric_adapter.get_value().await;
        }
        0.0
    }

    pub async fn get_multiple_values(&self, id: &str) -> Vec<f64> {
        if let Some(metric_adapter) = self.metric_adapters.get(id) {
            return metric_adapter.get_multiple_values().await;
        }
        vec![]
    }

    pub async fn get_timestamp(&self, id: &str) -> f64 {
        if let Some(metric_adapter) = self.metric_adapters.get(id) {
            return metric_adapter.get_timestamp().await;
        }
        0.0
    }
}
