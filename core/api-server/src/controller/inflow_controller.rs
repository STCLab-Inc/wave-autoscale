use crate::app_state::AppState;
use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;
use tracing::{debug, error};

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_inflow_metric_id).service(get_inflow_logs);
}

// Get inflow metric id
#[get("/api/inflow/metric_id")]
async fn get_inflow_metric_id(app_state: web::Data<AppState>) -> impl Responder {
    let metric_ids = app_state.data_layer.get_inflow_metric_ids().await;
    if metric_ids.is_err() {
        error!("Failed to get metric ids: {:?}", metric_ids);
        return HttpResponse::InternalServerError().body(format!("{:?}", metric_ids));
    }
    let metric_ids = metric_ids.unwrap();
    debug!("Got inflow metric ids: {:?}", metric_ids);
    HttpResponse::Ok().json(metric_ids)
}

#[derive(Debug, Deserialize)]
struct InflowQuery {
    metric_id: String,
    count: usize,
}

// Get inflow with metric id by from and to date
#[get("/api/inflow")]
async fn get_inflow_logs(
    query: web::Query<InflowQuery>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    debug!("Getting inflow with metric id by date: {:?}", query);

    let metric_id = query.metric_id.clone();
    let count = query.count;

    let inflow = app_state
        .data_layer
        .get_inflow_with_metric_id_and_count(metric_id, count)
        .await;

    if inflow.is_err() {
        error!("Failed to get inflow: {:?}", inflow);
        return HttpResponse::InternalServerError().body(format!("{:?}", inflow));
    }
    let inflow = inflow.unwrap();
    debug!("Got inflow: {:?}", inflow);
    HttpResponse::Ok().json(inflow)
}
