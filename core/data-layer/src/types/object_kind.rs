use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ObjectKind {
    Metric,
    ScalingPlan,
    ScalingTrigger,
    SLO,
}
