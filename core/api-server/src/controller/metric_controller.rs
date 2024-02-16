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
    // .service(get_metric_by_id)
    // .service(post_metrics)
    // .service(put_metric_by_id)
    // .service(delete_metric_by_id);
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

// #[get("/api/metrics/{db_id}")]
// async fn get_metric_by_id(
//     db_id: web::Path<String>,
//     app_state: web::Data<AppState>,
// ) -> impl Responder {
//     debug!("Getting metric by id: {}", db_id);
//     let metric = app_state
//         .data_layer
//         .get_metric_by_id(db_id.into_inner())
//         .await;
//     if metric.is_err() {
//         let error_message = format!("{:?}", metric);
//         error!("Failed to get metric: {}", error_message);
//         return HttpResponse::InternalServerError().body(error_message);
//     }
//     let metric = metric.unwrap();
//     if metric.is_none() {
//         error!("Metric not found");
//         return HttpResponse::NotFound().body("Metric not found");
//     }
//     let metric = metric.unwrap();
//     debug!("Got metric: {:?}", metric);
//     HttpResponse::Ok().json(metric)
// }

// // [POST] /metrics
// #[derive(Deserialize, Validate)]
// struct PostMetricsRequest {
//     metrics: Vec<MetricDefinition>,
// }

// #[post("/api/metrics")]
// async fn post_metrics(
//     request: web::Json<PostMetricsRequest>,
//     app_state: web::Data<AppState>,
// ) -> impl Responder {
//     debug!("Adding metrics: {:?}", request.metrics);
//     let result = app_state
//         .data_layer
//         .add_metrics(request.metrics.clone())
//         .await;
//     if result.is_err() {
//         error!("Failed to add metrics: {:?}", result);
//         return HttpResponse::InternalServerError().body(format!("{:?}", result));
//     }
//     debug!("Added metrics");
//     HttpResponse::Ok().body("ok")
// }

// #[put("/api/metrics/{db_id}")]
// async fn put_metric_by_id(
//     db_id: web::Path<String>,
//     request: web::Json<MetricDefinition>,
//     app_state: web::Data<AppState>,
// ) -> impl Responder {
//     let mut metric = request.into_inner();
//     debug!("Updating metric: {:?}", metric);
//     metric.db_id = db_id.into_inner();
//     let result = app_state.data_layer.update_metric(metric).await;
//     if result.is_err() {
//         error!("Failed to update metric: {:?}", result);
//         return HttpResponse::InternalServerError().body(format!("{:?}", result));
//     }
//     debug!("Updated metric");
//     HttpResponse::Ok().body("ok")
// }

