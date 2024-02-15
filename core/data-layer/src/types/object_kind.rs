use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(TS)]
#[ts(export, export_to = "../web-app/src/types/bindings/object-kind.ts")]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ObjectKind {
    Metric,
    ScalingPlan,
    ScalingComponent,
}

impl std::fmt::Display for ObjectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectKind::Metric => write!(f, "Metric"),
            ObjectKind::ScalingPlan => write!(f, "ScalingPlan"),
            ObjectKind::ScalingComponent => write!(f, "ScalingComponent"),
        }
    }
}
