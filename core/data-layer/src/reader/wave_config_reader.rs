use serde::Deserialize;
use serde_yaml::{Deserializer, Mapping, Value};
use std::fs::File;

const DEFAULT_CONFIG_PATH: &str = "./wave-config.yaml";

pub fn parse_wave_config_file(config_path: &str) -> Mapping {
    let config_path = if config_path.is_empty() {
        DEFAULT_CONFIG_PATH
    } else {
        config_path
    };

    // Read the file of the path
    let file = File::open(config_path);
    if file.is_err() {
        error!("Error reading config file: {}", file.err().unwrap());
        return Mapping::new();
    }
    let file = file.unwrap();
    // Make a deserializer to iterate the yaml that could have multiple documents
    let deserializer = Deserializer::from_reader(file);
    // For result
    let result = Value::deserialize(deserializer);
    match result {
        Ok(Value::Mapping(mapping)) => mapping,
        _ => {
            error!("Error parsing config file");
            Mapping::new()
        }
    }
}
