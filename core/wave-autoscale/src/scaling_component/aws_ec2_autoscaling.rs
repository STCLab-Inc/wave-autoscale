use super::ScalingComponent;
use crate::util::aws::get_aws_config;
use anyhow::{Ok, Result};
use async_trait::async_trait;
use aws_sdk_autoscaling::{error::ProvideErrorMetadata, Client};
use data_layer::ScalingComponentDefinition;
use serde_json::{json, Value};
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
            let config = get_aws_config(
                Some(region.to_string()),
                Some(access_key.to_string()),
                Some(secret_key.to_string()),
                None,
                None,
            )
            .await;
            if config.is_err() {
                let config_err = config.err().unwrap();
                return Err(anyhow::anyhow!(config_err));
            }
            let config = config.unwrap();
            let client = Client::new(&config);

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

            if result.is_err() {
                let error = result.err().unwrap();
                // error.
                let meta = error.meta();
                // meta.ex
                let json = json!({
                    "message": meta.message(),
                    "code": meta.code(),
                    "extras": meta.to_string()
                });
                return Err(anyhow::anyhow!(json));
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("Invalid metadata"))
        }
    }
}
