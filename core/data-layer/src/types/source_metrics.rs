use get_size::GetSize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, GetSize)]
pub struct SourceMetrics {
    pub id: String,
    pub collector: String,
    pub metric_id: String,
    pub json_value: String,
    pub create_dt: String,
}
