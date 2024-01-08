use tracing_subscriber;

pub enum LogLevel {
    Verbose,
    Quiet,
    Info,
}

pub fn init(log_level: LogLevel) {
    // install global collector configured based on RUST_LOG env var.
    // tracing_subscriber::fmt::;/
    match log_level {
        LogLevel::Verbose => {
            tracing_subscriber::fmt().with_env_filter("DEBUG").init();
        }
        LogLevel::Quiet => {
            // Do not initialize logger
        }
        LogLevel::Info => {
            tracing_subscriber::fmt()
                .with_env_filter("wave_autoscale=info,api_server=info,utils=info,data_layer=info")
                .init();
        }
    }
}
