use crate::app_state::AppState;
use actix_web::{get, post, web, HttpResponse, Responder};
use chrono::{TimeZone, Utc};
use serde::Deserialize;
use serde_json::json;
use tracing::{debug, error};

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_plan_logs_by_date)
        .service(generate_plan_logs_samples);
}

#[derive(Debug, Deserialize)]
struct GetPlanLogsByDateRequest {
    plan_id: Option<String>,
    // Unix timestamp in milliseconds
    from: u64,
    // Unix timestamp in milliseconds
    to: u64,
}

#[get("/api/plan-logs")]
async fn get_plan_logs_by_date(
    query: web::Query<GetPlanLogsByDateRequest>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    debug!("Getting plan logs by date: {:?}", query);
    let plan_id = query.plan_id.clone();
    let from_date = Utc.timestamp_millis_opt(query.from as i64).single();
    let to_date = Utc.timestamp_millis_opt(query.to as i64).single();
    if from_date.is_none() || to_date.is_none() {
        error!("Invalid date format");
        return HttpResponse::BadRequest().body("Invalid date format");
    }
    let from_date = from_date.unwrap().with_timezone(&Utc);
    let to_date = to_date.unwrap().with_timezone(&Utc);

    let plan_logs = app_state
        .data_layer
        .get_plan_logs_by_date(plan_id, from_date, to_date)
        .await;
    if plan_logs.is_err() {
        error!("Failed to get plan logs: {:?}", plan_logs);
        return HttpResponse::InternalServerError().body(format!("{:?}", plan_logs));
    }
    let plan_logs = plan_logs.unwrap();
    debug!("Got plan logs: {:?}", plan_logs);
    HttpResponse::Ok().json(plan_logs)
}

#[post("/api/plan-logs/generate-samples")]
async fn generate_plan_logs_samples(app_state: web::Data<AppState>) -> impl Responder {
    debug!("Generating plan logs samples");
    let plan_logs = app_state.data_layer.generate_plan_log_samples(5).await;
    if plan_logs.is_err() {
        error!("Failed to generate plan logs samples: {:?}", plan_logs);
        return HttpResponse::InternalServerError().body(format!("{:?}", plan_logs));
    }
    HttpResponse::Ok().json(json!({
        "message": "Plan logs samples generated"
    }))
}
