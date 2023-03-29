use super::ScalingComponent;
use crate::util::aws_region::get_aws_region_static_str;
use anyhow::{Ok, Result};
use async_trait::async_trait;
use aws_sdk_autoscaling::{Client, Credentials};
use data_layer::ScalingComponentDefinition;
use serde_json::Value;
use std::collections::HashMap;

// This is a metric adapter for prometheus.
pub struct EC2AutoScalingComponent {
    definition: ScalingComponentDefinition,
}

impl EC2AutoScalingComponent {
    // Static variables
    pub const TRIGGER_KIND: &'static str = "aws-ec2-autoscaling";

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
        // let region = Region::new(region.clone());
        let asg_name = metadata["asg_name"].as_str().unwrap();
        let access_key = metadata["access_key"].as_str().unwrap();
        let secret_key = metadata["secret_key"].as_str().unwrap();
        let credentials = Credentials::new(access_key, secret_key, None, None, "wave-autoscale");
        let region = metadata["region"].as_str().unwrap();
        println!("region: {}", region);
        // aws_config needs a static region string
        let region_static: &'static str = get_aws_region_static_str(region);
        let shared_config = aws_config::from_env()
            .region(region_static)
            .credentials_provider(credentials)
            .load()
            .await;

        let client = Client::new(&shared_config);

        let min = params["min"].as_i64().unwrap();
        let max = params["max"].as_i64().unwrap();
        let desired = params["desired"].as_i64().unwrap();

        let result = client
            .update_auto_scaling_group()
            .auto_scaling_group_name(asg_name)
            .min_size(min as i32)
            .max_size(max as i32)
            .desired_capacity(desired as i32)
            .send()
            .await;

        println!("{:?}", result);
        Ok(())
    }
}
