use crate::app_state::AppState;
use actix_web::{
    post,
    web::{self, Bytes},
    HttpResponse, Responder,
};
use serde::Deserialize;
use serde_json::json;
use tracing::{debug, error, info};

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
    debug!("Received metrics from collector: {}", query.collector);
    let (collector, metric_id) = (query.collector.clone(), query.metric_id.clone());
    let body_text = String::from_utf8(bytes.to_vec());
    if body_text.is_err() {
        error!("Failed to parse body: {:?}", body_text);
        return HttpResponse::BadRequest().body("Failed to parse body");
    }
    let body_text = body_text.unwrap();
    let body_json = serde_json::from_str::<serde_json::Value>(&body_text);
    if body_json.is_err() {
        error!("Failed to parse body as JSON: {:?}", body_json);
        return HttpResponse::BadRequest().body("Invalid JSON body");
    }
    let body_json = body_json.unwrap();
    let body_json = body_json.as_object();
    if body_json.is_none() {
        error!("Invalid JSON body. It should be an object: {:?}", body_json);
        return HttpResponse::BadRequest().body("Invalid JSON body. It should be an object");
    }
    let body_json = body_json.unwrap();
    let body_json = body_json.get("metrics");
    if body_json.is_none() {
        error!("Invalid JSON body. Missing 'metrics': {:?}", body_json);
        return HttpResponse::BadRequest().body("Invalid JSON body. Missing 'metrics'");
    }
    let body_json = body_json.unwrap();
    let metrics = body_json.as_array();
    if metrics.is_none() {
        error!(
            "Invalid JSON body. 'metrics' should be an array: {:?}",
            body_json
        );
        return HttpResponse::BadRequest().body("Invalid JSON body. 'metrics' should be an array");
    }
    let metrics = metrics.unwrap();
    let mut json_value: Vec<serde_json::Value> = vec![];

    if collector == "vector" {
        for metric in metrics {
            let metric = metric.as_object();
            if metric.is_none() {
                error!(
                    "Invalid JSON body. Failed to parse 'metrics' as object: {:?}",
                    metric
                );
                return HttpResponse::BadRequest()
                    .body("Invalid JSON body. Failed to parse 'metrics' as object");
            }
            let metric = metric.unwrap();
            let metric_name = metric.get("name");
            let metric_tags = metric.get("tags");
            let metric_value_gauge = metric.get("gauge").and_then(|gauge| gauge.get("value"));
            let metric_value_counter = metric
                .get("counter")
                .and_then(|counter| counter.get("value"));
            let mut metric_value = metric_value_counter;
            if metric_value.is_none() {
                metric_value = metric_value_gauge;
            }
            let timestamp = metric.get("timestamp");

            json_value.push(json!(
            {
                "name": metric_name,
                "tags": metric_tags,
                "value": metric_value,
                "timestamp": timestamp,
            }));
        }
    } else if collector == "telegraf" {
        for metric in metrics {
            let metric = metric.as_object();
            if metric.is_none() {
                error!(
                    "Invalid JSON body. Failed to parse 'metrics' as object: {:?}",
                    metric
                );
                return HttpResponse::BadRequest()
                    .body("Invalid JSON body. Failed to parse 'metrics' as object");
            }
            let metric = metric.unwrap();
            let Some(metric_name) = metric.get("name").and_then(serde_json::Value::as_str) else {
                error!("Invalid JSON body. Missing 'name' in metric: {:?}", metric);
                continue;
            };
            let metric_tags = metric.get("tags");
            let Some(fields) = metric.get("fields").and_then(serde_json::Value::as_object) else {
                error!("Invalid JSON body. Missing 'fields' in metric: {:?}", metric);
                continue;
            };
            let timestamp = metric.get("timestamp");

            for (field_name, field_value) in fields {
                json_value.push(json!(
                {
                    "name": format!("{}_{}", metric_name, field_name),
                    "tags": metric_tags,
                    "value": field_value,
                    "timestamp": timestamp,
                }));
            }
        }
    } else if collector == "wagenerator" {
        for metric in metrics {
            let metric = metric.as_object();
            if metric.is_none() {
                error!(
                    "Invalid JSON body. Failed to parse 'metrics' as object: {:?}",
                    metric
                );
                return HttpResponse::BadRequest()
                    .body("Invalid JSON body. Failed to parse 'metrics' as object");
            }
            let metric = metric.unwrap();
            let Some(metric_value) = metric.get("value").and_then(serde_json::Value::as_f64) else {
                error!("Invalid JSON body. Missing 'value' in metric: {:?}", metric);
                continue;
            };
            let timestamp = metric.get("timestamp");

            json_value.push(json!(
            {
                "value": metric_value,
                "timestamp": timestamp,
            }));
        }
    } else {
        error!("Invalid collector. Only 'vector' and 'telegraf' are supported");
        return HttpResponse::BadRequest()
            .body("Invalid collector. Only 'vector' and 'telegraf' are supported");
    }
    let json_value = serde_json::to_string(&json_value);
    if json_value.is_err() {
        error!("Failed to serialize JSON: {:?}", json_value);
        return HttpResponse::InternalServerError().body("Failed to serialize JSON");
    }
    let json_value = json_value.unwrap();
    // Save to data_layer
    let data_layer = &app_state.data_layer;
    let result = data_layer
        .add_source_metrics_in_data_layer(
            collector.as_str(),
            metric_id.as_str(),
            json_value.as_str(),
        )
        .await;
    if result.is_err() {
        error!("Failed to save metric into the data-layer: {:?}", result);
        return HttpResponse::InternalServerError().body(format!("{:?}", result));
    }
    debug!(
        "[api-server] Saved metrics into the data-layer: collector - {:?}, metric_id - {:?}, json_value - {:?}",
        collector.as_str(),
        metric_id.as_str(),
        json_value.as_str()
    );
    info!(
        "[api-server] Saved metrics into the data-layer: collector - {:?}, metric_id - {:?}, size: {:?}",
        collector.as_str(),
        metric_id.as_str(),
        json_value.len()
    );
    HttpResponse::Ok().finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_utils::get_app_state_for_test;
    use actix_web::{http, test, App};

    #[actix_web::test]
    #[tracing_test::traced_test]
    async fn test_post_metrics_receiver_vector() {
        let app_state = get_app_state_for_test().await;
        let app = test::init_service(App::new().app_data(app_state).configure(init)).await;

        let req = test::TestRequest::post()
            .uri("/api/metrics-receiver?collector=vector&metric_id=metric_vector")
            .set_payload(
                r#"{
                    "metrics": [
                        {
                            "name": "metric1",
                            "tags": {
                                "tag1": "value1"
                            },
                            "gauge": {
                                "value": 1
                            }
                        }
                    ]
                }"#,
            )
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        // TODO: Check database
    }

    #[actix_web::test]
    #[tracing_test::traced_test]
    async fn test_post_metrics_receiver_telegraf() {
        let app_state = get_app_state_for_test().await;
        let app = test::init_service(App::new().app_data(app_state).configure(init)).await;

        let req = test::TestRequest::post()
            .uri("/api/metrics-receiver?collector=telegraf&metric_id=metric_telegraf")
            .set_payload(
                r#"{
                    "metrics": [
                        {
                            "name": "metric1",
                            "tags": {
                                "tag1": "value1"
                            },
                            "fields": {
                                "field1": 1,
                                "field2": 2,
                                "field3": 3
                            }
                        }
                    ]
                }"#,
            )
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
        // TODO: Check database
    }

    #[actix_web::test]
    #[tracing_test::traced_test]
    async fn test_post_metrics_receiver_invalid_collector() {
        let app_state = get_app_state_for_test().await;
        let app = test::init_service(App::new().app_data(app_state).configure(init)).await;

        let req = test::TestRequest::post()
            .uri("/api/metrics-receiver?collector=invalid&metric_id=metric_invalid")
            .set_payload(
                r#"{
                    "metrics": [
                        {
                            "name": "metric1",
                            "tags": {
                                "tag1": "value1"
                            },
                            "gauge": {
                                "value": 1
                            }
                        }
                    ]
                }"#,
            )
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    }
}
