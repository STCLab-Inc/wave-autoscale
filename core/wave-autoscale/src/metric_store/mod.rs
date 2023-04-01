use std::{sync::Arc, collections::HashMap};

use serde_json::Value;
use tokio::sync::RwLock;

pub type MetricStore = Arc<RwLock<HashMap<String, Value>>>;

pub fn new_metric_store() -> MetricStore {
    Arc::new(RwLock::new(HashMap::new()))
}
