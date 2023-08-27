use std::sync::Arc;

use actix_web::web;
use data_layer::data_layer::DataLayer;
use tokio::sync::Mutex;

pub struct AppState {
    pub counter: Mutex<i32>,
    pub data_layer: Arc<DataLayer>,
}

pub fn get_app_state(shared_data_layer: Arc<DataLayer>) -> web::Data<AppState> {
    web::Data::new(AppState {
        counter: Mutex::new(0),
        data_layer: shared_data_layer,
    })
}
