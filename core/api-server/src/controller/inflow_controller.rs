use crate::app_state::AppState;
use actix_web::{get, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use tracing::{debug, error};

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_inflow_by_date_in_database)
        .service(get_inflow_metric_id_in_memory)
        .service(get_inflow_by_date_with_metric_id_in_memory);
}

#[derive(Debug, Deserialize)]
struct InflowRequestInMemory {
    metric_id: String,
    from: String,
    to: String,
}

#[derive(Debug, Deserialize)]
struct InflowRequestInDatabase {
    from: String,
    to: String,
}

#[get("/api/inflow/database")]
async fn get_inflow_by_date_in_database(
    query: web::Query<InflowRequestInDatabase>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    debug!("Getting inflow by date: {:?}", query);
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

    let inflow = app_state
        .data_layer
        .get_inflow_by_date_in_database(from_date, to_date)
        .await;
    if inflow.is_err() {
        error!("Failed to get inflow: {:?}", inflow);
        return HttpResponse::InternalServerError().body(format!("{:?}", inflow));
    }
    let inflow = inflow.unwrap();
    debug!("Got inflow: {:?}", inflow);
    HttpResponse::Ok().json(inflow)
}

#[get("/api/inflow/memory/metric_id")]
async fn get_inflow_metric_id_in_memory(app_state: web::Data<AppState>) -> impl Responder {
    let metric_ids = app_state.data_layer.get_inflow_metric_id_in_memory().await;
    if metric_ids.is_err() {
        error!("Failed to get metric ids: {:?}", metric_ids);
        return HttpResponse::InternalServerError().body(format!("{:?}", metric_ids));
    }
    let metric_ids = metric_ids.unwrap();
    debug!("Got inflow metric ids: {:?}", metric_ids);
    HttpResponse::Ok().json(metric_ids)
}

#[get("/api/inflow/memory")]
async fn get_inflow_by_date_with_metric_id_in_memory(
    query: web::Query<InflowRequestInMemory>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    debug!("Getting inflow by date: {:?}", query);
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

    let metric_id = query.metric_id.clone();
    let inflow = app_state
        .data_layer
        .get_inflow_by_date_with_metric_id_in_memory(metric_id, from_date, to_date)
        .await;
    if inflow.is_err() {
        error!("Failed to get inflow: {:?}", inflow);
        return HttpResponse::InternalServerError().body(format!("{:?}", inflow));
    }
    let inflow = inflow.unwrap();
    debug!("Got inflow: {:?}", inflow);
    HttpResponse::Ok().json(inflow)
}
