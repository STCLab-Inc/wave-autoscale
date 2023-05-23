use chrono::serde::ts_seconds;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_valid::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct AutoscalingHistoryDefinition {
    #[serde(default)]
    pub id: String,
    pub plan_db_id: String,
    pub plan_id: String,
    pub plan_item_json: String,
    pub metric_values_json: String,
    pub metadata_values_json: String,
    pub fail_message: String,
    #[serde(with = "ts_seconds")]
    pub created_at: chrono::DateTime<Utc>,
}
