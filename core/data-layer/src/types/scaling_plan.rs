use serde::{Deserialize, Serialize};

use super::kind::ObjectKind;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScalingPlan {
    kind: ObjectKind,
}