// #[delete("/api/metrics/{db_id}")]
// async fn delete_metric_by_id(
//     db_id: web::Path<String>,
//     app_state: web::Data<AppState>,
// ) -> impl Responder {
//     debug!("Deleting metric by id: {}", db_id);
//     let result = app_state.data_layer.delete_metric(db_id.into_inner()).await;
//     if result.is_err() {
//         error!("Failed to delete metric: {:?}", result);
//         return HttpResponse::InternalServerError().body(format!("{:?}", result));
//     }
//     debug!("Deleted metric");
//     HttpResponse::Ok().body("ok")
// }

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

    // // [GET] /api/metrics/{db_id}

    // #[actix_web::test]
    // #[tracing_test::traced_test]
    // async fn test_get_metric_by_id() {
    //     let app_state = get_app_state_for_test().await;
    //     sync_metrics_for_test(&app_state.data_layer).await;

    //     let metrics = get_metrics_for_test(&app_state.data_layer).await;

    //     let app = test::init_service(App::new().app_data(app_state).configure(init)).await;
    //     let req = test::TestRequest::get()
    //         .uri(format!("/api/metrics/{}", metrics[0].db_id).as_str())
    //         .to_request();
    //     let resp: Result<MetricDefinition, Box<dyn Error>> =
    //         test::try_call_and_read_body_json(&app, req).await;
    //     let resp = resp.unwrap();
    //     assert_eq!(resp.id, "metric_id_1");
    // }

    // #[actix_web::test]
    // #[tracing_test::traced_test]
    // async fn test_get_metric_by_id_failed() {
    //     let app_state = get_app_state_for_test().await;
    //     sync_metrics_for_test(&app_state.data_layer).await;

    //     let app = test::init_service(App::new().app_data(app_state).configure(init)).await;
    //     let req = test::TestRequest::get()
    //         .uri(format!("/api/metrics/{}", "random_id_to_fail").as_str())
    //         .to_request();
    //     let resp = test::call_service(&app, req).await;
    //     assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    // }

    // // [POST] /api/metrics
    // #[actix_web::test]
    // #[tracing_test::traced_test]
    // async fn test_post_metrics() {
    //     let app_state = get_app_state_for_test().await;
    //     let app = test::init_service(App::new().app_data(app_state).configure(init)).await;
    //     let req = test::TestRequest::post()
    //         .uri("/api/metrics")
    //         .set_json(json!(
    //             { "metrics": [{
    //                 "id": "metric_id_1",
    //                 "collector": "vector",
    //                 "metadata": {}
    //             },
    //             {
    //                 "id": "metric_id_2",
    //                 "collector": "vector",
    //                 "metadata": {}
    //             }]}
    //         ))
    //         .to_request();
    //     let resp = test::call_service(&app, req).await;
    //     assert_eq!(resp.status(), StatusCode::OK);
    //     // Check if metrics are added
    //     let req = test::TestRequest::get().uri("/api/metrics").to_request();
    //     let resp: Vec<MetricDefinition> = test::call_and_read_body_json(&app, req).await;
    //     assert_eq!(resp.len(), 2);
    // }

    // // [DELETE] /api/metrics/{db_id}
    // #[actix_web::test]
    // #[tracing_test::traced_test]
    // async fn test_delete_metric_by_id() {
    //     let app_state = get_app_state_for_test().await;
    //     sync_metrics_for_test(&app_state.data_layer).await;

    //     let metrics = get_metrics_for_test(&app_state.data_layer).await;

    //     let app = test::init_service(App::new().app_data(app_state).configure(init)).await;
    //     let req = test::TestRequest::delete()
    //         .uri(format!("/api/metrics/{}", metrics[0].db_id).as_str())
    //         .to_request();
    //     let resp = test::call_service(&app, req).await;
    //     assert_eq!(resp.status(), StatusCode::OK);
    //     // Check if metrics are deleted
    //     let req = test::TestRequest::get().uri("/api/metrics").to_request();
    //     let resp: Vec<MetricDefinition> = test::call_and_read_body_json(&app, req).await;
    //     assert_eq!(resp.len(), 1);
    // }

    // // [PUT] /api/metrics/{db_id}
    // #[actix_web::test]
    // #[tracing_test::traced_test]
    // async fn test_put_metric_by_id() {
    //     let app_state = get_app_state_for_test().await;
    //     sync_metrics_for_test(&app_state.data_layer).await;

    //     let metrics = get_metrics_for_test(&app_state.data_layer).await;

    //     let app = test::init_service(App::new().app_data(app_state).configure(init)).await;
    //     let new_metric = json!(
    //         {
    //             "id": "metric_id_3",
    //             "collector": "telegraf",
    //             "metadata": {}
    //         }
    //     );
    //     let req = test::TestRequest::put()
    //         .uri(format!("/api/metrics/{}", metrics[0].db_id).as_str())
    //         .set_json(&new_metric)
    //         .to_request();
    //     let resp = test::call_service(&app, req).await;
    //     assert_eq!(resp.status(), StatusCode::OK);
    //     // Check if metrics are updated
    //     let req = test::TestRequest::get().uri("/api/metrics").to_request();
    //     let resp: Vec<MetricDefinition> = test::call_and_read_body_json(&app, req).await;
    //     assert_eq!(resp.len(), 2);
    //     assert_eq!(resp[0].id, "metric_id_3");
    // }
}
