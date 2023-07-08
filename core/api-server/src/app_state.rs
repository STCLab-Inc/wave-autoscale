use actix_web::web;
use data_layer::data_layer::DataLayer;
use tokio::sync::Mutex;

pub struct AppState {
    pub counter: Mutex<i32>,
    pub data_layer: DataLayer,
}

pub async fn get_app_state(sql_url: &str) -> web::Data<AppState> {
    web::Data::new(AppState {
        counter: Mutex::new(0),
        data_layer: DataLayer::new(sql_url, "").await,
    })
}
