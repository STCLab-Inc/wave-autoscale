/**
 * MetricStore
 *
 * The metric store is a hashmap that stores the latest metric values for each metric.
 * The metric store is shared between the metric adapters and the scaling components.
 * So it's defined with an Arc and a RwLock.
 */
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

pub type SharedMetricStore = Arc<RwLock<HashMap<String, Value>>>;

pub fn new_shared_metric_store() -> SharedMetricStore {
    Arc::new(RwLock::new(HashMap::new()))
}
