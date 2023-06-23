use serde::Deserialize;
use serde_yaml::{Deserializer, Mapping, Value};
use std::fs::File;

pub fn read_config_file(config: Option<String>) -> Mapping {
    const DEFAULT_CONFIG_FILE: &str = "./wave-config.yaml";

    let config_file: String;
    if config.is_none() {
        debug!("No config file specified, using default config file: ./config.yaml");
        config_file = DEFAULT_CONFIG_FILE.to_string();
    } else {
        config_file = config.unwrap();
        debug!("Using config file: {:?}", &config_file);
    }

    // Read the file of the path
    let file = File::open(config_file);
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
