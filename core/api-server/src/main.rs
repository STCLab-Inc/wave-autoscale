mod app;
mod app_state;
mod args;
mod controller;

fn main() {
    env_logger::init();
    app::run_server().unwrap();
}
