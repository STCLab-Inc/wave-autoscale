use crate::{MetricDefinition, ScalingComponentDefinition, ScalingPlanDefinition, SloDefinition};
use anyhow::Result;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};

use serde_valid::Validate;
use serde_yaml::Deserializer;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{ErrorKind, Read},
    path::Path,
};

#[derive(Debug, Default)]
pub struct ParserResult {
    pub metric_definitions: Vec<MetricDefinition>,
    pub slo_definitions: Vec<SloDefinition>,
    pub scaling_plan_definitions: Vec<ScalingPlanDefinition>,
    pub scaling_component_definitions: Vec<ScalingComponentDefinition>,
}

// Function to parse a json file and return its contents as a hashmap string
fn parse_json_file(file_path: &str) -> Option<String> {
    let file_string = fs::read_to_string(file_path).ok()?;
    let file_json: serde_json::Value = serde_json::from_str(&file_string).ok()?;
    if let Some(file_object) = file_json.as_object() {
        let mut file_hashmap = HashMap::new();
        for (key, value) in file_object {
            file_hashmap.insert(key.clone(), value.clone().to_string());
        }
        Some(serde_json::to_string(&file_hashmap).unwrap())
    } else {
        None
    }
}
// Function to parse a yaml file and return its contents as a hashmap string
fn parse_yaml_file(file_path: &str) -> Option<String> {
    let file_string = fs::read_to_string(file_path).ok()?;
    let file_yaml: serde_yaml::Value = serde_yaml::from_str(&file_string).ok()?;
    if let Some(file_map) = file_yaml.as_mapping() {
        let mut file_hashmap = HashMap::new();
        for (key, value) in file_map.iter() {
            if let (Some(key_str), Some(value_str)) = (key.as_str(), value.as_str()) {
                file_hashmap.insert(key_str.to_string(), value_str.to_string());
            }
        }
        Some(serde_json::to_string(&file_hashmap).unwrap())
    } else {
        None
    }
}
// Function to parse an environment file and return its contents as a hashmap string
fn parse_env_file(file_path: &str) -> Option<String> {
    let file_string = if file_path == "None" {
        fs::read_to_string(".env").ok()?
    } else {
        fs::read_to_string(file_path).ok()?
    };
    let mut file_hashmap = HashMap::new();
    for line in file_string.lines() {
        if let Some((key, value)) = parse_env_line(line) {
            file_hashmap.insert(key, value);
        }
    }
    Some(serde_json::to_string(&file_hashmap).unwrap())
}
// Function to parse a single line of an environment file and return the key-value pair
fn parse_env_line(line: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = line.splitn(2, '=').collect();
    if parts.len() == 2 {
        Some((parts[0], parts[1]))
    } else {
        None
    }
}
// Function to add a source as a key to a hashmap string
fn add_soruce_key(data: String, key: &str) -> String {
    format!("{{\"{}\":{}}}", key, data)
}
// Function to remove backslash quotes from a hashmap string
fn remove_backslash_quotes(data: &str) -> String {
    data.replace("\"\\", "").replace("\\\"", "")
}
// Function to merge two json objects
fn merge_variables(
    mut additional_variable: serde_json::Value,
    accumulated_variable: serde_json::Value,
) -> serde_json::Value {
    if let serde_json::Value::Object(additional_variable_map) = &mut additional_variable {
        if let serde_json::Value::Object(accumulated_variable_map) = accumulated_variable {
            for (key, source_value) in accumulated_variable_map {
                additional_variable_map
                    .entry(key.clone())
                    .and_modify(|target_value| {
                        *target_value = merge_variables(target_value.clone(), source_value.clone());
                    })
                    .or_insert(source_value);
            }
        }
    }
    additional_variable
}
// Function to accumulate the hashmap with preprocessed key and value pair objects
fn accumulate_with_preprocess(
    accumulated_variable: &serde_json::Value,
    additional_variable: &str,
) -> serde_json::Value {
    let preprocessed_additional_variable: serde_json::Value =
        serde_json::from_str(&remove_backslash_quotes(additional_variable))
            .unwrap_or(serde_json::json!({}));
    merge_variables(
        preprocessed_additional_variable,
        accumulated_variable.clone(),
    )
}
// Function to extract variable reading the variable file description
fn get_variable(description: &[(String, Option<String>)]) -> serde_json::Value {
    let mut variable: serde_json::Value = serde_json::json!({});

    for (source_type, source_path) in description {
        match source_type.as_str() {
            "env" => {
                if let Some(source_path) = source_path {
                    if let Some(file_string) = parse_env_file(source_path) {
                        variable = accumulate_with_preprocess(
                            &variable,
                            &add_soruce_key(file_string, "env"),
                        );
                    }
                }
            }
            "json" => {
                if let Some(source_path) = source_path {
                    if let Some(file_string) = parse_json_file(source_path) {
                        variable = accumulate_with_preprocess(
                            &variable,
                            &add_soruce_key(file_string, "json"),
                        );
                    }
                }
            }
            "yaml" => {
                if let Some(source_path) = source_path {
                    if let Some(file_string) = parse_yaml_file(source_path) {
                        variable = accumulate_with_preprocess(
                            &variable,
                            &add_soruce_key(file_string, "yaml"),
                        );
                    }
                }
            }
            _ => (),
        }
    }
    variable
}
// Define a struct to represent the yaml content
#[derive(Debug, Serialize, Deserialize)]
struct VariableFileDescription {
    env: Option<String>,
    json: Option<String>,
    yaml: Option<String>,
}
// Function to read the variable file description
fn read_variable_file_description() -> Vec<(String, Option<String>)> {
    // Read the yaml file
    let file_path = "core/data-layer/tests/variables-examples/variables_file_description.yaml";
    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(error) => match error.kind() {
            ErrorKind::NotFound => {
                eprintln!("Default variable file description not found.");
                return Vec::new();
            }
            _ => panic!(
                "Failed to open default variable file description: {:?}",
                error
            ),
        },
    };
    let mut yaml_content = String::new();
    file.read_to_string(&mut yaml_content)
        .expect("Failed to read file.");

    // Deserialize the yaml content into the struct
    let variable_file_description: VariableFileDescription =
        serde_yaml::from_str(&yaml_content).expect("Failed to parse yaml.");

    // Construct the output vector
    let mut result = Vec::new();
    let mut hashmap = HashMap::new();

    if let Some(env) = variable_file_description.env {
        hashmap.insert("env".to_string(), Some(env));
    } else {
        hashmap.insert("env".to_string(), None);
    }

    if let Some(json) = variable_file_description.json {
        hashmap.insert("json".to_string(), Some(json));
    } else {
        hashmap.insert("json".to_string(), None);
    }

    if let Some(yaml) = variable_file_description.yaml {
        hashmap.insert("yaml".to_string(), Some(yaml));
    } else {
        hashmap.insert("yaml".to_string(), None);
    }

    for (key, value) in hashmap {
        result.push((key, value));
    }

    result
}
pub fn read_definition_yaml_file<P>(path: P) -> Result<ParserResult>
where
    P: AsRef<Path>,
{
    // Read the file of the path
    let mut file = File::open(path)?;
    // Read the yaml variables file description
    let variable_file_description = read_variable_file_description();
    // Create a hashmap from the variable file description
    let variable = get_variable(&variable_file_description);
    // Read to a string from the file
    let mut file_string = String::new();
    file.read_to_string(&mut file_string)?;
    // Render the file with the variable hashmap using handlebars
    let handlebars = Handlebars::new();
    let variable_json = serde_json::json!(variable);
    let new_file = handlebars.render_template(file_string.as_str(), &variable_json)?;
    let file_cursor = std::io::Cursor::new(new_file.as_bytes());
    /*  */
    // Make a deserializer to iterate the yaml that could have multiple documents
    let deserializer = Deserializer::from_reader(file_cursor);
    // For result
    let mut result = ParserResult::default();
    // Each document
    for document in deserializer {
        let value = serde_yaml::Value::deserialize(document)?;
        // The "kind" tells the document type like Metric, ScalingPlan
        if let Some(kind) = value.get("kind").and_then(serde_yaml::Value::as_str) {
            match kind {
                "Metric" => {
                    let parsed = serde_yaml::from_value::<MetricDefinition>(value)?;
                    parsed.validate()?;
                    result.metric_definitions.push(parsed);
                }
                "SLO" => {
                    let parsed = serde_yaml::from_value::<SloDefinition>(value)?;
                    parsed.validate()?;
                    result.slo_definitions.push(parsed);
                }
                "ScalingPlan" => {
                    let parsed = serde_yaml::from_value::<ScalingPlanDefinition>(value)?;
                    parsed.validate()?;
                    result.scaling_plan_definitions.push(parsed);
                }
                "ScalingComponent" => {
                    let parsed = serde_yaml::from_value::<ScalingComponentDefinition>(value)?;
                    parsed.validate()?;
                    result.scaling_component_definitions.push(parsed);
                }
                _ => println!("Not Found: {:?}", kind),
            }
        } else {
            // TODO: "kind" doesn't exist
        }
    }
    Ok(result)
}
