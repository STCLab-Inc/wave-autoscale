use super::{object_kind::ObjectKind, validate_id_regex};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_valid::Validate;
use std::collections::HashMap;
use ts_rs::TS;

fn default_kind() -> ObjectKind {
    ObjectKind::Metric
}

fn default_enabled() -> bool {
    false
}

#[derive(TS)]
#[ts(
    export,
    export_to = "../web-app/src/types/bindings/metric-definition.ts"
)]
#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct MetricDefinition {
    #[serde(default = "default_kind")]
    pub kind: ObjectKind,
    #[serde(default)]
    pub db_id: String,
    #[validate(custom(validate_id_regex))]
    #[validate(min_length = 2)]
    pub id: String,
    // Example: "vector", "telegraf", "fluentbit"
    #[serde(default)]
    pub collector: String,
    #[ts(type = "object")]
    pub metadata: HashMap<String, Value>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

impl Default for MetricDefinition {
    fn default() -> Self {
        Self {
            kind: ObjectKind::Metric,
            db_id: "".to_string(),
            id: "".to_string(),
            collector: "".to_string(),
            metadata: HashMap::new(),
            enabled: true,
        }
    }
}
