use super::ScalingComponent;
use crate::util::aws_region::get_aws_region_static_str;
use anyhow::{Ok, Result};
use async_trait::async_trait;
use aws_sdk_autoscaling::{config::Credentials, Client};
use data_layer::ScalingComponentDefinition;
use serde_json::Value;
use std::collections::HashMap;

// This is a metric adapter for prometheus.
pub struct EC2AutoScalingComponent {
    definition: ScalingComponentDefinition,
}

impl EC2AutoScalingComponent {
    // Static variables
    pub const SCALING_KIND: &'static str = "aws-ec2-autoscaling";

    // Functions
    pub fn new(definition: ScalingComponentDefinition) -> Self {
        EC2AutoScalingComponent { definition }
    }
}

#[async_trait]
impl ScalingComponent for EC2AutoScalingComponent {
    fn get_scaling_component_kind(&self) -> &str {
        &self.definition.component_kind
    }
    fn get_id(&self) -> &str {
        &self.definition.id
    }
    async fn apply(&self, params: HashMap<String, Value>) -> Result<()> {
        let metadata = self.definition.metadata.clone();

        if let (
            Some(Value::String(asg_name)),
            Some(Value::String(access_key)),
            Some(Value::String(secret_key)),
            Some(Value::String(region)),
            Some(desired),
        ) = (
            metadata.get("asg_name"),
            metadata.get("access_key"),
            metadata.get("secret_key"),
            metadata.get("region"),
            params.get("desired").and_then(Value::as_i64),
        ) {
            let credentials =
                Credentials::new(access_key, secret_key, None, None, "wave-autoscale");
            // aws_config needs a static region string
            let region_static: &'static str = get_aws_region_static_str(region);
            let shared_config = aws_config::from_env()
                .region(region_static)
                .credentials_provider(credentials)
                .load()
                .await;

            let client = Client::new(&shared_config);

            let mut result = client
                .update_auto_scaling_group()
                .auto_scaling_group_name(asg_name)
                .desired_capacity(desired as i32);

            if let Some(min) = params.get("min").and_then(Value::as_i64) {
                result = result.min_size(min as i32);
            }
            if let Some(max) = params.get("max").and_then(Value::as_i64) {
                result = result.max_size(max as i32);
            }

            let result = result.send().await;

            println!("{:?}", result);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Invalid metadata"))
        }
    }
}
