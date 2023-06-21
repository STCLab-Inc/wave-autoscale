use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_yaml::{Deserializer, Mapping, Value};
use std::{fs::File, path::Path};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Config {
    db_url: String,
}

pub fn read_config_file<P>(path: P) -> Mapping
where
    P: AsRef<Path>,
{
    // Read the file of the path
    let file = File::open(path);
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
