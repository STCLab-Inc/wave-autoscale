use crate::{app_state::get_app_state, args::Args, controller, tcp_server::run_tcp_server};
use actix_cors::Cors;
use actix_web::{App, HttpServer};
use clap::Parser;
use data_layer::reader::wave_config_reader::parse_wave_config_file;
use dotenv::dotenv;
use log::info;
use std::time::{SystemTime, UNIX_EPOCH};

async fn ping() -> String {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    time.to_string()
}

fn get_config_values(config: &str) -> (String, u16, String, u16, String) {
    // Read config file
    let config_result = parse_wave_config_file(config);

    //  get the host and the port from env
    let host = config_result
        .get("WAVE-API-SERVER")
        .and_then(|common| common.get("HOST"))
        .and_then(|db_url| db_url.as_str())
        .unwrap_or_default()
        .to_string();

    let port = config_result
        .get("WAVE-API-SERVER")
        .and_then(|common| common.get("PORT"))
        .and_then(|db_url| db_url.as_u64())
        .unwrap_or_default() as u16;

    //  get the host and the port from env
    let tcp_host = config_result
        .get("WAVE-API-SERVER")
        .and_then(|common| common.get("TCP_HOST"))
        .and_then(|db_url| db_url.as_str())
        .unwrap_or_default()
        .to_string();

    let tcp_port = config_result
        .get("WAVE-API-SERVER")
        .and_then(|common| common.get("TCP_PORT"))
        .and_then(|db_url| db_url.as_u64())
        .unwrap_or_default() as u16;

    // get the sql_url from env
    let db_url = config_result
        .get("COMMON")
        .and_then(|common| common.get("DB_URL"))
        .and_then(|db_url| db_url.as_str())
        .unwrap_or_default()
        .to_string();

    (host, port, tcp_host, tcp_port, db_url)
}

#[tokio::main]
pub async fn run_server() -> std::io::Result<()> {
    dotenv().ok();

    // Parse command line arguments
    let args: Args = Args::parse();

    // Read arguments
    let config = args.config.clone().unwrap_or_default();
    let (host, port, tcp_host, tcp_port, db_url) = get_config_values(&config);

    // Run TCP Server
    run_tcp_server(tcp_host.as_str(), tcp_port).await;

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
