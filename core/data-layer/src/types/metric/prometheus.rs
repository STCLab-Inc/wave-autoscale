use ts_rs::TS;

#[derive(TS)]
#[ts(
    export,
    export_to = "../web-app/src/types/bindings/prometheus-metric.ts"
)]
pub struct PrometheusMetricMetadata {
    pub query: String,
    pub prometheus_url: String,
    pub prometheus_token: Option<String>,
    #[ts(type = "number")]
    pub polling_interval: u64,
}
