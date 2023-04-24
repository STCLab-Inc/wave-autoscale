use actix_web::web;
use tokio::sync::Mutex;

pub struct AppState {
    pub counter: Mutex<i32>,
}

pub fn get_app_state() -> web::Data<AppState> {
    web::Data::new(AppState {
        counter: Mutex::new(0),
    })
}
