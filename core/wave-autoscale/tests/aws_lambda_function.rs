#[cfg(test)]
mod aws_lambda_function_test {
  use std::collections::HashMap;
  
  use anyhow::Result;
  use data_layer::reader::yaml_reader::read_yaml_file;
  use serde_json::{json, Value};
  
  use aws_sdk_lambda::{config::Credentials, error::ProvideErrorMetadata, Client};
  
  const LAMBDA_FUNCTION_FILE_PATH: &str = "./tests/yaml/component_lambda_function.yaml";  
  static STATIC_REGION: &str = "ap-northeast-3";
  
  #[tokio::test]
  async fn aws_lambda_function_update() -> Result<()> {
    
    // The unreserved account concurrency can't go below 100.
    // You can't set reserved concurrency below the provisioned concurrency that is requested for this function.
    let reserved_concurrency: Option<i32> = Some(2);

    // The maximum allowed provisioned concurrency is (i - j), based on the function's reserved concurrency (i) minus the provisioned concurrency on other versions (j).
    // The minimum provisioned concurrency value allowed is 1.
    let provisioned_concurrency: Option<i32> = Some(1);
    
    let parse_result = read_yaml_file(LAMBDA_FUNCTION_FILE_PATH).unwrap();
    
    for scaling_component_definition in parse_result.scaling_component_definitions.clone() {      
      let metadata: HashMap<String, Value> = scaling_component_definition.metadata;
      
      if let (        
        Some(Value::String(access_key)),
        Some(Value::String(secret_key)),
        Some(Value::String(_region)),
        Some(Value::String(function_name)),
      ) = (        
        metadata.get("access_key"),
        metadata.get("secret_key"),
        metadata.get("region"),
        metadata.get("function_name"),
      ) {
        let provider_name = "wave-autoscale";
        let credentials = Credentials::new(access_key, secret_key, None, None, provider_name);
        let shared_config = aws_config::from_env().region(STATIC_REGION).credentials_provider(credentials).load().await;
        let client = Client::new(&shared_config);
        
        // Set and put reserved concurrency if provided.
        if let Some(reserved_concurrency) = reserved_concurrency {
          let result = client.put_function_concurrency().function_name(function_name).reserved_concurrent_executions(reserved_concurrency);
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
              "message": meta.message(),
              "code": meta.code(),
              "extras": meta.to_string()
            });
            return Err(anyhow::anyhow!(json));
          }
        }
      } else {
        return Err(anyhow::anyhow!("Missing required metadata"));
      }
    }
    Ok(())
  }
}