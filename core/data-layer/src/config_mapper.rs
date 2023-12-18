use crate::{MetricDefinition, ScalingComponentDefinition, ScalingPlanDefinition, SloDefinition};
use dotenv_parser::parse_dotenv;
use handlebars::Handlebars;
use serde::Deserialize;
use serde_valid::Validate;
use serde_yaml::Deserializer;
use std::{collections::HashMap, fs, path::Path};
use tracing::error;

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
    let yaml_source_path_str = yaml_source_path.to_str().unwrap_or("").to_string();
    let env_source_path_str = env_source_path.to_str().unwrap_or("").to_string();
    let json_source_path_str = json_source_path.to_str().unwrap_or("").to_string();

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
// Function to extract parameters from an env file
fn extract_params_from_env_file(file_path: &str) -> Option<serde_json::Value> {
    let file_string = fs::read_to_string(file_path).ok()?;
    let file_map = parse_dotenv(&file_string).ok()?;

    let mut file_hashmap = HashMap::new();
    for (key, value) in file_map.iter() {
        file_hashmap.insert(key, value);
    }

    serde_json::to_value(file_hashmap).ok()
}
// Function to remove backslash quotes from a json string
fn remove_backslash_quotes(data: &str) -> String {
    data.replace('\"', "")
}
// Function to extract parameters from a json file
fn extract_params_from_json_file(file_path: &str) -> Option<serde_json::Value> {
    let file_string = fs::read_to_string(file_path).ok()?;
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

// Function to get parameters for handlebars
pub fn get_params_for_handlebars<P>(path: P) -> serde_json::Value
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

// Function to synchronize data type from a string
fn synchronize_data_type(data: &str) -> String {
    data.replace(['\\'], "")
        .replace(['[', ']'], "")
        .replace("\\n", "\n")
}
pub fn get_config_mapper<P>(
    template: String,
    variable_path: P,
) -> Result<ParserResult, anyhow::Error>
where
    P: AsRef<Path>,
{
    // Get parameters for handlebars
    let params_for_handlebars = get_params_for_handlebars(variable_path);
    // Render the file with the variables using handlebars
    let handlebars = Handlebars::new();
    let template = handlebars.render_template(
        synchronize_data_type(&template).as_str(),
        &params_for_handlebars,
    )?;
    // Make a cursor to read the new file
    let file_cursor = std::io::Cursor::new(template.as_bytes());
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
                _ => error!("Not Found: {:?}", kind),
            }
        }
    }
    Ok(result)
}

#[tokio::test]
async fn test_get_source_paths() {
    let path = "tests/variables-examples/example.yaml";
    let (yaml_source_path, env_source_path, json_source_path) = get_source_paths(path);
    println!("yaml_source_path: {:?}", yaml_source_path);
    println!("env_source_path: {:?}", env_source_path);
    println!("json_source_path: {:?}", json_source_path);
    assert_eq!(yaml_source_path, "tests/variables-examples/variables.yaml");
    assert_eq!(env_source_path, "tests/variables-examples/variables.env");
    assert_eq!(json_source_path, "tests/variables-examples/variables.json");
}

#[tokio::test]
async fn test_get_params_for_handlebars() {
    let path = "tests/variables-examples/example.yaml";
    let params_for_handlebars = get_params_for_handlebars(path);
    println!("params_for_handlebars: {:?}", params_for_handlebars);
    assert_eq!(
        params_for_handlebars,
        serde_json::json!({
            "yaml": {
                "user_1_access_key": "user_1_access_key",
                "user_1_secret_key": "user_1_secret_key",
                "user_1_region": "ap-northeast-3",
            },
            "env": {
                "user_1_access_key": "access_key",
                "user_1_secret_key": "user_1_secret_key",
                "user_1_region": "user_1_region",
            },
            "json": {
                "user_1_access_key": "user_1_access_key",
                "user_1_secret_key": "secret_key",
                "user_1_region": "user_1_region",
            },
        })
    );
}

