#[cfg(test)]
mod aws_ecs_service_scaling_test {
    use anyhow::{Ok, Result};
    use aws_sdk_ecs::{config::Credentials, error::ProvideErrorMetadata, Client};
    use data_layer::reader::wave_definition_reader;
    use serde_json::{json, Value};
    use std::collections::HashMap;

    const COMPONENT_ECS_SERVICE_SCALING_FILE_PATH: &str =
        "./tests/yaml/component_ecs_service_scaling.yaml";
    static STATIC_REGION: &str = "ap-northeast-3";

    #[tokio::test]
    #[ignore]
    async fn aws_ecs_api_service_desired_count_update_test() -> Result<()> {
        let desired_count = 2;
        let parse_result = wave_definition_reader::read_definition_yaml_file(
            COMPONENT_ECS_SERVICE_SCALING_FILE_PATH,
        )
        .unwrap();

        for scaling_component_definition in parse_result.scaling_component_definitions.clone() {
            let metadata: HashMap<String, Value> = scaling_component_definition.metadata;

            if let (
                Some(Value::String(access_key)),
                Some(Value::String(secret_key)),
                Some(Value::String(_region)),
                Some(Value::String(cluster_name)),
                Some(Value::String(service_name)),
            ) = (
                metadata.get("access_key"),
                metadata.get("secret_key"),
                metadata.get("region"),
                metadata.get("cluster_name"),
                metadata.get("service_name"),
            ) {
                // TODO Add provider_name as meta data (?)
                let provider_name = "wave-autoscale-test";

                let credentials =
                    Credentials::new(access_key, secret_key, None, None, provider_name);
                let shared_config = aws_config::from_env()
                    .region(STATIC_REGION)
                    .credentials_provider(credentials)
                    .load()
                    .await;

                let client = Client::new(&shared_config);

                let result = client
                    .update_service()
                    .cluster(cluster_name)
                    .service(service_name)
                    .desired_count(desired_count);

                let result = result.send().await;

                if result.is_err() {
                    let error = result.err().unwrap();
                    let meta = error.meta();
                    let result_json = json!({
                        "message": meta.message(),
                        "code": meta.code(),
                        "extras": meta.to_string()
                    });

                    return Err(anyhow::anyhow!(result_json));
                } else {
                    assert!(result.is_ok());
                }
            }
        }

        Ok(())
    }
}
