use serde::{Deserialize, Serialize};

use super::object_kind::ObjectKind;

#[derive(Debug, Serialize, Deserialize)]
pub struct SloDefinition {
    pub kind: ObjectKind,
    pub id: String,
}
