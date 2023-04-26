use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use data_layer::ScalingComponentDefinition;
use serde::Deserialize;
use validator::Validate;

use crate::app_state::AppState;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_scaling_components)
        .service(post_scaling_components)
        .service(put_scaling_component_by_id)
        .service(delete_scaling_component_by_id);
}

#[get("/scaling-components")]
async fn get_scaling_components(app_state: web::Data<AppState>) -> impl Responder {
    // HttpResponse::Ok().body("Hello world!")
    // const scaling_components = &app
    let scaling_components = app_state.data_layer.get_all_scaling_components().await;
    if scaling_components.is_err() {
        return HttpResponse::InternalServerError().body(format!("{:?}", scaling_components));
    }
    HttpResponse::Ok().json(scaling_components.unwrap())
}

// [POST] /scaling-components
#[derive(Deserialize, Validate)]
struct PostScalingComponentsRequest {
    scaling_components: Vec<ScalingComponentDefinition>,
}

#[post("/scaling-components")]
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

#[put("/scaling-components/{db_id}")]
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

#[delete("/scaling-components/{db_id}")]
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
