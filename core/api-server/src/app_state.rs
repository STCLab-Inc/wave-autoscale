use actix_web::web;
use data_layer::data_layer::DataLayer;
use std::sync::Arc;
use utils::wave_config::WaveConfig;

pub struct AppState {
    pub data_layer: Arc<DataLayer>,
    pub wave_config: WaveConfig,
}

pub fn get_app_state(
    shared_data_layer: Arc<DataLayer>,
    wave_config: WaveConfig,
) -> web::Data<AppState> {
    web::Data::new(AppState {
        data_layer: shared_data_layer,
        wave_config,
    })
}
