use super::ScalingComponent;
use crate::util::aws::get_aws_config;
use anyhow::{Ok, Result};
use async_trait::async_trait;

use aws_smithy_types::error::metadata::ProvideErrorMetadata;

use aws_sdk_lambda::Client as LambdaClient;

use data_layer::ScalingComponentDefinition;
use serde_json::{json, Value};
use std::collections::HashMap;

pub struct LambdaFunctionScalingComponent {
    definition: ScalingComponentDefinition,
}

impl LambdaFunctionScalingComponent {
    // Static variables
    pub const SCALING_KIND: &'static str = "aws-lambda";

    // Functions
    pub fn new(definition: ScalingComponentDefinition) -> Self {
        LambdaFunctionScalingComponent { definition }
    }
}

#[async_trait]
impl ScalingComponent for LambdaFunctionScalingComponent {
    fn get_scaling_component_kind(&self) -> &str {
        &self.definition.component_kind
    }
    fn get_id(&self) -> &str {
        &self.definition.id
    }
    async fn apply(&self, params: HashMap<String, Value>) -> Result<()> {
        let metadata: HashMap<String, Value> = self.definition.metadata.clone();

        if let (
            Some(Value::String(region)),
            Some(Value::String(function_name)),
            reserved_concurrency,
            provisioned_concurrency,
        ) = (
            metadata.get("region"),
            metadata.get("function_name"),
            params
                .get("reserved_concurrency")
                .and_then(Value::as_i64)
                .map(|v| v as i32),
            params
                .get("provisioned_concurrency")
                .and_then(Value::as_i64)
                .map(|v| v as i32),
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
            let shared_config = config.unwrap();

            let client = LambdaClient::new(&shared_config);

            // Set and put reserved concurrency if provided.
            if let Some(reserved_concurrency) = reserved_concurrency {
                let result = client
                    .put_function_concurrency()
                    .function_name(function_name)
                    .reserved_concurrent_executions(reserved_concurrency);
                let result = result.send().await;
                if result.is_err() {
                    let error = result.err().unwrap();
                    let meta = error.meta();
                    let json = json!({
                      "message": meta.message().unwrap_or(&error.to_string()),
                      "code": meta.code(),
                      "extras": meta.to_string()
                    });
                    return Err(anyhow::anyhow!(json));
                }
            }

            // Set and put provisioned concurrency if provided regardless of qualifier.
            if let Some(provisioned_concurrency) = provisioned_concurrency {
                // Set qualifier if provided in meta data.
                let qualifier = metadata
                    .get("qualifier")
                    .and_then(Value::as_str)
                    .map(String::from)
                    .unwrap_or_default();
                // Error occurs when qualifier is not provided or assigned to empty string.
                let result = client
                    .put_provisioned_concurrency_config()
                    .function_name(function_name)
                    .provisioned_concurrent_executions(provisioned_concurrency)
                    .qualifier(qualifier);
                let result = result.send().await;
                if result.is_err() {
                    let error = result.err().unwrap();
                    let meta = error.meta();
                    let json = json!({
                      "message": meta.message().unwrap_or(&error.to_string()),
                      "code": meta.code(),
                      "extras": meta.to_string()
                    });
                    return Err(anyhow::anyhow!(json));
                }
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("Invalid metadata"))
        }
    }
}

#[cfg(test)]
mod test {
    use super::LambdaFunctionScalingComponent;
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
            component_kind: String::from("aws-lambda"),
            metadata: HashMap::new(),
            ..Default::default()
        };

        let params = HashMap::new();
        let lambda_function_scaling_component =
            LambdaFunctionScalingComponent::new(scaling_definition)
                .apply(params)
                .await;
        assert!(lambda_function_scaling_component.is_err());
    }
}
