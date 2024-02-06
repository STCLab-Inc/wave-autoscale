use super::{object_kind::ObjectKind, plan_item_definition::PlanItemDefinition, validate_id_regex};
use serde::{Deserialize, Serialize};
use serde_valid::Validate;
use std::collections::HashMap;
use ts_rs::TS;

pub const DEFAULT_PLAN_INTERVAL: u16 = 1000;

fn default_kind() -> ObjectKind {
    ObjectKind::ScalingPlan
}
fn default_enabled() -> bool {
    false
}
fn default_variables() -> HashMap<String, serde_json::Value> {
    HashMap::new()
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
    #[validate(custom(validate_id_regex))]
    #[validate(min_length = 2)]
    pub id: String,
    #[ts(type = "object")]
    pub metadata: HashMap<String, serde_json::Value>,
    #[serde(default = "default_variables")]
    #[ts(type = "object")]
    pub variables: HashMap<String, serde_json::Value>,
    // #[ts(type = "Array<object>")]
    pub plans: Vec<PlanItemDefinition>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

impl Default for ScalingPlanDefinition {
    fn default() -> Self {
        Self {
            kind: ObjectKind::ScalingPlan,
            db_id: "".to_string(),
            id: "".to_string(),
            metadata: HashMap::new(),
            variables: HashMap::new(),
            plans: vec![],
            enabled: true,
        }
    }
}
