use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_yaml::{Deserializer, Mapping, Value};
use std::{fs::File, path::Path};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Config {
    db_url: String,
}

pub fn read_config_file<P>(path: P) -> Result<Mapping>
where
    P: AsRef<Path>,
{
    // Read the file of the path
    let file = File::open(path)?;
    // Make a deserializer to iterate the yaml that could have multiple documents
    let deserializer = Deserializer::from_reader(file);
    // For result
    let result = Value::deserialize(deserializer);
    // Check if the result is ok
    if result.is_err() {
        return Err(anyhow::anyhow!("Error reading config file"));
    }
    // Get the value
    let value = result.unwrap();
    // Check if the value is a mapping
    if !value.is_mapping() {
        return Err(anyhow::anyhow!("Config file is not a mapping"));
    }
    // Get the mapping
    let mapping = value.as_mapping().unwrap();
    // Return the mapping
    Ok(mapping.clone())
}
