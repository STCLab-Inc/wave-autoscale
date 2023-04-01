use crate::{MetricDefinition, ScalingComponentDefinition, ScalingPlanDefinition, SloDefinition};
use anyhow::Result;
use serde::Deserialize;
use serde_valid::Validate;
use serde_yaml::{Deserializer, Value};
use std::{fs::File, path::Path};

#[derive(Debug, Default)]
pub struct ParserResult {
    pub metric_definitions: Vec<MetricDefinition>,
    pub slo_definitions: Vec<SloDefinition>,
    pub scaling_plan_definitions: Vec<ScalingPlanDefinition>,
    pub scaling_component_definitions: Vec<ScalingComponentDefinition>,
}

pub fn read_yaml_file<P>(path: P) -> Result<ParserResult>
where
    P: AsRef<Path>,
{
    // Read the file of the path
    let file = File::open(path)?;
    // Make a deserializer to iterate the yaml that could have multiple documents
    let mut deserializer = Deserializer::from_reader(file);
    // For result
    let mut result = ParserResult::default();
    // Each document
    while let Some(document) = deserializer.next() {
        let value = Value::deserialize(document)?;
        // The "kind" tells the document type like Metric, ScalingPlan
        if let Some(kind) = value.get("kind").and_then(Value::as_str) {
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
