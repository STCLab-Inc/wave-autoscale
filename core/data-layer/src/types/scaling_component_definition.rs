use super::{object_kind::ObjectKind, validate_id_regex};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_valid::Validate;
use std::collections::HashMap;
use ts_rs::TS;

fn default_kind() -> ObjectKind {
    ObjectKind::ScalingComponent
}
fn default_metadata() -> HashMap<String, Value> {
    HashMap::new()
}
fn default_enabled() -> bool {
    false
}

#[derive(TS)]
#[ts(
    export,
    export_to = "../web-app/src/types/bindings/scaling-component-definition.ts"
)]
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
    #[ts(type = "object")]
    #[serde(default = "default_metadata")]
    pub metadata: HashMap<String, Value>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

impl Default for ScalingComponentDefinition {
    fn default() -> Self {
        Self {
            kind: ObjectKind::ScalingComponent,
            db_id: "".to_string(),
            id: "".to_string(),
            component_kind: "".to_string(),
            metadata: HashMap::new(),
            enabled: true,
        }
    }
}
