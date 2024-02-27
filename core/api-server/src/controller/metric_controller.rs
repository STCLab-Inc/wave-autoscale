use crate::app_state::AppState;
use actix_web::{
    get, post,
    web::{self},
    HttpResponse, Responder,
};
use serde::Deserialize;
use tracing::{debug, error};
use validator::Validate;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_metrics)
        .service(get_metrics_yaml)
        .service(post_metrics_yaml);
}

#[get("/api/metrics")]
async fn get_metrics(app_state: web::Data<AppState>) -> impl Responder {
    debug!("Getting all metrics");
    let metrics = app_state.data_layer.get_all_metrics_json().await;
    if metrics.is_err() {
        error!("Failed to get metrics: {:?}", metrics);
        return HttpResponse::InternalServerError().body(format!("{:?}", metrics));
    }
    HttpResponse::Ok().json(metrics.unwrap())
}

#[get("/api/metrics/yaml")]
async fn get_metrics_yaml(app_state: web::Data<AppState>) -> impl Responder {
    debug!("Getting all metrics in YAML");
    let metrics = app_state.data_layer.get_metric_yamls().await;
    if metrics.is_err() {
        error!("Failed to get metrics: {:?}", metrics);
        return HttpResponse::InternalServerError().body(format!("{:?}", metrics));
    }
    HttpResponse::Ok().json(metrics.unwrap())
}

// [POST] /metrics/yaml
#[derive(Deserialize, Validate)]
struct PostMetricsYaml {
    yaml: String,
}

#[post("/api/metrics/yaml")]
async fn post_metrics_yaml(
    request: web::Json<PostMetricsYaml>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    debug!("Adding metrics: {:?}", request.yaml);
    let yaml = request.yaml.as_str();
    let result = app_state.data_layer.sync_metric_yaml(yaml).await;
    if result.is_err() {
        error!("Failed to add metrics: {:?}", result);
        return HttpResponse::InternalServerError().body(format!("{:?}", result));
    }
    debug!("Synced metrics");
    HttpResponse::Ok().body("ok")
}

#[cfg(test)]
mod tests {
    use crate::utils::test_utils::get_app_state_for_test;

    use super::init;
    use actix_web::{test, App};
    use data_layer::{data_layer::DataLayer, MetricDefinition};

    // Utility functions
    async fn sync_metrics_for_test(data_layer: &DataLayer) {
        let yaml = r#"
kind: Metric
id: metric_id_1
collector: vector
metadata:
  user_key: user_value
enabled: false
---
kind: Metric
id: metric_id_2
collector: vector
metadata:
  user_key: user_value
enabled: true
"#;

        let _ = data_layer.sync_metric_yaml(yaml).await;
    }
    // [GET] /api/metrics (get_all_metrics)

    #[actix_web::test]
    #[tracing_test::traced_test]
    async fn test_get_all_metrics() {
        let app_state = get_app_state_for_test().await;
        sync_metrics_for_test(&app_state.data_layer).await;
        let app = test::init_service(App::new().app_data(app_state).configure(init)).await;
        let req = test::TestRequest::get().uri("/api/metrics").to_request();
        let resp: Vec<MetricDefinition> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.len(), 2);
    }

    // [GET] /api/metrics (get_all_metrics_json)

    #[actix_web::test]
    #[tracing_test::traced_test]
    async fn test_get_all_metrics_json() {
        let app_state = get_app_state_for_test().await;
        sync_metrics_for_test(&app_state.data_layer).await;
        let app = test::init_service(App::new().app_data(app_state).configure(init)).await;
        let req = test::TestRequest::get().uri("/api/metrics").to_request();
        let resp: Vec<serde_json::Value> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.len(), 2);
        for metric in resp.iter() {
            if let Some(created_at) = metric.get("created_at").and_then(|v| v.as_str()) {
                assert!(!created_at.is_empty());
            } else {
                panic!("created_at field is missing or not a string");
            }

            if let Some(updated_at) = metric.get("updated_at").and_then(|v| v.as_str()) {
                assert!(!updated_at.is_empty());
            } else {
                panic!("updated_at field is missing or not a string");
            }
        }
    }

    // [GET] /api/metrics/yaml

    #[actix_web::test]
    #[tracing_test::traced_test]
    async fn test_get_metrics_yaml() {
        let app_state = get_app_state_for_test().await;
        sync_metrics_for_test(&app_state.data_layer).await;
        let app = test::init_service(App::new().app_data(app_state).configure(init)).await;
        let req = test::TestRequest::get()
            .uri("/api/metrics/yaml")
            .to_request();
        let resp: Vec<String> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.len(), 2);
    }
}
