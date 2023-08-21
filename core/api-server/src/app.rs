use crate::{app_state::get_app_state, controller};
use actix_cors::Cors;
use actix_web::{App, HttpServer};
use data_layer::data_layer::DataLayer;
use log::info;
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use utils::wave_config::WaveConfig;

async fn ping() -> String {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    time.to_string()
}

#[tokio::main]
pub async fn run_api_server(
    wave_config: WaveConfig,
    shared_data_layer: Arc<DataLayer>,
) -> std::io::Result<()> {
    // Read arguments
    let host = wave_config.host;
    let port = wave_config.port;

    // Run HTTP Server
    let app_state = get_app_state(shared_data_layer);

    let http_server = HttpServer::new(move || {
        let cors = Cors::permissive().max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .route("/", actix_web::web::get().to(ping))
            .route("/ping", actix_web::web::get().to(ping))
            .configure(controller::init_metric_controller)
            .configure(controller::init_scaling_component_controller)
            .configure(controller::init_plan_controller)
            .configure(controller::init_autoscaling_history_controller)
            .configure(controller::init_metrics_receiver_controller)
    })
    .bind((host.clone(), port));

    // Server structure implements Future.
    let server = match http_server {
        Ok(server) => server.run(),
        Err(error) => {
            panic!("There was a problem opening the file: {:?}", error)
        }
    };

    info!("It's up! {}:{}", host, port);
    server.await
}
