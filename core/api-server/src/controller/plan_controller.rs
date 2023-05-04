use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use data_layer::ScalingPlanDefinition;
use serde::Deserialize;
use validator::Validate;

use crate::app_state::AppState;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_plans)
        .service(post_plans)
        .service(put_plan_by_id)
        .service(delete_plan_by_id);
}

#[get("/plans")]
async fn get_plans(app_state: web::Data<AppState>) -> impl Responder {
    // HttpResponse::Ok().body("Hello world!")
    // const plans = &app
    let plans = app_state.data_layer.get_all_plans().await;
    if plans.is_err() {
        return HttpResponse::InternalServerError().body(format!("{:?}", plans));
    }
    HttpResponse::Ok().json(plans.unwrap())
}

#[derive(Deserialize, Validate)]
struct PostPlansRequest {
    plans: Vec<ScalingPlanDefinition>,
}

#[post("/plans")]
async fn post_plans(
    request: web::Json<PostPlansRequest>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let result = app_state.data_layer.add_plans(request.plans.clone()).await;
    if result.is_err() {
        return HttpResponse::InternalServerError().body(format!("{:?}", result));
    }
    HttpResponse::Ok().body("ok")
}

#[put("/plans/{db_id}")]
async fn put_plan_by_id(
    db_id: web::Path<String>,
    request: web::Json<ScalingPlanDefinition>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let mut plan = request.into_inner();
    plan.db_id = db_id.into_inner();

    let result = app_state.data_layer.update_plan(plan).await;
    if result.is_err() {
        return HttpResponse::InternalServerError().body(format!("{:?}", result));
    }
    HttpResponse::Ok().body("ok")
}

#[delete("/plans/{db_id}")]
async fn delete_plan_by_id(
    db_id: web::Path<String>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let result = app_state.data_layer.delete_plan(db_id.into_inner()).await;
    if result.is_err() {
        return HttpResponse::InternalServerError().body(format!("{:?}", result));
    }
    HttpResponse::Ok().body("ok")
}