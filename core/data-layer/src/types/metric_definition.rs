use super::{object_kind::ObjectKind, validate_id_regex};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_valid::Validate;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct MetricDefinition {
    pub kind: ObjectKind,
    #[validate(custom(validate_id_regex))]
    #[validate(min_length = 2)]
    pub id: String,
    pub metric_kind: String,
    pub metadata: HashMap<String, Value>,
}
