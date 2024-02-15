use std::collections::HashMap;

use super::validate_id_regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_valid::Validate;
use ts_rs::TS;

fn default_cool_down() -> Option<u64> {
    None
}

#[derive(TS)]
#[ts(
    export,
    export_to = "../web-app/src/types/bindings/plan-item-definition.ts"
)]
#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct PlanItemDefinition {
    #[validate(custom(validate_id_regex))]
    #[validate(min_length = 2)]
    pub id: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub expression: Option<String>,
    #[serde(default)]
    pub cron_expression: Option<String>,
    #[serde(default = "default_cool_down")]
    pub cool_down: Option<u64>,
    #[serde(default)]
    pub priority: i16,
    #[ts(type = "Array<any>")]
    pub scaling_components: Vec<Value>,
    #[ts(type = "any")]
    pub ui: Option<HashMap<String, Value>>,
}
