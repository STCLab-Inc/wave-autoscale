use super::{object_kind::ObjectKind, validate_id_regex};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_valid::Validate;
use std::collections::HashMap;
use ts_rs::TS;

fn default_kind() -> ObjectKind {
    ObjectKind::Metric
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
    // Example: "prometheus", "cloudwatch-statistics"
    pub metric_kind: String,
    #[ts(type = "object")]
    pub metadata: HashMap<String, Value>,
}
