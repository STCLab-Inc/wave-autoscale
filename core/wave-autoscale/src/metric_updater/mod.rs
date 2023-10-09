use data_layer::data_layer::DataLayer;
use log::debug;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

pub type SharedMetricUpdater = Arc<RwLock<MetricUpdater>>;

// TODO: move this to config
// 1 minute
const MAX_TIME_GREATER_THAN: u64 = 1000 * 60;
pub struct MetricUpdater {
    metric_values: Arc<RwLock<HashMap<String, Value>>>,
    data_layer: Arc<DataLayer>,
    polling_interval: u64,
    task: Option<tokio::task::JoinHandle<()>>,
}

impl MetricUpdater {
    pub fn new(data_layer: Arc<DataLayer>, polling_interval: u64) -> Self {
        let metrics = Arc::new(RwLock::new(HashMap::new()));
        MetricUpdater {
            metric_values: metrics,
            data_layer,
            polling_interval,
            task: None,
        }
    }
    pub fn new_shared(data_layer: Arc<DataLayer>, polling_interval: u64) -> SharedMetricUpdater {
        Arc::new(RwLock::new(MetricUpdater::new(
            data_layer,
            polling_interval,
        )))
    }
    pub async fn run(&mut self) {
        self.stop();

        let metric_values = self.metric_values.clone();
        let polling_interval = self.polling_interval;

        let metric_definitions = self.data_layer.get_all_metrics().await;
        if metric_definitions.is_err() {
            return;
        }
        let metric_definitions = metric_definitions.unwrap();
        let metric_ids = metric_definitions
            .iter()
            .map(|m| m.id.clone())
            .collect::<Vec<String>>();

        let data_layer = self.data_layer.clone();
        let task = tokio::spawn(async move {
            loop {
                let new_metric_values = data_layer
                    .get_source_metrics_values(metric_ids.clone(), MAX_TIME_GREATER_THAN)
                    .await;

                debug!("new_metric_values: {:?}", new_metric_values);

                if let Ok(new_metric_values) = new_metric_values {
                    let mut metric_values = metric_values.write().await;
                    for (metric_id, metric_value) in new_metric_values {
                        metric_values.insert(metric_id, metric_value);
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(polling_interval)).await;
            }
        });

        self.task = Some(task);
    }

    pub fn stop(&mut self) {
        self.metric_values = Arc::new(RwLock::new(HashMap::new()));
        if let Some(task) = self.task.take() {
            tokio::task::block_in_place(|| {
                task.abort();
            });
        }
    }

    pub async fn get_metric_values(&self) -> Result<HashMap<String, Value>, String> {
        let metric_values = self.metric_values.read().await;
        Ok(metric_values.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_layer::data_layer::DataLayer;
    use data_layer::types::object_kind::ObjectKind;
    use data_layer::MetricDefinition;
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_metric_updater() {
        let data_layer = DataLayer::new("").await;
        data_layer.sync("").await;
        let data_layer = Arc::new(data_layer);
        let metric_definitions = vec![MetricDefinition {
            id: "metric1".to_string(),
            metadata: HashMap::new(),
            kind: ObjectKind::Metric,
            db_id: "".to_string(),
            collector: "vector".to_string(),
        }];
        let _ = data_layer.add_metrics(metric_definitions).await;

        let metric = json!([
            {
                "name": "test",
                "tags": {
                    "tag1": "value1"
                },
                "value": 1,
            }
        ])
        .to_string();
        let metric = metric.as_str();

        let _ = data_layer
            .add_source_metric("vector", "metric1", metric)
            .await;

        let mut metric_updater = MetricUpdater::new(data_layer.clone(), 1000);
        metric_updater.run().await;
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        let metric_values: tokio::sync::RwLockReadGuard<'_, HashMap<String, Value>> =
            metric_updater.metric_values.read().await;
        assert_eq!(metric_values.len(), 1);
    }
}
