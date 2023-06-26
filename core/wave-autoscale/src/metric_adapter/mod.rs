use self::{
    cloudwatch_data::CloudWatchDataMetricAdapter,
    cloudwatch_statistics::CloudWatchStatisticsMetricAdapter, prometheus::PrometheusMetricAdapter,
};
use crate::metric_store::SharedMetricStore;
use anyhow::Result;
use async_trait::async_trait;
use data_layer::MetricDefinition;
use std::collections::HashMap;


pub mod cloudwatch_data;
pub mod cloudwatch_statistics;
pub mod prometheus;

// Factory method to create a metric adapter
pub fn create_metric_adapter(
    definition: &MetricDefinition,
    metric_store: SharedMetricStore,
) -> Result<Box<dyn MetricAdapter>> {
    // Get a value of metric and clone it.
    let cloned_definition = definition.clone();
    match definition.metric_kind.as_str() {
        PrometheusMetricAdapter::METRIC_KIND => Ok(Box::new(PrometheusMetricAdapter::new(
            cloned_definition,
            metric_store,
        ))),
        CloudWatchDataMetricAdapter::METRIC_KIND => Ok(Box::new(CloudWatchDataMetricAdapter::new(
            cloned_definition,
            metric_store,
        ))),
        CloudWatchStatisticsMetricAdapter::METRIC_KIND => Ok(Box::new(
            CloudWatchStatisticsMetricAdapter::new(cloned_definition, metric_store),
        )),
        _ => Err(anyhow::anyhow!(
            "Metric adapter not implemented for this kind"
        )),
    }
}

#[async_trait]
pub trait MetricAdapter {
    fn run(&mut self);
    fn stop(&mut self);
    fn get_id(&self) -> &str;
    fn get_metric_kind(&self) -> &str;
}

//
// MetricAdapterManager manages metric adapters
//
pub struct MetricAdapterManager {
    metric_adapters: HashMap<String, Box<dyn MetricAdapter>>,
    metric_store: SharedMetricStore,
}

impl MetricAdapterManager {
    pub fn new(metric_store: SharedMetricStore) -> Self {
        MetricAdapterManager {
            metric_adapters: HashMap::new(),
            metric_store,
        }
    }

    pub fn add_definition(&mut self, definition: MetricDefinition) -> Result<()> {
        let metric_adapter = create_metric_adapter(&definition, self.metric_store.clone())?;
        self.add_metric_adapter(metric_adapter);
        Ok(())
    }

    pub fn add_definitions(&mut self, definitions: Vec<MetricDefinition>) -> Result<()> {
        for definition in definitions {
            self.add_definition(definition)?;
        }
        Ok(())
    }

    pub fn get_metric_adapters(&self) -> &HashMap<String, Box<dyn MetricAdapter>> {
        &self.metric_adapters
    }

    pub fn remove_all_definitions(&mut self) {
        self.metric_adapters.clear();
    }

    pub fn add_metric_adapter(&mut self, metric_adapter: Box<dyn MetricAdapter>) {
        self.metric_adapters
            .insert(metric_adapter.get_id().to_string(), metric_adapter);
    }

    pub fn run(&mut self) {
        for metric_adapter in self.metric_adapters.values_mut() {
            metric_adapter.run();
        }
    }

    pub fn stop(&mut self) {
        for metric_adapter in self.metric_adapters.values_mut() {
            metric_adapter.stop();
        }
    }

    pub fn get_metric_adapter(&self, id: &str) -> Option<&Box<dyn MetricAdapter>> {
        self.metric_adapters.get(id)
    }
}
