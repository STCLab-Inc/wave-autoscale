use super::{object_kind::ObjectKind, validate_id_regex};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_valid::Validate;
use std::collections::HashMap;

fn default_kind() -> ObjectKind {
    ObjectKind::ScalingComponent
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct ScalingComponentDefinition {
    #[serde(default = "default_kind")]
    pub kind: ObjectKind,
    #[serde(default)]
    pub db_id: String,
    #[validate(custom(validate_id_regex))]
    #[validate(min_length = 2)]
    pub id: String,
    pub component_kind: String,
    pub metadata: HashMap<String, Value>,
}
