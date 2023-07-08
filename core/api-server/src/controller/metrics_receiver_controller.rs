use actix_web::{
    post,
    web::{self, Bytes},
    HttpResponse, Responder,
};
use serde::Deserialize;
use serde_json::json;

use crate::app_state::AppState;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(post_metrics_receiver);
}

// [POST] /api/metrics-receiver
// #[derive(Deserialize, Validate)]
// struct PostMetricsCollectorRequest {
//     metrics: Vec<MetricDefinition>,
// }

#[derive(Deserialize)]
struct PostMetricsReceiverQuery {
    collector: String,
    metric_id: String,
}

#[post("/api/metrics-receiver")]
async fn post_metrics_receiver(
    query: web::Query<PostMetricsReceiverQuery>,
    bytes: Bytes,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let (collector, metric_id) = (query.collector.clone(), query.metric_id.clone());
    let body_text = String::from_utf8(bytes.to_vec());
    if body_text.is_err() {
        return HttpResponse::InternalServerError().body("Failed to parse body");
    }
    let body_text = body_text.unwrap();
    let body_json = serde_json::from_str::<serde_json::Value>(&body_text);
    if body_json.is_err() {
        return HttpResponse::InternalServerError().body("Failed to parse body");
    }
    let body_json = body_json.unwrap();
    let body_json = body_json.as_object();
    if body_json.is_none() {
        return HttpResponse::InternalServerError().body("Failed to parse body");
    }
    let body_json = body_json.unwrap();
    let body_json = body_json.get("metrics");
    if body_json.is_none() {
        return HttpResponse::InternalServerError().body("Failed to parse body");
    }
    let body_json = body_json.unwrap();
    let metrics = body_json.as_array();
    if metrics.is_none() {
        return HttpResponse::InternalServerError().body("Failed to parse body");
    }
    let metrics = metrics.unwrap();
    let metrics = metrics
        .iter()
        .map(|metric| {
            let metric = metric.as_object();
            if metric.is_none() {
                return "{}".to_string();
            }
            let metric = metric.unwrap();
            let metric_name = metric.get("name");
            let metric_tags = metric.get("tags");
            let metric_value = metric
                .get("counter")
                .and_then(|counter| counter.get("value"));

            json!(
            {
                "name": metric_name,
                "tags": metric_tags,
                "value": metric_value
            })
            .to_string()
        })
        .collect::<Vec<String>>();
    let json_value = serde_json::to_string(&metrics);
    if json_value.is_err() {
        return HttpResponse::InternalServerError().body("Failed to parse body");
    }
    let json_value = json_value.unwrap();
    // Save to database
    let data_layer = &app_state.data_layer;
    let result = data_layer
        .add_source_metric(collector.as_str(), metric_id.as_str(), json_value.as_str())
        .await;
    if result.is_err() {
        return HttpResponse::InternalServerError().body(format!("{:?}", result));
    }

    HttpResponse::Ok().finish()
}

#[cfg(test)]
mod tests {}
