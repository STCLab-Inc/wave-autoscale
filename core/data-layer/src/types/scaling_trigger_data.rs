use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::object_kind::ObjectKind;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScalingTriggerData {
    pub kind: ObjectKind,
    pub id: String,
    pub trigger_kind: String,
    pub metadata: HashMap<String, String>,
}
