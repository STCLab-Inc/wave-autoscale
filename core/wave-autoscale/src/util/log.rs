use tracing_subscriber::{filter, fmt, prelude::*, reload, Registry};

const INFO_FILTER: &str = "wave_autoscale=info,api_server=info,utils=info,data_layer=info";
const DEBUG_FILTER: &str = "wave_autoscale=debug,api_server=debug,utils=debug,data_layer=debug";
const QUIET_FILTER: &str = "none";

pub struct WALog {
    reload_handle: reload::Handle<filter::EnvFilter, Registry>,
}

impl WALog {
    pub fn new() -> Self {
        let (filter, reload_handle) =
            reload::Layer::new(tracing_subscriber::EnvFilter::new(INFO_FILTER));
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::Layer::default())
            .init();
        WALog { reload_handle }
    }
    pub fn set_info(&self) {
        let _ = self
            .reload_handle
            .reload(filter::EnvFilter::new(INFO_FILTER));
    }
    pub fn set_debug(&self) {
        let _ = self
            .reload_handle
            .reload(filter::EnvFilter::new(DEBUG_FILTER));
    }
    pub fn set_quiet(&self) {
        // Turn off the logger including Error
        let _ = self
            .reload_handle
            .reload(filter::EnvFilter::new(QUIET_FILTER));
    }
}

impl Default for WALog {
    fn default() -> Self {
        Self::new()
    }
}
