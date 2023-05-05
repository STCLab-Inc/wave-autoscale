use ts_rs::TS;

#[derive(TS)]
#[ts(
    export,
    export_to = "../web-app/src/types/bindings/cloudwatch-data-metric.ts"
)]
pub struct CloudwatchDataMetricMetadata {
    #[ts(type = "number")]
    pub polling_interval: u64,
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
    pub expression: String,
    #[ts(type = "number")]
    pub period: i32,
    #[ts(type = "number")]
    pub duration_seconds: i64,
}
