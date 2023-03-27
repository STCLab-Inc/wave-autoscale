use serde::{Deserialize, Serialize};

use super::object_kind::ObjectKind;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScalingTrigger {
    pub kind: ObjectKind,
    pub id: String,
}
