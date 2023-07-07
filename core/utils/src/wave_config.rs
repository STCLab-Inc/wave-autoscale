use log::error;
use serde::Deserialize;
use std::fs::File;

const DEFAULT_CONFIG_PATH: &str = "./wave-config.yaml";

#[derive(Debug, PartialEq, Deserialize, Default)]
pub struct CommonConfig {
    #[serde(default)]
    pub db_url: String,
}

#[derive(Debug, PartialEq, Deserialize, Default)]
pub struct WaveMetricsConfig {
    #[serde(default)]
    pub output: WaveMetricsOutputConfig,
}

#[derive(Debug, PartialEq, Deserialize, Default)]
pub struct WaveMetricsOutputConfig {
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, PartialEq, Deserialize, Default)]
pub struct WaveApiServerConfig {
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub port: u16,
}

#[derive(Debug, PartialEq, Deserialize, Default)]
pub struct WaveConfig {
    // pub config: Mapping,
    #[serde(default)]
    pub common: CommonConfig,
    #[serde(default)]
    pub wave_metrics: WaveMetricsConfig,
    #[serde(default)]
    pub wave_api_server: WaveApiServerConfig,
}

impl WaveConfig {
    pub fn new(config_path: &str) -> Self {
        let config_path = if config_path.is_empty() {
            DEFAULT_CONFIG_PATH
        } else {
            config_path
        };
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
        wave_config.unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_wave_config() -> WaveConfig {
        WaveConfig::new("./tests/yaml/wave-config.yaml")
    }

    #[test]
    fn test_wave_metrics() {
        let wave_config = get_wave_config();
        assert_eq!(wave_config.common.db_url, "sqlite://./wave.db");
        assert_eq!(
            wave_config.wave_metrics.output.url,
            "http://localhost:8081/api/metrics-receiver"
        );
    }

    #[test]
    fn test_wave_api_server() {
        let wave_config = get_wave_config();
        assert_eq!(wave_config.wave_api_server.host, "localhost");
        assert_eq!(wave_config.wave_api_server.port, 8081);
    }
}
