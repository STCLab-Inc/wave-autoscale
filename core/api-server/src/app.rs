use crate::{app_state::get_app_state, args::Args, controller};
use actix_cors::Cors;
use actix_web::{App, HttpServer};
use clap::Parser;
use dotenv::dotenv;
use log::info;
use std::time::{SystemTime, UNIX_EPOCH};
use utils::wave_config::WaveConfig;

async fn ping() -> String {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    time.to_string()
}

#[tokio::main]
pub async fn run_server() -> std::io::Result<()> {
    dotenv().ok();

    // Parse command line arguments
    let args: Args = Args::parse();

    // Read arguments
    let config = args.config.clone().unwrap_or_default();
    let wave_config = WaveConfig::new(config.as_str());
    let host = wave_config.wave_api_server.host;
    let port = wave_config.wave_api_server.port;
    let db_url = wave_config.common.db_url;

    // Run HTTP Server
    let app_state = get_app_state(db_url.as_str()).await;

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
