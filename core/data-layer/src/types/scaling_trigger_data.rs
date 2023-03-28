use super::object_kind::ObjectKind;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScalingTriggerData {
    pub kind: ObjectKind,
    pub id: String,
    pub trigger_kind: String,
    pub metadata: HashMap<String, Value>,
}
