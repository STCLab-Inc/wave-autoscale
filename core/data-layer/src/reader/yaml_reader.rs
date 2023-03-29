use anyhow::Result;
use serde::Deserialize;
use serde_yaml::{Deserializer, Value};
use std::{fs::File, path::Path};

use crate::{MetricDefinition, ScalingComponentDefinition, ScalingPlanDefinition, SloDefinition};

#[derive(Debug)]
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
    let mut result = ParserResult {
        metric_definitions: Vec::new(),
        slo_definitions: Vec::new(),
        scaling_plan_definitions: Vec::new(),
        scaling_component_definitions: Vec::new(),
    };
    // Each document
    while let Some(document) = deserializer.next() {
        let value = Value::deserialize(document)?;
        // The "kind" tells the document type like Metric, ScalingPlan
        if let Some(kind) = value.get("kind") {
            if let Value::String(kind_str) = kind {
                println!("Found: {:?}", kind_str);
                if kind_str == "Metric" {
                    let metric_definition: MetricDefinition = serde_yaml::from_value(value)?;
                    result.metric_definitions.push(metric_definition);
                } else if kind_str == "SLO" {
                    let slo_definition: SloDefinition = serde_yaml::from_value(value)?;
                    result.slo_definitions.push(slo_definition);
                } else if kind_str == "ScalingPlan" {
                    let scaling_plan_definition: ScalingPlanDefinition =
                        serde_yaml::from_value(value)?;
                    result
                        .scaling_plan_definitions
                        .push(scaling_plan_definition);
                } else if kind_str == "ScalingComponent" {
                    let scaling_component_definition: ScalingComponentDefinition =
                        serde_yaml::from_value(value)?;
                    result
                        .scaling_component_definitions
                        .push(scaling_component_definition);
                } else {
                    println!("Not Found: {:?}", kind_str);
                }
            }
        } else {
            // TODO: "kind" doesn't exist
        }
    }
    Ok(result)
}
