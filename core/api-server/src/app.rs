use actix_web::{
    web::{self, Data},
    App, HttpServer,
};
use dotenv::dotenv;
use tokio::sync::Mutex;

use crate::{
    app_state::{get_app_state, AppState},
    controller,
};

#[tokio::main]
pub async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let ip_address = std::env::var("IP_ADDRESS").expect("IP_ADDRESS must be set");
    // get the port from env and parse it to u16
    let port = std::env::var("PORT")
        .expect("PORT must be set")
        .parse::<u16>()
        .expect("PORT must be a number");

    let app_state = get_app_state();

    let http_server = HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .configure(controller::init_metric_controller)
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
