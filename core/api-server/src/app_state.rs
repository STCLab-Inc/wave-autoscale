use actix_web::web;
use data_layer::data_layer::{DataLayer, DataLayerNewParam};
use tokio::sync::Mutex;

pub struct AppState {
    pub counter: Mutex<i32>,
    pub data_layer: DataLayer,
}

pub struct GetAppStateParam {
    pub sql_url: String,
}

pub async fn get_app_state(params: GetAppStateParam) -> web::Data<AppState> {
    web::Data::new(AppState {
        counter: Mutex::new(0),
        data_layer: DataLayer::new(DataLayerNewParam {
            sql_url: params.sql_url,
            watch_duration: 5,
        })
        .await,
    })
}
