#[cfg(not(debug_assertions))]
use tracing::Level;
use tracing_subscriber;

pub enum LogLevel {
    Verbose,
    Quiet,
    Info,
}

#[cfg(debug_assertions)]
pub fn init(_log_level: LogLevel) {
    tracing_subscriber::fmt::init();
}

#[cfg(not(debug_assertions))]
pub fn init(log_level: LogLevel) {
    // install global collector configured based on RUST_LOG env var.
    // tracing_subscriber::fmt::;/
    match log_level {
        LogLevel::Verbose => {
            tracing_subscriber::fmt()
                .with_env_filter(
                    "wave_autoscale=debug,api_server=debug,utils=debug,data_layer=debug",
                )
                .init();
        }
        LogLevel::Quiet => {
            tracing_subscriber::fmt()
                .with_max_level(Level::ERROR)
                .init();
        }
        LogLevel::Info => {
            tracing_subscriber::fmt()
                .with_env_filter("wave_autoscale=info,api_server=info,utils=info,data_layer=info")
                .init();
        }
    }
}
