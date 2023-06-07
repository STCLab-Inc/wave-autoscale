use std::time::{SystemTime, UNIX_EPOCH};

use actix_cors::Cors;
use actix_web::{App, HttpServer};
use dotenv::dotenv;

use crate::{
    app_state::{get_app_state, GetAppStateParam},
    controller,
};

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

    // get the ip_address from env
    let ip_address = std::env::var("IP_ADDRESS").expect("IP_ADDRESS must be set");

    // get the port from env and parse it to u16
    let port = std::env::var("PORT")
        .expect("PORT must be set")
        .parse::<u16>()
        .expect("PORT must be a number");

    // get the sql_url from env
    let sql_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let app_state = get_app_state(GetAppStateParam { sql_url }).await;

    let http_server = HttpServer::new(move || {
        let cors = Cors::permissive().max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .route("/ping", actix_web::web::get().to(ping))
            .configure(controller::init_metric_controller)
            .configure(controller::init_scaling_component_controller)
            .configure(controller::init_plan_controller)
            .configure(controller::init_autoscaling_history_controller)
    })
    .bind((ip_address.clone(), port));

    // Server structure implements Future.
    let server = match http_server {
        Ok(server) => server.run(),
        Err(error) => {
            panic!("There was a problem opening the file: {:?}", error)
        }
    };

    println!("It's up! {}:{}", ip_address, port);
    server.await
}
