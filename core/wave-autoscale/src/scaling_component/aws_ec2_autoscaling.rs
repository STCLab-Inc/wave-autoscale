use super::ScalingComponent;
use super::{evaluate_expression_with_current_state, filter_current_state_in_expression};
use crate::util::aws::get_aws_config;
use anyhow::{Ok, Result};
use async_trait::async_trait;
use aws_sdk_autoscaling::{error::ProvideErrorMetadata, Client};
use data_layer::ScalingComponentDefinition;
use serde_json::{json, Value};
use std::collections::HashMap;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

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

#[derive(Debug, EnumIter)]
enum EC2ComponentTargetValue {
    Desired,
    Min,
    Max,
}
impl std::fmt::Display for EC2ComponentTargetValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EC2ComponentTargetValue::Desired => write!(f, "desired"),
            EC2ComponentTargetValue::Min => write!(f, "min"),
            EC2ComponentTargetValue::Max => write!(f, "max"),
        }
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
            Some(Value::String(region)),
            Some(Value::String(desired)),
        ) = (
            metadata.get("asg_name"),
            metadata.get("region"),
            params.get("desired"),
        ) {
            // AWS Credentials
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

            let current_state_key_array = EC2ComponentTargetValue::iter()
                .map(|value| value.to_string())
                .collect::<Vec<String>>();
            // check target value contains enum variables
            let current_state_array =
                filter_current_state_in_expression(desired, current_state_key_array);
            // save target value to map
            let current_state_map =
                get_current_state_map(current_state_array, client.clone(), asg_name.clone()).await;
            if current_state_map.is_err() {
                return Err(current_state_map.unwrap_err());
            };

            // evaluate target value
            let desired =
                evaluate_expression_with_current_state(desired, current_state_map.unwrap().clone())
                    .await;
            if desired.is_err() {
                return Err(desired.unwrap_err());
            }
            let desired = desired.unwrap();

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

async fn get_current_state_map(
    current_state_array: Vec<String>,
    client: Client,
    asg_name: String,
) -> Result<HashMap<String, i64>, anyhow::Error> {
    let mut current_state_map: HashMap<String, i64> = HashMap::new();
    for current_state in current_state_array {
        let mut current_state_kind = EC2ComponentTargetValue::Desired;
        if current_state.eq(&format!("${}", EC2ComponentTargetValue::Desired)) {
            current_state_kind = EC2ComponentTargetValue::Desired;
        } else if current_state.eq(&format!("${}", EC2ComponentTargetValue::Min)) {
            current_state_kind = EC2ComponentTargetValue::Min;
        } else if current_state.eq(&format!("${}", EC2ComponentTargetValue::Max)) {
            current_state_kind = EC2ComponentTargetValue::Max;
        }

        let Some(desired_capacity) = get_auto_scaling_group_capacity(
            client.clone(),
            asg_name.clone(),
            current_state_kind
        )
        .await
            else {
            return Err(anyhow::anyhow!(
                "Failed to get auto scaling group capacity"
            ));
        };
        current_state_map.insert(current_state.clone(), desired_capacity as i64);
    }
    Ok(current_state_map)
}

async fn get_auto_scaling_group_capacity(
    client: Client,
    asg_name: String,
    kind: EC2ComponentTargetValue,
) -> Option<i32> {
    let describe_auto_scaling_groups = client
        .describe_auto_scaling_groups()
        .auto_scaling_group_names(asg_name.clone())
        .send()
        .await;
    if describe_auto_scaling_groups.is_err() {
        return None;
    }
    let describe_groups = describe_auto_scaling_groups.unwrap();
    let Some(group) = describe_groups.auto_scaling_groups() else {
        return None;
    };
    let group = group[0].clone();
    match kind {
        EC2ComponentTargetValue::Desired => group.desired_capacity(),
        EC2ComponentTargetValue::Min => group.min_size(),
        EC2ComponentTargetValue::Max => group.max_size(),
    }
}

#[cfg(test)]
mod test {
    use super::super::ScalingComponentManager;
    use super::*;
    use data_layer::types::object_kind::ObjectKind;
    use serde_json::{json, Value};
    use std::collections::HashMap;

    fn get_data() -> (String, String) {
        (
            "ap-northeast-3".to_string(),    // region
            "wave-ec2-as-nginx".to_string(), // asg_name
        )
    }

    #[ignore]
    #[tokio::test]
    async fn test_get_auto_scaling_group_desired_capacity() {
        let config = get_aws_config(Some(get_data().0), None, None, None, None).await;
        let client = Client::new(&config.unwrap());
        let desired_capacity =
            get_auto_scaling_group_capacity(client, get_data().1, EC2ComponentTargetValue::Desired);
        assert!(desired_capacity.await.unwrap() > 0);
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

        let mut params: HashMap<String, Value> = HashMap::new();
        params.insert("desired".to_string(), json!(2));

        if let (Some(Value::String(_asg_name)), Some(Value::String(_region)), Some(_desired)) = (
            metadata.get("asg_name"),
            metadata.get("region"),
            params.get("desired").and_then(Value::as_i64),
        ) {
            let access_key = metadata.get("access_key");
            let secret_key = metadata.get("secret_key");
            assert_eq!(access_key, None);
            assert_eq!(secret_key, None);
        } else {
            assert!(false);
        }
    }

    #[ignore]
    #[tokio::test]
    async fn test_get_current_state_map() {
        let config = get_aws_config(Some(get_data().0), None, None, None, None).await;
        let client = Client::new(&config.unwrap());
        let map = get_current_state_map(
            vec!["$min".to_string(), "$desired".to_string()],
            client,
            get_data().1,
        )
        .await
        .unwrap();
        assert!(map.contains_key("$min"));
        assert!(map.contains_key("$desired"));
    }

    #[ignore]
    #[tokio::test]
    async fn test_aws_ec2_autoscaling() {
        let mut scaling_component_metadata = HashMap::new();
        scaling_component_metadata.insert(
            "region".to_string(),
            serde_json::Value::String(get_data().0),
        );
        scaling_component_metadata.insert(
            "asg_name".to_string(),
            serde_json::Value::String(get_data().1),
        );

        let scaling_component_definitions = vec![ScalingComponentDefinition {
            kind: ObjectKind::ScalingComponent,
            db_id: "".to_string(),
            id: "api_server".to_string(),
            component_kind: "aws-ec2-autoscaling".to_string(),
            metadata: scaling_component_metadata,
        }];

        // create metric adapter
        let mut scaling_component_manager = ScalingComponentManager::new();
        scaling_component_manager.add_definitions(scaling_component_definitions);

        // run scaling trigger
        let mut options: HashMap<String, serde_json::Value> = HashMap::new();
        options.insert(
            "desired".to_string(),
            json!("$desired * $min + 1".to_string()),
        );

        let result = scaling_component_manager
            .apply_to("api_server", options)
            .await;
        assert!(result.is_ok());
    }
}
