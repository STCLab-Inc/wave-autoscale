use get_size::GetSize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, GetSize)]
pub struct SourceMetrics {
    pub json_value: String,
}
