use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(TS)]
#[ts(
    export,
    export_to = "../web-app/src/types/bindings/object-kind.ts"
)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ObjectKind {
    Metric,
    ScalingPlan,
    ScalingComponent,
    SLO,
}
