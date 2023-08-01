use super::super::util::google_cloud::google_cloud_functions_helper::{
    call_patch_google_cloud_functions, GoogleCloudFunctionsSetting,
};
use super::ScalingComponent;
use anyhow::{Ok, Result};
use async_trait::async_trait;
use data_layer::ScalingComponentDefinition;
use reqwest::Error;
use serde_json::{json, Value};
use std::collections::HashMap;

pub struct GoogleCloudFunctionsScalingComponent {
    definition: ScalingComponentDefinition,
}

impl GoogleCloudFunctionsScalingComponent {
    // Static variables
    pub const SCALING_KIND: &'static str = "google-cloud-functions";

    // Functions
    pub fn new(definition: ScalingComponentDefinition) -> Self {
        GoogleCloudFunctionsScalingComponent { definition }
    }
}

#[async_trait]
impl ScalingComponent for GoogleCloudFunctionsScalingComponent {
    fn get_scaling_component_kind(&self) -> &str {
        &self.definition.component_kind
    }

    fn get_id(&self) -> &str {
        &self.definition.id
    }

    async fn apply(&self, params: HashMap<String, Value>) -> Result<()> {
        let metadata: HashMap<String, Value> = self.definition.metadata.clone();

        if let (
            Some(Value::String(function_version)),
            Some(Value::String(project_name)),
            Some(Value::String(location_name)),
            Some(Value::String(function_name)),
            min_instances,
            max_instances,
            min_instance_count,
            max_instance_count,
            max_instance_request_concurrency,
        ) = (
            metadata.get("function_version"),
            metadata.get("project_name"),
            metadata.get("location_name"),
            metadata.get("function_name"),
            params
                .get("min_instances")
                .and_then(Value::as_i64)
                .map(|v| v as i32),
            params
                .get("max_instances")
                .and_then(Value::as_i64)
                .map(|v| v as i32),
            params
                .get("min_instance_count")
                .and_then(Value::as_i64)
                .map(|v| v as i32),
            params
                .get("max_instance_count")
                .and_then(Value::as_i64)
                .map(|v| v as i32),
            params
                .get("max_instance_request_concurrency")
                .and_then(Value::as_i64)
                .map(|v| v as i32),
        ) {
            // Helper function to add a field to payload json and query
            fn add_to_payload_and_query(
                field: &str,
                value: Option<i32>,
                query_str: &str,
                payload_json: &mut serde_json::Value,
                query: &mut Vec<String>,
            ) {
                if let Some(value) = value {
                    payload_json[field] = json!(value);
                    query.push(query_str.to_string());
                }
            }

            let mut payload_json = json!({});
            let mut query = Vec::new();

            match function_version.as_str() {
                "v1" => {
                    add_to_payload_and_query(
                        "minInstances",
                        min_instances,
                        "minInstances,",
                        &mut payload_json,
                        &mut query,
                    );
                    add_to_payload_and_query(
                        "maxInstances",
                        max_instances,
                        "maxInstances,",
                        &mut payload_json,
                        &mut query,
                    );
                }
                "v2" => {
                    let mut service_config = json!({});
                    add_to_payload_and_query(
                        "minInstanceCount",
                        min_instance_count,
                        "serviceConfig.minInstanceCount,",
                        &mut service_config,
                        &mut query,
                    );
                    add_to_payload_and_query(
                        "maxInstanceCount",
                        max_instance_count,
                        "serviceConfig.maxInstanceCount,",
                        &mut service_config,
                        &mut query,
                    );
                    add_to_payload_and_query(
                        "maxInstanceRequestConcurrency",
                        max_instance_request_concurrency,
                        "serviceConfig.maxInstanceRequestConcurrency,",
                        &mut service_config,
                        &mut query,
                    );

                    if let Some(obj) = service_config.as_object() {
                        if !obj.is_empty() {
                            payload_json["serviceConfig"] = service_config;
                        }
                    }
                }
                _ => {
                    return Err(anyhow::anyhow!("Invalid function version"));
                }
            }

            if query.is_empty() {
                return Err(anyhow::anyhow!("Invalid metadata"));
            } else {
                query.pop(); // Remove the trailing comma
                let google_cloud_functions_setting = GoogleCloudFunctionsSetting {
                    function_version: function_version.to_string(),
                    project_name: project_name.to_string(),
                    location_name: location_name.to_string(),
                    function_name: function_name.to_string(),
                    payload: Some(payload_json),
                    query: Some(vec![(String::from("updateMask"), query.join(""))]),
                };

                let result =
                    call_patch_google_cloud_functions(google_cloud_functions_setting).await;
                if result.is_err() {
                    return Err(anyhow::anyhow!(json!({
                        "message": "Google Cloud Functions API Call Error 1",
                        "code": "500",
                        "extras": result.unwrap_err().is_body().to_string()
                    })));
                }
                let result = result.unwrap();
                let result_status_code = result.status();
                let core::result::Result::Ok(result_body) = result.text().await else {
                    return Err(anyhow::anyhow!(json!({
                        "message": "Google Cloud Functions API Call Error 2",
                        "code": "500",
                        "extras": "Not found response text",
                    })));
                };
                if !result_status_code.is_success() {
                    return Err(anyhow::anyhow!(json!({
                        "message": "Google Cloud Functions API Call Error 3",
                        "code": result_status_code.as_str(),
                        "extras": result_body
                    })));
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
    use super::GoogleCloudFunctionsScalingComponent;
    use crate::scaling_component::ScalingComponent;
    use data_layer::ScalingComponentDefinition;
    use std::collections::HashMap;

    //#[ignore]
    #[tokio::test]
    async fn apply_call_patch_google_cloud_functions_for_version_1_function() {
        let metadata: HashMap<String, serde_json::Value> = vec![
            (String::from("function_version"), serde_json::json!("v1")),
            (
                String::from("project_name"),
                serde_json::json!("wave-autoscale-test"),
            ),
            (
                String::from("location_name"),
                serde_json::json!("asia-northeast2"),
            ),
            (
                String::from("function_name"),
                serde_json::json!("function-1"),
            ),
        ]
        .into_iter()
        .collect();
        let params: HashMap<String, serde_json::Value> = vec![
            (String::from("min_instances"), serde_json::json!(-2)),
            (String::from("max_instances"), serde_json::json!(20)),
        ]
        .into_iter()
        .collect();
        let scaling_definition = ScalingComponentDefinition {
            kind: data_layer::types::object_kind::ObjectKind::ScalingComponent,
            db_id: String::from("db-id"),
            id: String::from("scaling-id"),
            component_kind: String::from("google-cloud-functions"),
            metadata,
        };
        let google_cloud_functions_scaling_component: Result<(), anyhow::Error> =
            GoogleCloudFunctionsScalingComponent::new(scaling_definition)
                .apply(params)
                .await;

        println!(
            "google_cloud_functions_scaling_component: {:?}",
            google_cloud_functions_scaling_component
        );
        assert!(google_cloud_functions_scaling_component.is_ok());
    }
}
