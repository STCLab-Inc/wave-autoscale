use super::object_kind::ObjectKind;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScalingPlanDefinition {
    pub kind: ObjectKind,
    pub id: String,
    pub plans: Vec<Value>,
}
