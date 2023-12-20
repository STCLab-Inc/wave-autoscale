use crate::{MetricDefinition, ScalingComponentDefinition, ScalingPlanDefinition, SloDefinition};
use anyhow::Result;
use serde::Deserialize;
use serde_valid::Validate;
use serde_yaml::Deserializer;
use std::{fs::File, io::Read, path::Path};
use tracing::error;

#[derive(Debug, Default)]
pub struct ParserResult {
    pub metric_definitions: Vec<MetricDefinition>,
    pub slo_definitions: Vec<SloDefinition>,
    pub scaling_plan_definitions: Vec<ScalingPlanDefinition>,
    pub scaling_component_definitions: Vec<ScalingComponentDefinition>,
}

pub fn read_definition_yaml_file<P>(path: P) -> Result<ParserResult>
where
    P: AsRef<Path>,
{
    // Read the file of the path
    let mut file = File::open(&path)?;
    // Read to a string from the file
    let mut file_string = String::new();
    file.read_to_string(&mut file_string)?;
    // Make a cursor to read the new file
    let file_cursor = std::io::Cursor::new(file_string.as_bytes());
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
        } else {
            // TODO: "kind" doesn't exist
        }
    }
    Ok(result)
}
