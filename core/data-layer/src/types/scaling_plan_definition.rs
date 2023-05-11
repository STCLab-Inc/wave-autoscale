use super::{object_kind::ObjectKind, validate_id_regex};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_valid::Validate;
use ts_rs::TS;

fn default_kind() -> ObjectKind {
    ObjectKind::ScalingPlan
}

#[derive(TS)]
#[ts(
    export,
    export_to = "../web-app/src/types/bindings/scaling-plan-definition.ts"
)]
#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct ScalingPlanDefinition {
    #[serde(default = "default_kind")]
    pub kind: ObjectKind,
    #[serde(default)]
    pub db_id: String,
    #[serde(default)]
    pub title: String,
    #[validate(custom(validate_id_regex))]
    #[validate(min_length = 2)]
    pub id: String,
    #[ts(type = "Array<object>")]
    pub plans: Vec<Value>,
}
