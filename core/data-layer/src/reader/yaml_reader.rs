use anyhow::Result;
use serde::Deserialize;
use serde_yaml::{Deserializer, Value};
use std::{fs::File, path::Path};

use crate::{Metric, ScalingPlanData, ScalingTriggerData, SLO};

#[derive(Debug)]
pub struct ParserResult {
    pub metrics: Vec<Metric>,
    pub slos: Vec<SLO>,
    pub scaling_plans: Vec<ScalingPlanData>,
    pub scaling_triggers: Vec<ScalingTriggerData>,
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
        metrics: Vec::new(),
        slos: Vec::new(),
        scaling_plans: Vec::new(),
        scaling_triggers: Vec::new(),
    };
    // Each document
    while let Some(document) = deserializer.next() {
        let value = Value::deserialize(document)?;
        // The "kind" tells the document type like Metric, ScalingPlan
        if let Some(kind) = value.get("kind") {
            if let Value::String(kind_str) = kind {
                println!("Found: {:?}", kind_str);
                if kind_str == "Metric" {
                    let metric: Metric = serde_yaml::from_value(value)?;
                    result.metrics.push(metric);
                } else if kind_str == "SLO" {
                    let slo: SLO = serde_yaml::from_value(value)?;
                    result.slos.push(slo);
                } else if kind_str == "ScalingPlans" {
                    let scaling_plan: ScalingPlanData = serde_yaml::from_value(value)?;
                    result.scaling_plans.push(scaling_plan);
                } else if kind_str == "ScalingTrigger" {
                    let scaling_trigger: ScalingTriggerData = serde_yaml::from_value(value)?;
                    result.scaling_triggers.push(scaling_trigger);
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
