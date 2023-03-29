use super::object_kind::ObjectKind;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScalingComponentDefinition {
    pub kind: ObjectKind,
    pub id: String,
    pub component_kind: String,
    pub metadata: HashMap<String, Value>,
}
