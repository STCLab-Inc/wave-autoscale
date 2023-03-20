use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ObjectKind {
    Metric,
    ScalingPlan,
    ScalingTrigger,
    SLO,
}
