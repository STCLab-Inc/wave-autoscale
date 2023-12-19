use dotenv_parser::parse_dotenv;
use handlebars::Handlebars;
use std::env;
use std::{collections::HashMap, fs};
use tracing::debug;

// Function to extract environment variables from the system
fn extract_params_from_env() -> Option<serde_json::Value> {
    let mut env_vars = HashMap::new();
    for (key, value) in env::vars() {
        env_vars.insert(key, value);
    }

    serde_json::to_value(env_vars).ok()
}
// Function to extract parameters from a yaml file
fn extract_params_from_yaml_file(path: &str) -> Option<serde_json::Value> {
    let file_string = fs::read_to_string(path).ok()?;
    let file_yaml: serde_yaml::Value = serde_yaml::from_str(&file_string).ok()?;

    let file_map = match file_yaml.as_mapping() {
        Some(file_map) => file_map,
        None => return None,
    };

    let mut file_hashmap = HashMap::new();
    for (key, value) in file_map.iter() {
        if let (Some(key_str), Some(value_str)) = (key.as_str(), value.as_str()) {
            file_hashmap.insert(key_str.to_string(), value_str.to_string());
        }
    }

    serde_json::to_value(file_hashmap).ok()
}
// Function to remove backslash quotes from a json string
fn remove_backslash_quotes(data: &str) -> String {
    data.replace('\"', "")
}
// Function to extract parameters from a json file
fn extract_params_from_json_file(path: &str) -> Option<serde_json::Value> {
    let file_string = fs::read_to_string(path).ok()?;
    let file_json: serde_json::Value = serde_json::from_str(&file_string).ok()?;

    let file_object = match file_json.as_object() {
        Some(file_object) => file_object,
        None => return None,
    };

    let mut file_hashmap = HashMap::new();
    for (key, value) in file_object {
        file_hashmap.insert(
            remove_backslash_quotes(key),
            remove_backslash_quotes(&value.to_string()),
        );
    }

    serde_json::to_value(file_hashmap).ok()
}
// Function to extract parameters from an env file
fn extract_params_from_env_file(path: &str) -> Option<serde_json::Value> {
    let file_string = fs::read_to_string(path).ok()?;
    let file_map = parse_dotenv(&file_string).ok()?;

    let mut file_hashmap = HashMap::new();
    for (key, value) in file_map.iter() {
        file_hashmap.insert(key, value);
    }

    serde_json::to_value(file_hashmap).ok()
}

// Function to get config mapper
pub fn get_variable_mapper() -> serde_json::Value {
    let yaml_source_path = "./variables.yaml".to_string();
    let json_source_path = "./variables.json".to_string();
    let env_source_path = "./variables.env".to_string();

    let variables_from_yaml_file = extract_params_from_yaml_file(&yaml_source_path);
    let variables_from_json_file = extract_params_from_json_file(&json_source_path);
    let mut variables_from_env_file = extract_params_from_env_file(&env_source_path)
        .unwrap_or_default()
        .as_object()
        .unwrap_or(&serde_json::Map::new())
        .clone();
    let variables_from_env = extract_params_from_env()
        .unwrap_or_default()
        .as_object()
        .unwrap_or(&serde_json::Map::new())
        .clone();

    variables_from_env_file.extend(variables_from_env);

    serde_json::json!({
        "yaml": serde_json::json!(variables_from_yaml_file),
        "json": serde_json::json!(variables_from_json_file),
        "env": serde_json::json!(variables_from_env_file),

    })
}

// Function to execute config mapper
pub fn execute_variable_mapper(
    template: String,
    data: serde_json::Value,
) -> Result<String, anyhow::Error> {
    let handlebars = Handlebars::new();

    let result = match handlebars.render_template(&template, &data) {
        Ok(rendered) => rendered,
        Err(e) => {
            debug!("Check rendering template: {}", e);
            template
        }
    };

    Ok(result)
}

#[tokio::test]
async fn test_get_variable_mapper() {
    let result = get_variable_mapper();

    println!("RESULT: {:?}", result);

    result
        .get("yaml")
        .map(|value| {
            println!("YAML: {:?}", value);
        })
        .unwrap_or_else(|| {
            println!("No YAML data found");
        });

    result
        .get("json")
        .map(|value| {
            println!("JSON: {:?}", value);
        })
        .unwrap_or_else(|| {
            println!("No JSON data found");
        });

    result
        .get("env")
        .map(|value| {
            println!("ENV: {:?}", value);
        })
        .unwrap_or_else(|| {
            println!("No ENV data found");
        });
}

#[tokio::test]
async fn test_execute_variable_mapper() {
    let data = get_variable_mapper();
    println!("DATA: {:?}", data);
    let result = execute_variable_mapper(
        "{{yaml.user_1_access_key}} {{json.user_2_access_key}} {{env.user_3_region}}".to_string(),
        data,
    );
    println!("RESULT: {:?}", result);
}
