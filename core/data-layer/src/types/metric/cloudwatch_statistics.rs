use ts_rs::TS;

#[derive(TS)]
#[ts(
    export,
    export_to = "../web-app/src/types/bindings/cloudwatch-statistics-metric.ts"
)]
pub struct CloudwatchStatisticsMetricMetadata {
    #[ts(type = "number")]
    pub polling_interval: u64,
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
    pub namespace: String,
    pub metric_name: String,
    pub dimensions: Vec<String>,
    pub statistic: String,
    #[ts(type = "number")]
    pub period: i32,
    pub unit: String,
    #[ts(type = "number")]
    pub duration_seconds: i64,
}
