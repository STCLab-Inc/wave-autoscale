use crate::config_path::find_file_in_wa;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File};
use tracing::{debug, error};

const CONFIG_FILE_NAME: &str = "wave-config.yaml";
// Default values
const DEFAULT_DEBUG: bool = false;
const DEFAULT_QUIET: bool = false;
const DEFAULT_DB_URL: &str = "sqlite://./wave.db";
const DEFAULT_METRIC_BUFFER_SIZE_KB: u64 = 500_000;
const DEFAULT_ENABLE_METRICS_LOG: bool = false;
const DEFAULT_WATCH_DEFINITION_DURATION: u64 = 5000;
const DEFAULT_PLAN_LOGS_RETENTION: &str = "14d";
const DEFAULT_API_HOST: &str = "0.0.0.0";
const DEFAULT_API_PORT: u16 = 3024;
const DEFAULT_WEB_UI: bool = true;
const DEFAULT_WEB_UI_HOST: &str = "0.0.0.0";
const DEFAULT_WEB_UI_PORT: u16 = 3025;
const DEFAULT_RESET_DEFINITIONS_ON_STARTUP: bool = false;
const DEFAULT_WEBHOOKS: Option<Vec<Webhooks>> = None;
const DEFAULT_WEBHOOKS_URL: Option<String> = None;
const DEFAULT_WEBHOOKS_HEADERS: Option<HashMap<String, String>> = None;

fn default_debug() -> bool {
    DEFAULT_DEBUG
}
fn default_quiet() -> bool {
    DEFAULT_QUIET
}
fn default_db_url() -> String {
    DEFAULT_DB_URL.to_string()
}
fn default_metric_buffer_size_kb() -> u64 {
    DEFAULT_METRIC_BUFFER_SIZE_KB
}
fn default_enable_metrics_log() -> bool {
    DEFAULT_ENABLE_METRICS_LOG
}
fn default_watch_definition_duration() -> u64 {
    DEFAULT_WATCH_DEFINITION_DURATION
}
fn default_plan_logs_retention() -> String {
    DEFAULT_PLAN_LOGS_RETENTION.to_string()
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
fn default_reset_definitions_on_startup() -> bool {
    DEFAULT_RESET_DEFINITIONS_ON_STARTUP
}
fn default_webhooks() -> Option<Vec<Webhooks>> {
    DEFAULT_WEBHOOKS
}
fn default_webhooks_url() -> Option<String> {
    DEFAULT_WEBHOOKS_URL
}
fn default_webhooks_headers() -> Option<HashMap<String, String>> {
    DEFAULT_WEBHOOKS_HEADERS
}

#[derive(Debug, PartialEq, Deserialize, Default, Clone, Serialize)]
struct DownloadUrlDefinition {
    macos_x86_64: String,
    macos_aarch64: String,
    linux_x86_64: String,
    linux_aarch64: String,
    windows_x86_64: String,
}

#[derive(PartialEq, Clone, Deserialize, Debug, Serialize)]
pub struct Webhooks {
    pub id: String,
    pub webhook_type: WebhookType,
    #[serde(default = "default_webhooks_url")]
    pub url: Option<String>,
    #[serde(default = "default_webhooks_headers")]
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, PartialEq, Deserialize, Clone, Serialize)]
pub enum WebhookType {
    #[serde(alias = "Http", alias = "http")]
    Http,
    #[serde(alias = "SlackIncomingWebhook", alias = "slackincomingwebhook")]
    SlackIncomingWebhook,
    // SlackOauth, // TODO: To be developed.
}

#[derive(Debug, PartialEq, Deserialize, Clone, Serialize)]
pub struct WaveConfig {
    // Wave Autoscale
    // Verbose mode, Overridden by 'quiet'
    #[serde(default = "default_debug")]
    pub debug: bool,
    // Quiet mode, Overrides 'debug'
    #[serde(default = "default_quiet")]
    pub quiet: bool,

    //
    // Data Layer
    //
    #[serde(default = "default_db_url")]
    pub db_url: String,
    // milliseconds
    #[serde(default = "default_watch_definition_duration")]
    pub watch_definition_duration: u64,
    // Plan logs retention. You can specify a duration like 1d, 2w, 3m, 4y, etc.
    #[serde(default = "default_plan_logs_retention")]
    pub plan_logs_retention: String,
    // Reset definitions on startup
    #[serde(default = "default_reset_definitions_on_startup")]
    pub reset_definitions_on_startup: bool,

    //
    // Metrics
    //
    #[serde(default = "default_metric_buffer_size_kb")]
    pub metric_buffer_size_kb: u64,
    // Store metrics in the database
    #[serde(default = "default_enable_metrics_log")]
    pub enable_metrics_log: bool,

    //
    // API Server
    //
    #[serde(default = "default_api_host")]
    pub host: String,
    #[serde(default = "default_api_port")]
    pub port: u16,

