use crate::{
    app_state::{get_app_state, AppState},
    controller,
};
use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use data_layer::data_layer::DataLayer;
use serde_json::json;
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tracing::info;
use utils::wave_config::WaveConfig;

async fn ping() -> String {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    time.to_string()
}

async fn get_info(app_state: web::Data<AppState>) -> impl Responder {
    let version = env!("CARGO_PKG_VERSION");
    let wave_config = &app_state.wave_config;
    let wave_config_yaml = serde_yaml::to_string(wave_config).unwrap();
    let json = json!({ "version": version, "wave_config": wave_config_yaml});
    HttpResponse::Ok().json(json)
}

#[tokio::main]
pub async fn run_api_server(
    wave_config: WaveConfig,
    shared_data_layer: Arc<DataLayer>,
) -> std::io::Result<()> {
    // Read arguments
    let host = wave_config.host.clone();
    let port = wave_config.port;

    // Run HTTP Server
    let app_state = get_app_state(shared_data_layer, wave_config);
    let http_server = HttpServer::new(move || {
        let cors = Cors::permissive().max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .route("/", actix_web::web::get().to(get_info))
            .route("/ping", actix_web::web::get().to(ping))
            .route("/api/info", actix_web::web::get().to(get_info))
            .configure(controller::init_metric_controller)
            .configure(controller::init_scaling_component_controller)
            .configure(controller::init_plan_controller)
            .configure(controller::init_plan_logs_controller)
            .configure(controller::init_metrics_receiver_controller)
            .configure(controller::init_definition_controller)
    })
    .workers(1)
    .bind((host.clone(), port));

    if http_server.is_err() {
        panic!(
            "There was a problem opening the file: {:?}",
            http_server.err().unwrap()
        )
    }
    let http_server = http_server.unwrap();
    info!("[api-server] running on {}:{}", host, port);
    let _ = http_server.run().await;

    Ok(())
}
