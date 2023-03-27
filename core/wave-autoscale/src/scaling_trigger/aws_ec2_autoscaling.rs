use std::collections::HashMap;

use async_trait::async_trait;
use aws_sdk_autoscaling::{Client, Credentials, Region};
use data_layer::ScalingTriggerData;

use super::{ScalingTrigger, ScalingTriggerValueType};

// This is a metric adapter for prometheus.
pub struct EC2AutoScalingTrigger {
    scaling_trigger: ScalingTriggerData,
}

impl EC2AutoScalingTrigger {
    // Static variables
    pub const TRIGGER_KIND: &'static str = "aws-ec2-autoscaling";

    // Functions
    pub fn new(scaling_trigger: ScalingTriggerData) -> Self {
        EC2AutoScalingTrigger { scaling_trigger }
    }
}

#[async_trait]
impl ScalingTrigger for EC2AutoScalingTrigger {
    fn get_trigger_kind(&self) -> &str {
        EC2AutoScalingTrigger::TRIGGER_KIND
    }
    async fn apply(&self, params: HashMap<String, ScalingTriggerValueType>) {
        let metadata = self.scaling_trigger.metadata.clone();
        let region = metadata.get("region").unwrap();
        let region = Region::new(region.clone());
        let access_key = metadata.get("access_key").unwrap();
        let secret_key = metadata.get("secret_key").unwrap();
        let credentials = Credentials::new(
            access_key.clone(),
            secret_key.clone(),
            None,
            None,
            "wave-autoscale",
        );
        let shared_config = aws_config::from_env()
            .region(region)
            .credentials_provider(credentials)
            .load()
            .await;
        let client = Client::new(&shared_config);

        if let (
            Some(ScalingTriggerValueType::String(name)),
            Some(ScalingTriggerValueType::Int(min)),
            Some(ScalingTriggerValueType::Int(max)),
            Some(ScalingTriggerValueType::Int(desired)),
        ) = (
            params.get("name"),
            params.get("min"),
            params.get("max"),
            params.get("desired"),
        ) {
            let result = client
                .update_auto_scaling_group()
                .auto_scaling_group_name(name)
                .min_size(*min as i32)
                .max_size(*max as i32)
                .desired_capacity(*desired as i32)
                .send()
                .await;

            println!("{:?}", result);
            return;
        } else {
            panic!("Invalid params");
        }
    }
}
