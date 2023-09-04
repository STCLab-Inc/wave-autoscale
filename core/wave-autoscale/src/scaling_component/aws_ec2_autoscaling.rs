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

        if let (Some(Value::String(asg_name)), Some(Value::String(region)), Some(desired)) = (
            metadata.get("asg_name"),
            metadata.get("region"),
            params.get("desired").and_then(Value::as_i64),
        ) {
            let access_key = metadata
                .get("access_key")
                .map(|access_key| access_key.to_string());
            let secret_key = metadata
                .get("secret_key")
                .map(|secret_key| secret_key.to_string());

            let config =
                get_aws_config(Some(region.to_string()), access_key, secret_key, None, None).await;
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
            println!(" >> Invalid metadata");
            Err(anyhow::anyhow!("Invalid metadata"))
        }
    }
}

#[test]
fn test() {
    let mut metadata: HashMap<String, Value> = HashMap::new();
    metadata.insert(
        "region".to_string(),
        Value::String("ap-northeast-3".to_string()),
    );
    metadata.insert(
        "asg_name".to_string(),
        Value::String("wave-ec2-as".to_string()),
    );
    // metadata.insert("access_key".to_string(), Value::Null);
    // metadata.insert("secret_key".to_string(), Value::Null);
    println!(" >> EC2 metadata - {:?}", metadata.clone());

    let mut params: HashMap<String, Value> = HashMap::new();
    params.insert("desired".to_string(), json!(2));
    println!(" >> EC2 params - {:?}", params.clone());

    if let (
        Some(Value::String(_asg_name)),
        // Some(_access_key),
        // Some(_secret_key),
        Some(Value::String(_region)),
        Some(_desired),
    ) = (
        metadata.get("asg_name"),
        // metadata.get("access_key"),
        // metadata.get("secret_key"),
        metadata.get("region"),
        params.get("desired").and_then(Value::as_i64),
    ) {
        let _access_key = metadata.get("access_key");
        let _secret_key = metadata.get("secret_key");
        println!("ok");
    } else {
        println!("err");
    }
}
