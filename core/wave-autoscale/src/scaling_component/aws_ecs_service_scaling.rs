use super::ScalingComponent;
use crate::util::aws_region::get_aws_region_static_str;
use anyhow::{Ok, Result};
use async_trait::async_trait;
use data_layer::ScalingComponentDefinition;
use aws_sdk_ecs::{config::Credentials, error::ProvideErrorMetadata, Client};
use serde_json::{json, Value};
use std::collections::HashMap;

pub struct ECSServiceScalingComponent {
    definition: ScalingComponentDefinition,
}

impl ECSServiceScalingComponent {

    pub const SCALING_KIND: &'static str = "amazon-ecs";

    pub fn new(definition: ScalingComponentDefinition) -> Self {
        ECSServiceScalingComponent {
            definition
        }
    }
}

#[async_trait]
impl ScalingComponent for ECSServiceScalingComponent {
    fn get_scaling_component_kind(&self) -> &str {
        &self.definition.component_kind
    }
    fn get_id(&self) -> &str {
        &self.definition.id
    }
    async fn apply(&self, params: HashMap<String, Value>) -> Result<()> {
        let metadata: HashMap<String, Value> = self.definition.metadata.clone();
        if let (
            Some(Value::String(access_key)),
            Some(Value::String(secret_key)),
            Some(Value::String(region)),
            Some(Value::String(cluster_name)),
            Some(Value::String(service_name)),
            Some(desired),
        ) = (
            metadata.get("access_key"),
            metadata.get("secret_key"),
            metadata.get("region"),
            metadata.get("cluster_name"),
            metadata.get("service_name"),
            params.get("desired").and_then(Value::as_i64),
        ) {
            // TODO provider_name 도 meta data 로?!
            let credentials =
                Credentials::new(access_key, secret_key, None, None, "wave-autoscale-test");
            // aws_config needs a static region string
            let region_static: &'static str = get_aws_region_static_str(region);
            let shared_config = aws_config::from_env()
                .region(region_static)
                .credentials_provider(credentials)
                .load()
                .await;

            let client = Client::new(&shared_config);

            let result = client
                .update_service()
                .cluster(cluster_name)
                .service(service_name)
                .desired_count(desired as i32);

            let result = result.send().await;

            if result.is_err() {
                let error = result.err().unwrap();
                let meta = error.meta();
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


#[cfg(test)]
mod test {
    use data_layer::ScalingComponentDefinition;
    use std::collections::HashMap;
    use crate::scaling_component::ScalingComponent;
    use super::ECSServiceScalingComponent;


    // Purpose of the test is call apply function and fail test. just consists of test forms only.
    #[tokio::test]
    async fn apply_test() {

        let scaling_definition = ScalingComponentDefinition {
            kind: data_layer::types::object_kind::ObjectKind::ScalingComponent,
            db_id: String::from("db_id"),
            id: String::from("scaling-id"),
            component_kind: String::from("amazon-ecs"),
            metadata: HashMap::new()
        };

        let params = HashMap::new();
        let ecs_service_scaling_component = ECSServiceScalingComponent::new(scaling_definition).apply(params).await;
        assert!(ecs_service_scaling_component.is_err());
    }

}
