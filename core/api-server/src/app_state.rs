use actix_web::web;
use data_layer::data_layer::DataLayer;
use std::sync::Arc;

pub struct AppState {
    pub data_layer: Arc<DataLayer>,
}

pub fn get_app_state(shared_data_layer: Arc<DataLayer>) -> web::Data<AppState> {
    web::Data::new(AppState {
        data_layer: shared_data_layer,
    })
}
