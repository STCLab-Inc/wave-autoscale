use serde::{Deserialize, Serialize};
use serde_valid::Validate;
use ts_rs::TS;

#[derive(TS)]
#[ts(
    export,
    export_to = "../web-app/src/types/bindings/autoscaling-history-definition.ts"
)]
#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct AutoscalingHistoryDefinition {
    #[serde(default)]
    pub id: String,
    pub plan_db_id: String,
    pub plan_id: String,
    pub plan_item_json: String,
    pub metric_values_json: String,
    pub metadata_values_json: String,
    pub fail_message: Option<String>,
}

impl AutoscalingHistoryDefinition {
    pub fn new(
        plan_db_id: String,
        plan_id: String,
        plan_item_json: String,
        metric_values_json: String,
        metadata_values_json: String,
        fail_message: Option<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            plan_db_id,
            plan_id,
            plan_item_json,
            metric_values_json,
            metadata_values_json,
            fail_message,
        }
    }
}
