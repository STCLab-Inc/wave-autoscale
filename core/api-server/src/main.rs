use api_server::app;

fn main() {
    env_logger::init();
    app::run_server().unwrap();
}
