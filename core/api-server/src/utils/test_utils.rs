use crate::app_state::{get_app_state, AppState};
use actix_web::web;
use data_layer::data_layer::DataLayer;
use std::sync::Arc;

pub async fn get_app_state_for_test() -> web::Data<AppState> {
    let data_layer = DataLayer::new("sqlite::memory:", 500_000, false).await;
    let shared_data_layer = Arc::new(data_layer);
    let app_state = get_app_state(shared_data_layer);
    app_state.data_layer.sync("").await;
    app_state
}
