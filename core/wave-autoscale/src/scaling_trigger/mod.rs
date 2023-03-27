use std::collections::HashMap;

use async_trait::async_trait;
use data_layer::ScalingTriggerData;
pub mod aws_ec2_autoscaling;
use anyhow::Result;

use self::aws_ec2_autoscaling::{EC2AutoScalingTrigger};

pub fn create_scaling_trigger(scaling_trigger_data: &ScalingTriggerData) -> Result<Box<dyn ScalingTrigger>> {
    // Get a value of metric and clone it.
    let cloned_trigger_data = scaling_trigger_data.clone();
    match cloned_trigger_data.trigger_kind.as_str() {
        EC2AutoScalingTrigger::TRIGGER_KIND => Ok(Box::new(
            EC2AutoScalingTrigger::new(cloned_trigger_data),
        )),
        _ => Err(anyhow::anyhow!("Unknown trigger kind")),
    }
}

pub enum ScalingTriggerValueType {
    Int(i64),
    Float(f64),
    String(String),
}

#[async_trait]
pub trait ScalingTrigger {
    async fn apply(&self, params: HashMap<String, ScalingTriggerValueType>);
    fn get_trigger_kind(&self) -> &str;
}