    //
    // Wave Autoscale UI
    //
    #[serde(default = "default_web_ui")]
    pub web_ui: bool,
    #[serde(default = "default_web_ui_host")]
    pub web_ui_host: String,
    #[serde(default = "default_web_ui_port")]
    pub web_ui_port: u16,

    //
    // Metrics Collector
    //
    #[serde(default)]
    vector: DownloadUrlDefinition,
    #[serde(default)]
    telegraf: DownloadUrlDefinition,

    //
    // Web hooks
    //
    #[serde(default = "default_webhooks")]
    pub webhooks: Option<Vec<Webhooks>>,
}

impl Default for WaveConfig {
    fn default() -> Self {
        WaveConfig {
            debug: DEFAULT_DEBUG,
            quiet: DEFAULT_QUIET,
            db_url: DEFAULT_DB_URL.to_string(),
            metric_buffer_size_kb: DEFAULT_METRIC_BUFFER_SIZE_KB,
            enable_metrics_log: DEFAULT_ENABLE_METRICS_LOG,
            watch_definition_duration: DEFAULT_WATCH_DEFINITION_DURATION,
            plan_logs_retention: DEFAULT_PLAN_LOGS_RETENTION.to_string(),
            reset_definitions_on_startup: DEFAULT_RESET_DEFINITIONS_ON_STARTUP,
            host: DEFAULT_API_HOST.to_string(),
            port: DEFAULT_API_PORT,
            web_ui: DEFAULT_WEB_UI,
            web_ui_host: DEFAULT_WEB_UI_HOST.to_string(),
            web_ui_port: DEFAULT_WEB_UI_PORT,
            vector: DownloadUrlDefinition::default(),
            telegraf: DownloadUrlDefinition::default(),
            webhooks: DEFAULT_WEBHOOKS,
        }
    }
}

impl WaveConfig {
    pub fn new() -> Self {
        let Ok(config_path) = find_file_in_wa(CONFIG_FILE_NAME) else {
            error!("Error finding config file. It returns default config.");
            return WaveConfig::default();
        };
        // Read the file of the path
        let file = File::open(&config_path);
        if file.is_err() {
            error!(
                "Error reading config file: {}. It returns default config.",
                file.err().unwrap()
            );
            return WaveConfig::default();
        }
        let file = file.unwrap();
        let wave_config: Result<WaveConfig, serde_yaml::Error> = serde_yaml::from_reader(file);
        if wave_config.is_err() {
            error!(
                "Error parsing config file: {}. It returns default config.",
                wave_config.err().unwrap()
            );
            return WaveConfig::default();
        }

        let wave_config = wave_config.unwrap();
        if wave_config.debug && !wave_config.quiet {
            debug!("[wave-config] config_path: {:?}", &config_path);
        }
        wave_config
    }
    pub fn get_download_url(&self, name: &str) -> &str {
        match name {
            "vector_macos_x64_64" => self.vector.macos_x86_64.as_str(),
            "vector_macos_aarch64" => self.vector.macos_aarch64.as_str(),
            "vector_linux_x86_64" => self.vector.linux_x86_64.as_str(),
            "vector_linux_aarch64" => self.vector.linux_aarch64.as_str(),
            "vector_windows_x86_64" => self.vector.windows_x86_64.as_str(),
            "telegraf_macos_x64_64" => self.telegraf.macos_x86_64.as_str(),
            "telegraf_macos_aarch64" => self.telegraf.macos_aarch64.as_str(),
            "telegraf_linux_x86_64" => self.telegraf.linux_x86_64.as_str(),
            "telegraf_linux_aarch64" => self.telegraf.linux_aarch64.as_str(),
            "telegraf_windows_x86_64" => self.telegraf.windows_x86_64.as_str(),
            _ => "",
        }
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use super::*;

    fn get_wave_config() -> WaveConfig {
        WaveConfig::new()
    }

    #[test]
    #[traced_test]
    fn test_watch_definition_duration() {
        let wave_config = get_wave_config();
        assert_eq!(wave_config.db_url, DEFAULT_DB_URL);
        assert_eq!(
            wave_config.watch_definition_duration,
            DEFAULT_WATCH_DEFINITION_DURATION
        );
        assert_eq!(wave_config.plan_logs_retention, DEFAULT_PLAN_LOGS_RETENTION);
        assert_eq!(wave_config.host, DEFAULT_API_HOST);
        assert_eq!(wave_config.port, DEFAULT_API_PORT);
        assert_eq!(wave_config.web_ui, DEFAULT_WEB_UI);
        assert_eq!(wave_config.web_ui_host, DEFAULT_WEB_UI_HOST);
        assert_eq!(wave_config.web_ui_port, DEFAULT_WEB_UI_PORT);
        assert_eq!(wave_config.webhooks, DEFAULT_WEBHOOKS);
    }
}
