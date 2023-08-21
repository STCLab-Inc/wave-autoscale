use log::{debug, error};
use serde::Deserialize;
use std::fs::File;

const DEFAULT_CONFIG_PATH: &str = "./wave-config.yaml";
const DEFAULT_DB_URL: &str = "sqlite://./wave.db";
const DEFAULT_WATCH_DEFINITION_DURATION: u64 = 5000;
const DEFAULT_AUTOSCALING_HISTORY_RETENTION: &str = "1d";
const DEFAULT_COLLECTORS_INFO: &str = "./collectors.yaml";
const DEFAULT_API_HOST: &str = "0.0.0.0";
const DEFAULT_API_PORT: u16 = 3024;
const DEFAULT_WEB_UI: bool = false;
const DEFAULT_WEB_UI_HOST: &str = "0.0.0.0";
const DEFAULT_WEB_UI_PORT: u16 = 3025;

fn default_db_url() -> String {
    DEFAULT_DB_URL.to_string()
}
fn default_watch_definition_duration() -> u64 {
    DEFAULT_WATCH_DEFINITION_DURATION
}
fn default_autoscaling_history_retention() -> String {
    DEFAULT_AUTOSCALING_HISTORY_RETENTION.to_string()
}
fn default_collectors_info() -> String {
    DEFAULT_COLLECTORS_INFO.to_string()
}
fn default_api_host() -> String {
    DEFAULT_API_HOST.to_string()
}
fn default_api_port() -> u16 {
    DEFAULT_API_PORT
}
fn default_web_ui() -> bool {
    DEFAULT_WEB_UI
}
fn default_web_ui_host() -> String {
    DEFAULT_WEB_UI_HOST.to_string()
}
fn default_web_ui_port() -> u16 {
    DEFAULT_WEB_UI_PORT
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct WaveConfig {
    // For data-layer
    #[serde(default = "default_db_url")]
    pub db_url: String,
    // For the controller
    // milliseconds
    #[serde(default = "default_watch_definition_duration")]
    pub watch_definition_duration: u64,
    // Autoscaling history retention. You can specify a duration like 1d, 2w, 3m, 4y, etc.
    #[serde(default = "default_autoscaling_history_retention")]
    pub autoscaling_history_retention: String,
    // For metrics
    #[serde(default = "default_collectors_info")]
    pub collectors_info: String,
    // For api-server
    #[serde(default = "default_api_host")]
    pub host: String,
    #[serde(default = "default_api_port")]
    pub port: u16,
    // For web-app
    #[serde(default = "default_web_ui")]
    pub web_ui: bool,
    #[serde(default = "default_web_ui_host")]
    pub web_ui_host: String,
    #[serde(default = "default_web_ui_port")]
    pub web_ui_port: u16,
}

impl Default for WaveConfig {
    fn default() -> Self {
        WaveConfig {
            db_url: DEFAULT_DB_URL.to_string(),
            watch_definition_duration: DEFAULT_WATCH_DEFINITION_DURATION,
            autoscaling_history_retention: DEFAULT_AUTOSCALING_HISTORY_RETENTION.to_string(),
            collectors_info: DEFAULT_COLLECTORS_INFO.to_string(),
            host: DEFAULT_API_HOST.to_string(),
            port: DEFAULT_API_PORT,
            web_ui: DEFAULT_WEB_UI,
            web_ui_host: DEFAULT_WEB_UI_HOST.to_string(),
            web_ui_port: DEFAULT_WEB_UI_PORT,
        }
    }
}

impl WaveConfig {
    pub fn new(config_path: &str) -> Self {
        let config_path = if config_path.is_empty() {
            DEFAULT_CONFIG_PATH
        } else {
            config_path
        };

        debug!("Reading config file: {}", config_path);
        // Read the file of the path
        let file = File::open(config_path);
        if file.is_err() {
            error!("Error reading config file: {}", file.err().unwrap());
            return WaveConfig::default();
        }
        let file = file.unwrap();
        let wave_config: Result<WaveConfig, serde_yaml::Error> = serde_yaml::from_reader(file);
        if wave_config.is_err() {
            error!("Error parsing config file: {}", wave_config.err().unwrap());
            return WaveConfig::default();
        }
        let wave_config = wave_config.unwrap();
        debug!("Config file parsed: {:?}", wave_config);
        wave_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_wave_config() -> WaveConfig {
        WaveConfig::new("./tests/yaml/wave-config.yaml")
    }

    #[test]
    fn test_watch_definition_duration() {
        let wave_config = get_wave_config();
        assert_eq!(wave_config.db_url, DEFAULT_DB_URL);
        assert_eq!(
            wave_config.watch_definition_duration,
            DEFAULT_WATCH_DEFINITION_DURATION
        );
        assert_eq!(
            wave_config.autoscaling_history_retention,
            DEFAULT_AUTOSCALING_HISTORY_RETENTION
        );
        assert_eq!(wave_config.collectors_info, DEFAULT_COLLECTORS_INFO);
        assert_eq!(wave_config.host, DEFAULT_API_HOST);
        assert_eq!(wave_config.port, DEFAULT_API_PORT);
        assert_eq!(wave_config.web_ui, DEFAULT_WEB_UI);
        assert_eq!(wave_config.web_ui_host, DEFAULT_WEB_UI_HOST);
        assert_eq!(wave_config.web_ui_port, DEFAULT_WEB_UI_PORT);
    }
}