#[tokio::test]
async fn test_render_transformation_1() {
    use std::{fs::File, io::Read};

    let path = "tests/variables-examples/example.yaml";

    // Read the file of the path
    let mut file = File::open(path).unwrap();
    // Read to a string from the file
    let mut file_string = String::new();
    file.read_to_string(&mut file_string).unwrap();

    println!("file_string: {:?}", file_string);

    let result = get_config_mapper(file_string, path);

    println!("result: {:?}", result);
    /* result: Ok(ParserResult { metric_definitions: [], slo_definitions: [], scaling_plan_definitions: [], scaling_component_definitions: [ScalingComponentDefinition { kind: ScalingComponent, db_id: "", id: "dynamodb_table", component_kind: "amazon-dynamodb", metadata: {"region": String("ap-northeast-3"), "access_key": String("access_key"), "secret_key": String("secret_key"), "table_name": String("test-dynamodb-table")} }] }) */

    if let Some(scaling_component) = result.unwrap().scaling_component_definitions.first() {
        let metadata = &scaling_component.metadata;
        assert_eq!(
            metadata.get("access_key").and_then(|v| v.as_str()),
            Some("access_key")
        );
        assert_eq!(
            metadata.get("secret_key").and_then(|v| v.as_str()),
            Some("secret_key")
        );
        assert_eq!(
            metadata.get("region").and_then(|v| v.as_str()),
            Some("ap-northeast-3")
        );
    }
}

#[tokio::test]
async fn test_render_transformation_2() {
    let path = "tests/variables-examples/example.yaml";

    let file_string = r#"---\nkind: ScalingComponent\nid: dynamodb_table\ncomponent_kind: amazon-dynamodb\nmetadata:\n  region: {{ yaml.user_1_region }}\n  access_key: {{ env.user_1_access_key }}\n  secret_key: {{ json.user_1_secret_key }}\n  table_name: test-dynamodb-table\n"#
    .to_string().replace("\\n", "\n");

    let result = get_config_mapper(file_string, path);

    println!("result: {:?}", result);
    /* result: Ok(ParserResult { metric_definitions: [], slo_definitions: [], scaling_plan_definitions: [], scaling_component_definitions: [ScalingComponentDefinition { kind: ScalingComponent, db_id: "", id: "dynamodb_table", component_kind: "amazon-dynamodb", metadata: {"region": String("ap-northeast-3"), "table_name": String("test-dynamodb-table"), "access_key": String("access_key"), "secret_key": String("secret_key")} }] }) */

    if let Some(scaling_component) = result.unwrap().scaling_component_definitions.first() {
        let metadata = &scaling_component.metadata;
        assert_eq!(
            metadata.get("access_key").and_then(|v| v.as_str()),
            Some("access_key")
        );
        assert_eq!(
            metadata.get("secret_key").and_then(|v| v.as_str()),
            Some("secret_key")
        );
        assert_eq!(
            metadata.get("region").and_then(|v| v.as_str()),
            Some("ap-northeast-3")
        );
    }
}

#[tokio::test]
async fn test_render_transformation_3() {
    let path = "tests/variables-examples/example.yaml";

    let original_string = r#"[{\"kind\":\"ScalingComponent\",\"db_id\":\"02671e96-ed1d-433d-b622-5b0ee812a0ba\",\"id\":\"aws_ecs_scaling_component\",\"component_kind\":\"amazon-ecs\",\"metadata\":{\"region\":\"{{ yaml.user_1_region }}\"},\"created_at\":\"2023-12-18T13:19:24.614105+00:00\",\"updated_at\":\"2023-12-18T13:19:24.614105+00:00\"}]"#;
    let file_string = original_string
        .to_string()
        .replace(['\\'], "")
        .replace(['[', ']'], "");
    println!("file_string: {:?}", file_string);

    let result = get_config_mapper(file_string, path);

    println!("result: {:?}", result);
    /* result: Ok(ParserResult { metric_definitions: [], slo_definitions: [], scaling_plan_definitions: [], scaling_component_definitions: [ScalingComponentDefinition { kind: ScalingComponent, db_id: "02671e96-ed1d-433d-b622-5b0ee812a0ba", id: "aws_ecs_scaling_component", component_kind: "amazon-ecs", metadata: {"region": String("ap-northeast-3")} }] }) */

    if let Some(scaling_component) = result.unwrap().scaling_component_definitions.first() {
        let metadata = &scaling_component.metadata;
        assert_eq!(
            metadata.get("region").and_then(|v| v.as_str()),
            Some("ap-northeast-3")
        );
    }
}
