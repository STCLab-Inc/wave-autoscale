use super::ScalingComponent;
use crate::util::aws::get_aws_config;
use anyhow::{Ok, Result};
use async_trait::async_trait;
use aws_sdk_ecs::{error::ProvideErrorMetadata, Client};
use data_layer::ScalingComponentDefinition;
use serde_json::{json, Value};
use std::collections::HashMap;

pub struct ECSServiceScalingComponent {
    definition: ScalingComponentDefinition,
}

impl ECSServiceScalingComponent {
    pub const SCALING_KIND: &'static str = "amazon-ecs";

    pub fn new(definition: ScalingComponentDefinition) -> Self {
        ECSServiceScalingComponent { definition }
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
    async fn apply(
        &self,
        params: HashMap<String, Value>,
        _context: rquickjs::AsyncContext,
    ) -> Result<HashMap<String, Value>> {
        let metadata: HashMap<String, Value> = self.definition.metadata.clone();
        if let (
            Some(Value::String(region)),
            Some(Value::String(cluster_name)),
            Some(Value::String(service_name)),
            Some(desired),
        ) = (
            metadata.get("region"),
            metadata.get("cluster_name"),
            metadata.get("service_name"),
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

            let result = client
                .update_service()
                .cluster(cluster_name)
                .service(service_name)
                .desired_count(desired as i32);

            let result = result.send().await;

            if result.is_err() {
                let error = result.err().unwrap();
                let json = json!({
                    "message": error.message(),
                    "code": error.code(),
                    "extras": error.to_string()
                });

                return Err(anyhow::anyhow!(json));
            }

            Ok(params)
        } else {
            Err(anyhow::anyhow!("Invalid metadata"))
        }
    }
}

#[cfg(test)]
mod test {
    use super::ECSServiceScalingComponent;
    use crate::scaling_component::test::get_rquickjs_context;
    use crate::scaling_component::ScalingComponent;
    use data_layer::ScalingComponentDefinition;
    use std::collections::HashMap;

    // Purpose of the test is call apply function and fail test. just consists of test forms only.
    #[tokio::test]
    async fn apply_test() {
        let scaling_definition = ScalingComponentDefinition {
            kind: data_layer::types::object_kind::ObjectKind::ScalingComponent,
            db_id: String::from("db_id"),
            id: String::from("scaling-id"),
            component_kind: String::from("amazon-ecs"),
            metadata: HashMap::new(),
            ..Default::default()
        };

        let params = HashMap::new();
        let ecs_service_scaling_component = ECSServiceScalingComponent::new(scaling_definition)
            .apply(params, get_rquickjs_context().await)
            .await;
        assert!(ecs_service_scaling_component.is_err());
    }
}
