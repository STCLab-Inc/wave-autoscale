use crate::app_state::AppState;
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Deserialize;
use tracing::debug;
use validator::Validate;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_scaling_components)
        .service(get_scaling_component_yaml)
        .service(post_scaling_component_yaml);
    // .service(get_scaling_component_by_id)
    // .service(post_scaling_components)
    // .service(put_scaling_component_by_id)
    // .service(delete_scaling_component_by_id);
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

#[get("/api/scaling-components/yaml")]
async fn get_scaling_component_yaml(app_state: web::Data<AppState>) -> impl Responder {
    let scaling_components = app_state.data_layer.get_scaling_component_yamls().await;
    if scaling_components.is_err() {
        return HttpResponse::InternalServerError().body(format!("{:?}", scaling_components));
    }
    HttpResponse::Ok().json(scaling_components.unwrap())
}

// [POST] /api/scaling-components/yaml
#[derive(Deserialize, Validate)]
struct PostScalingComponentYamlRequest {
    yaml: String,
}

#[post("/api/scaling-components/yaml")]
async fn post_scaling_component_yaml(
    request: web::Json<PostScalingComponentYamlRequest>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    debug!("Adding scaling components from yaml: {:?}", request.yaml);
    let yaml = request.yaml.as_str();
    let result = app_state.data_layer.sync_scaling_component_yaml(yaml).await;
    if result.is_err() {
        return HttpResponse::InternalServerError().body(format!("{:?}", result));
    }
    HttpResponse::Ok().body("ok")
}

// #[get("/api/scaling-components/{db_id}")]
// async fn get_scaling_component_by_id(
//     db_id: web::Path<String>,
//     app_state: web::Data<AppState>,
// ) -> impl Responder {
//     let scaling_component = app_state
//         .data_layer
//         .get_scaling_component_by_id(db_id.into_inner())
//         .await;
//     if scaling_component.is_err() {
//         return HttpResponse::InternalServerError().body(format!("{:?}", scaling_component));
//     }
//     HttpResponse::Ok().json(scaling_component.unwrap())
// }

// // [POST] /scaling-components
// #[derive(Deserialize, Validate)]
// struct PostScalingComponentsRequest {
//     scaling_components: Vec<ScalingComponentDefinition>,
// }

// #[post("/api/scaling-components")]
// async fn post_scaling_components(
//     request: web::Json<PostScalingComponentsRequest>,
//     app_state: web::Data<AppState>,
// ) -> impl Responder {
//     let result = app_state
//         .data_layer
//         .add_scaling_components(request.scaling_components.clone())
//         .await;
//     if result.is_err() {
//         return HttpResponse::InternalServerError().body(format!("{:?}", result));
//     }
//     HttpResponse::Ok().body("ok")
// }

// #[put("/api/scaling-components/{db_id}")]
// async fn put_scaling_component_by_id(
//     db_id: web::Path<String>,
//     request: web::Json<ScalingComponentDefinition>,
//     app_state: web::Data<AppState>,
// ) -> impl Responder {
//     let mut scaling_component = request.into_inner();
//     scaling_component.db_id = db_id.into_inner();

//     let result = app_state
//         .data_layer
//         .update_scaling_component(scaling_component)
//         .await;
//     if result.is_err() {
//         return HttpResponse::InternalServerError().body(format!("{:?}", result));
//     }
//     HttpResponse::Ok().body("ok")
// }

// #[delete("/api/scaling-components/{db_id}")]
// async fn delete_scaling_component_by_id(
//     db_id: web::Path<String>,
//     app_state: web::Data<AppState>,
// ) -> impl Responder {
//     let result = app_state
//         .data_layer
//         .delete_scaling_component(db_id.into_inner())
//         .await;
//     if result.is_err() {
//         return HttpResponse::InternalServerError().body(format!("{:?}", result));
//     }
//     HttpResponse::Ok().body("ok")
// }

#[cfg(test)]
mod tests {
    use crate::utils::test_utils::get_app_state_for_test;

    use super::init;
    use actix_web::{test, App};
    use data_layer::{data_layer::DataLayer};
    

    // Utility functions
    async fn sync_scaling_components_for_test(data_layer: &DataLayer) {
        let yaml = r#"
kind: ScalingComponent
id: test_component_1
metadata:
  name: Test Component
enabled: true
---
kind: ScalingComponent
id: test_component_2
metadata:
  name: Test Component
enabled: true
"#;
        data_layer.sync_scaling_component_yaml(yaml).await.unwrap();
    }

    // [GET] /api/scaling-components (get_all_scaling_components_json)

    #[actix_web::test]
    #[tracing_test::traced_test]
    async fn test_get_all_scaling_components_json() {
        let app_state = get_app_state_for_test().await;
        sync_scaling_components_for_test(&app_state.data_layer).await;
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
