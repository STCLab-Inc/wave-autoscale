use crate::app_state::AppState;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use data_layer::ScalingPlanDefinition;
use serde::Deserialize;
use validator::Validate;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_plans)
        .service(get_plan_by_id)
        .service(post_plans)
        .service(put_plan_by_id)
        .service(delete_plan_by_id);
}

#[get("/api/plans")]
async fn get_plans(app_state: web::Data<AppState>) -> impl Responder {
    // HttpResponse::Ok().body("Hello world!")
    // const plans = &app
    let plans = app_state.data_layer.get_all_plans().await;
    if plans.is_err() {
        return HttpResponse::InternalServerError().body(format!("{:?}", plans));
    }
    HttpResponse::Ok().json(plans.unwrap())
}

#[get("/api/plans/{db_id}")]
async fn get_plan_by_id(
    db_id: web::Path<String>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let plan = app_state
        .data_layer
        .get_plan_by_id(db_id.into_inner())
        .await;
    if plan.is_err() {
        return HttpResponse::InternalServerError().body(format!("{:?}", plan));
    }
    HttpResponse::Ok().json(plan.unwrap())
}

#[derive(Deserialize, Validate)]
struct PostPlansRequest {
    plans: Vec<ScalingPlanDefinition>,
}

#[post("/api/plans")]
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

#[put("/api/plans/{db_id}")]
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

#[delete("/api/plans/{db_id}")]
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
