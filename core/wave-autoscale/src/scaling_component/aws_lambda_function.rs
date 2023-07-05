use super::ScalingComponent;
use crate::util::aws_region::get_aws_region_static_str;
use anyhow::{Ok, Result};
use async_trait::async_trait;
use aws_sdk_lambda::{config::Credentials, error::ProvideErrorMetadata, Client};
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
      Some(Value::String(access_key)),
      Some(Value::String(secret_key)),
      Some(Value::String(region)),
      Some(Value::String(function_name)),
      reserved_concurrency,
      provisioned_concurrency,
    ) = (
      metadata.get("access_key"),
      metadata.get("secret_key"),
      metadata.get("region"),
      metadata.get("function_name"),
      params.get("reserved_concurrency").and_then(Value::as_i64).map(|v| v as i32),
      params.get("provisioned_concurrency").and_then(Value::as_i64).map(|v| v as i32),
    ) {
      let credentials = Credentials::new(access_key, secret_key, None, None, "wave-autoscale");
      // aws_config needs a static region string
      let region_static: &'static str = get_aws_region_static_str(region);
      let shared_config = aws_config::from_env().region(region_static).credentials_provider(credentials).load().await;
      
      let client = Client::new(&shared_config);
      
      // Set and put reserved concurrency if provided.      
      if let Some(reserved_concurrency) = reserved_concurrency {
        let result = client.put_function_concurrency().function_name(function_name).reserved_concurrent_executions(reserved_concurrency);
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
        let qualifier = metadata.get("qualifier").and_then(Value::as_str).map(String::from).unwrap_or_default();
        // Error occurs when qualifier is not provided or assigned to empty string.
        let result = client.put_provisioned_concurrency_config().function_name(function_name).provisioned_concurrent_executions(provisioned_concurrency).qualifier(qualifier);
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