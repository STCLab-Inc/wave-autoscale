use super::{object_kind::ObjectKind, validate_id_regex};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_valid::Validate;

fn default_kind() -> ObjectKind {
    ObjectKind::ScalingPlan
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct ScalingPlanDefinition {
    #[serde(default = "default_kind")]
    pub kind: ObjectKind,
    #[serde(default)]
    pub db_id: String,
    #[validate(custom(validate_id_regex))]
    #[validate(min_length = 2)]
    pub id: String,
    pub plans: Vec<Value>,
}
