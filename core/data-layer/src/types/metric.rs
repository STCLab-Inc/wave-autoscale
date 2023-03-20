use serde::{Deserialize, Serialize};

use super::kind::ObjectKind;

#[derive(Debug, Serialize, Deserialize)]
pub struct Metric {
    kind: ObjectKind,
    metric_kind: String,
}
