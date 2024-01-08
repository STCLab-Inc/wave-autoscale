use crate::app_state::AppState;
use actix_web::{get, post, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::json;
use tracing::{debug, error};

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_autoscaling_history_by_date)
        .service(generate_autoscaling_history_samples);
}

#[derive(Debug, Deserialize)]
struct AutoscalingHistoryRequest {
    from: String,
    to: String,
}

#[get("/api/autoscaling-history")]
async fn get_autoscaling_history_by_date(
    query: web::Query<AutoscalingHistoryRequest>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    debug!("Getting autoscaling history by date: {:?}", query);
    let from_date = query.from.clone();
    let to_date = query.to.clone();
    let from_date = DateTime::parse_from_rfc3339(from_date.as_str());
    let to_date = DateTime::parse_from_rfc3339(to_date.as_str());
    if from_date.is_err() || to_date.is_err() {
        error!("Invalid date format");
        return HttpResponse::BadRequest().body("Invalid date format");
    }
    let from_date = from_date.unwrap().with_timezone(&Utc);
    let to_date = to_date.unwrap().with_timezone(&Utc);

    let autoscaling_history = app_state
        .data_layer
        .get_autoscaling_history_by_date(from_date, to_date)
        .await;
    if autoscaling_history.is_err() {
        error!(
            "Failed to get autoscaling history: {:?}",
            autoscaling_history
        );
        return HttpResponse::InternalServerError().body(format!("{:?}", autoscaling_history));
    }
    let autoscaling_history = autoscaling_history.unwrap();
    debug!("Got autoscaling history: {:?}", autoscaling_history);
    HttpResponse::Ok().json(autoscaling_history)
}

#[post("/api/autoscaling-history/generate-samples")]
async fn generate_autoscaling_history_samples(app_state: web::Data<AppState>) -> impl Responder {
    debug!("Generating autoscaling history samples");
    let autoscaling_history = app_state
        .data_layer
        .generate_autoscaling_history_samples(5)
        .await;
    if autoscaling_history.is_err() {
        error!(
            "Failed to generate autoscaling history samples: {:?}",
            autoscaling_history
        );
        return HttpResponse::InternalServerError().body(format!("{:?}", autoscaling_history));
    }
    HttpResponse::Ok().json(json!({
        "message": "Autoscaling history samples generated"
    }))
}
