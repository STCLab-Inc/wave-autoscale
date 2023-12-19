use crate::app_state::AppState;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use data_layer::ScalingComponentDefinition;
use serde::Deserialize;
use validator::Validate;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_scaling_components)
        .service(get_scaling_component_by_id)
        .service(post_scaling_components)
        .service(put_scaling_component_by_id)
        .service(delete_scaling_component_by_id);
}

#[get("/api/scaling-components")]
async fn get_scaling_components(app_state: web::Data<AppState>) -> impl Responder {
    // HttpResponse::Ok().body("Hello world!")
    // const scaling_components = &app
    let scaling_components = app_state.data_layer.get_all_scaling_components_json().await;
    if scaling_components.is_err() {
        return HttpResponse::InternalServerError().body(format!("{:?}", scaling_components));
    }
    HttpResponse::Ok().json(scaling_components.unwrap())
}

#[get("/api/scaling-components/{db_id}")]
async fn get_scaling_component_by_id(
    db_id: web::Path<String>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let scaling_component = app_state
        .data_layer
        .get_scaling_component_by_id(db_id.into_inner())
        .await;
    if scaling_component.is_err() {
        return HttpResponse::InternalServerError().body(format!("{:?}", scaling_component));
    }
    HttpResponse::Ok().json(scaling_component.unwrap())
}

// [POST] /scaling-components
#[derive(Deserialize, Validate)]
struct PostScalingComponentsRequest {
    scaling_components: Vec<ScalingComponentDefinition>,
}

#[post("/api/scaling-components")]
async fn post_scaling_components(
    request: web::Json<PostScalingComponentsRequest>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let result = app_state
        .data_layer
        .add_scaling_components(request.scaling_components.clone())
        .await;
    if result.is_err() {
        return HttpResponse::InternalServerError().body(format!("{:?}", result));
    }
    HttpResponse::Ok().body("ok")
}

#[put("/api/scaling-components/{db_id}")]
async fn put_scaling_component_by_id(
    db_id: web::Path<String>,
    request: web::Json<ScalingComponentDefinition>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let mut scaling_component = request.into_inner();
    scaling_component.db_id = db_id.into_inner();

    let result = app_state
        .data_layer
        .update_scaling_component(scaling_component)
        .await;
    if result.is_err() {
        return HttpResponse::InternalServerError().body(format!("{:?}", result));
    }
    HttpResponse::Ok().body("ok")
}

#[delete("/api/scaling-components/{db_id}")]
async fn delete_scaling_component_by_id(
    db_id: web::Path<String>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let result = app_state
        .data_layer
        .delete_scaling_component(db_id.into_inner())
        .await;
    if result.is_err() {
        return HttpResponse::InternalServerError().body(format!("{:?}", result));
    }
    HttpResponse::Ok().body("ok")
}

#[cfg(test)]
mod tests {
    use crate::utils::test_utils::get_app_state_for_test;

    use super::init;
    use actix_web::{test, App};
    use data_layer::{data_layer::DataLayer, types::object_kind::ObjectKind};
    use std::collections::HashMap;

    // Utility functions

    async fn add_scaling_components_for_test(data_layer: &DataLayer) {
        let _ = data_layer
            .add_scaling_components(vec![
                data_layer::ScalingComponentDefinition {
                    id: "test1".to_string(),
                    db_id: "test1".to_string(),
                    component_kind: "test1".to_string(),
                    kind: ObjectKind::ScalingComponent,
                    metadata: HashMap::new(),
                    enabled: true,
                },
                data_layer::ScalingComponentDefinition {
                    id: "test2".to_string(),
                    db_id: "test2".to_string(),
                    component_kind: "test2".to_string(),
                    kind: ObjectKind::ScalingComponent,
                    metadata: HashMap::new(),
                    enabled: true,
                },
            ])
            .await;
    }

    // [GET] /api/scaling-components (get_all_scaling_components_json)

    #[actix_web::test]
    #[tracing_test::traced_test]
    async fn test_get_all_scaling_components_json() {
        let app_state = get_app_state_for_test().await;
        add_scaling_components_for_test(&app_state.data_layer).await;
        let app = test::init_service(App::new().app_data(app_state).configure(init)).await;
        let req = test::TestRequest::get()
            .uri("/api/scaling-components")
            .to_request();
        let resp: Vec<serde_json::Value> = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.len(), 2);
        for component in resp.iter() {
            if let Some(created_at) = component.get("created_at").and_then(|v| v.as_str()) {
                assert!(!created_at.is_empty());
            } else {
                panic!("created_at field is missing or not a string");
            }

            if let Some(updated_at) = component.get("updated_at").and_then(|v| v.as_str()) {
                assert!(!updated_at.is_empty());
            } else {
                panic!("updated_at field is missing or not a string");
            }
        }
    }
}
