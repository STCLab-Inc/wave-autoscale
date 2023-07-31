use crate::{MetricDefinition, ScalingComponentDefinition, ScalingPlanDefinition, SloDefinition};
use anyhow::Result;
use dotenv_parser::parse_dotenv;
use handlebars::Handlebars;
use serde::Deserialize;
use serde_valid::Validate;
use serde_yaml::Deserializer;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
    path::Path,
};

#[derive(Debug, Default)]
pub struct ParserResult {
    pub metric_definitions: Vec<MetricDefinition>,
    pub slo_definitions: Vec<SloDefinition>,
    pub scaling_plan_definitions: Vec<ScalingPlanDefinition>,
    pub scaling_component_definitions: Vec<ScalingComponentDefinition>,
}

// Function to get the source paths
fn get_source_paths<P: AsRef<Path>>(input: P) -> (String, String, String) {
    // Convert the input path to a path
    let path = input.as_ref();

    // Get the parent directory of the input path
    let parent_path = path.parent().unwrap_or(Path::new(""));

    // Construct the new file paths using path bufs
    let yaml_source_path = parent_path.join("variables.yaml");
    let env_source_path = parent_path.join("variables.env");
    let json_source_path = parent_path.join("variables.json");

    // Convert the path bufs to strings
    let yaml_source_path_str = yaml_source_path.to_str().unwrap().to_string();
    let env_source_path_str = env_source_path.to_str().unwrap().to_string();
    let json_source_path_str = json_source_path.to_str().unwrap().to_string();

    // Return the source paths
    (
        yaml_source_path_str,
        env_source_path_str,
        json_source_path_str,
    )
}

// Function to extract parameters from a yaml file
fn extract_params_from_yaml_file(file_path: &str) -> Option<serde_json::Value> {
    let file_string = fs::read_to_string(file_path).ok()?;
    let file_yaml: serde_yaml::Value = serde_yaml::from_str(&file_string).ok()?;
    if file_yaml.as_mapping().is_none() {
        None
    } else {
        let file_map = file_yaml.as_mapping().unwrap();
        let mut file_hashmap = HashMap::new();
        for (key, value) in file_map.iter() {
            if let (Some(key_str), Some(value_str)) = (key.as_str(), value.as_str()) {
                file_hashmap.insert(key_str, value_str);
            }
        }
        Some(serde_json::to_value(file_hashmap).unwrap())
    }
}
// Function to extract parameters from an env file
fn extract_params_from_env_file(file_path: &str) -> Option<serde_json::Value> {
    let file_string = fs::read_to_string(file_path).ok()?;
    let file_map = parse_dotenv(&file_string).unwrap();
    let mut file_hashmap = HashMap::new();
    for (key, value) in file_map.iter() {
        file_hashmap.insert(key, value);
    }
    Some(serde_json::to_value(file_hashmap).unwrap())
}
// Function to remove backslash quotes from a json string
fn remove_backslash_quotes(data: &str) -> String {
    data.replace('\"', "")
}
// Function to extract parameters from a json file
fn extract_params_from_json_file(file_path: &str) -> Option<serde_json::Value> {
    let file_string = fs::read_to_string(file_path).ok()?;
    let file_json: serde_json::Value = serde_json::from_str(&file_string).ok()?;
    if file_json.as_object().is_none() {
        None
    } else {
        let file_object = file_json.as_object().unwrap();
        let mut file_hashmap = HashMap::new();
        for (key, value) in file_object {
            file_hashmap.insert(
                remove_backslash_quotes(key),
                remove_backslash_quotes(&value.to_string()),
            );
        }
        Some(serde_json::to_value(file_hashmap).unwrap())
    }
}
// Function to get parameters for handlebars
fn get_params_for_handlebars<P>(path: P) -> serde_json::Value
where
    P: AsRef<Path>,
{
    let (variables_path_from_yaml, variables_path_from_env, variables_path_from_json) =
        get_source_paths(&path);
    let variables_from_yaml = extract_params_from_yaml_file(&variables_path_from_yaml);
    let variables_from_env = extract_params_from_env_file(&variables_path_from_env);
    let variables_from_json = extract_params_from_json_file(&variables_path_from_json);
    serde_json::json!({
        "yaml": serde_json::json!(variables_from_yaml),
        "env": serde_json::json!(variables_from_env),
        "json": serde_json::json!(variables_from_json),
    })
}

pub fn read_definition_yaml_file<P>(path: P) -> Result<ParserResult>
where
    P: AsRef<Path>,
{
    // Read the file of the path
    let mut file = File::open(&path)?;
    // Get parameters for handlebars
    let params_for_handlebars = get_params_for_handlebars(&path);
    // Read to a string from the file
    let mut file_string = String::new();
    file.read_to_string(&mut file_string)?;
    // Render the file with the variables using handlebars
    let handlebars = Handlebars::new();
    let new_file = handlebars.render_template(file_string.as_str(), &params_for_handlebars)?;
    // Make a cursor to read the new file
    let file_cursor = std::io::Cursor::new(new_file.as_bytes());
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
