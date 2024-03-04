use super::super::util::google_cloud::google_cloud_functions_instance_helper::{
    call_patch_cloud_functions_instance, CloudFunctionsPatchInstanceSetting,
};
use super::ScalingComponent;
use anyhow::{Ok, Result};
use async_trait::async_trait;
use data_layer::ScalingComponentDefinition;

use std::collections::HashMap;

pub struct CloudFunctionsInstanceScalingComponent {
    definition: ScalingComponentDefinition,
}

impl CloudFunctionsInstanceScalingComponent {
    // Static variables
    pub const SCALING_KIND: &'static str = "google-cloud-functions";

    // Functions
    pub fn new(definition: ScalingComponentDefinition) -> Self {
        CloudFunctionsInstanceScalingComponent { definition }
    }
}

#[async_trait]
impl ScalingComponent for CloudFunctionsInstanceScalingComponent {
    fn get_scaling_component_kind(&self) -> &str {
        &self.definition.component_kind
    }

    fn get_id(&self) -> &str {
        &self.definition.id
    }

    async fn apply(
        &self,
        params: HashMap<String, serde_json::Value>,
        _context: rquickjs::AsyncContext,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let metadata: HashMap<String, serde_json::Value> = self.definition.metadata.clone();

        if let (
            Some(serde_json::Value::String(function_version)),
            Some(serde_json::Value::String(project_name)),
            Some(serde_json::Value::String(location_name)),
            Some(serde_json::Value::String(function_name)),
            min_instance_count,
            max_instance_count,
            max_request_per_instance,
        ) = (
            metadata.get("function_version"),
            metadata.get("project_name"),
            metadata.get("location_name"),
            metadata.get("function_name"),
            params
                .get("min_instance_count")
                .and_then(serde_json::Value::as_i64)
                .map(|v| v as i32),
            params
                .get("max_instance_count")
                .and_then(serde_json::Value::as_i64)
                .map(|v| v as i32),
            params
                .get("max_request_per_instance")
                .and_then(serde_json::Value::as_i64)
                .map(|v| v as i32),
        ) {
            let mut payload_json = serde_json::json!({});
            let mut query = Vec::new();

            match function_version.as_str() {
                "v1" => {
                    add_to_payload_and_query(
                        "minInstances",
                        min_instance_count,
                        "minInstances,",
                        &mut payload_json,
                        &mut query,
                    );
                    add_to_payload_and_query(
                        "maxInstances",
                        max_instance_count,
                        "maxInstances,",
                        &mut payload_json,
                        &mut query,
                    );
                }
                "v2" => {
                    let mut service_config = serde_json::json!({});
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
                        max_request_per_instance,
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
            }

            query.pop(); // Remove the trailing comma
            let cloud_functions_instance_setting = CloudFunctionsPatchInstanceSetting {
                function_version: function_version.to_string(),
                project_name: project_name.to_string(),
                location_name: location_name.to_string(),
                function_name: function_name.to_string(),
                payload: Some(payload_json),
                query: Some(vec![(String::from("updateMask"), query.join(""))]),
            };

            let result =
                call_patch_cloud_functions_instance(cloud_functions_instance_setting).await;
            if result.is_err() {
                return Err(anyhow::anyhow!(serde_json::json!({
                    "message": "API call error",
                    "code": "500",
                    "extras": result.unwrap_err().is_body().to_string()
                })));
            }
            let result = result.unwrap();
            let result_status_code = result.status();
            let result_body: String = match result.text().await {
                core::result::Result::Ok(result_body) => result_body,
                Err(_error) => {
                    return Err(anyhow::anyhow!(serde_json::json!({
                        "message": "API call error",
                        "code": "500",
                        "extras": "Not found response text",
                    })));
                }
            };
            if !result_status_code.is_success() {
                tracing::error!("API call error: {:?}", &result_body);
                let json = serde_json::json!({
                    "message": "API call error",
                    "code": result_status_code.as_str(),
                    "extras": result_body
                });
                return Err(anyhow::anyhow!(json));
            }

            Ok(params)
        } else {
            Err(anyhow::anyhow!("Invalid metadata"))
        }
    }
}

// Helper function to add a field to payload json and query
fn add_to_payload_and_query(
    field: &str,
    value: Option<i32>,
    query_str: &str,
    payload_json: &mut serde_json::Value,
    query: &mut Vec<String>,
) {
    if let Some(value) = value {
        payload_json[field] = serde_json::json!(value);
        query.push(query_str.to_string());
    }
}

#[cfg(test)]
mod test {
    use super::CloudFunctionsInstanceScalingComponent;
    use crate::scaling_component::test::get_rquickjs_context;
    use crate::scaling_component::ScalingComponent;
    use data_layer::ScalingComponentDefinition;
    use std::collections::HashMap;

    #[ignore]
    #[tokio::test]
    async fn apply_call_patch_cloud_functions_instance_for_version_1_function() {
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
            (String::from("min_instance_count"), serde_json::json!(4)),
            (String::from("max_instance_count"), serde_json::json!(5)),
        ]
        .into_iter()
        .collect();
        let scaling_definition = ScalingComponentDefinition {
            kind: data_layer::types::object_kind::ObjectKind::ScalingComponent,
            db_id: String::from("db-id"),
            id: String::from("scaling-id"),
            component_kind: String::from("google-cloud-functions"),
            metadata,
            ..Default::default()
        };
        let cloud_functions_instance_scaling_component: Result<
            HashMap<String, serde_json::Value>,
            anyhow::Error,
        > = CloudFunctionsInstanceScalingComponent::new(scaling_definition)
            .apply(params, get_rquickjs_context().await)
            .await;

        assert!(cloud_functions_instance_scaling_component.is_ok());
    }

    #[ignore]
    #[tokio::test]
    async fn apply_call_patch_cloud_functions_instance_for_version_2_function() {
        let metadata: HashMap<String, serde_json::Value> = vec![
            (String::from("function_version"), serde_json::json!("v2")),
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
                serde_json::json!("function-2"),
            ),
        ]
        .into_iter()
        .collect();
        let params: HashMap<String, serde_json::Value> = vec![
            (String::from("min_instance_count"), serde_json::json!(5)),
            (String::from("max_instance_count"), serde_json::json!(8)),
            (
                String::from("max_request_per_instance"),
                serde_json::json!(3),
            ),
        ]
        .into_iter()
        .collect();
        let scaling_definition = ScalingComponentDefinition {
            kind: data_layer::types::object_kind::ObjectKind::ScalingComponent,
            db_id: String::from("db-id"),
            id: String::from("scaling-id"),
            component_kind: String::from("google-cloud-functions"),
            metadata,
            ..Default::default()
        };
        let cloud_functions_instance_scaling_component: Result<
            HashMap<String, serde_json::Value>,
            anyhow::Error,
        > = CloudFunctionsInstanceScalingComponent::new(scaling_definition)
            .apply(params, get_rquickjs_context().await)
            .await;

        assert!(cloud_functions_instance_scaling_component.is_ok());
    }

    #[ignore]
    #[tokio::test]
    async fn apply_error_call_patch_cloud_functions_instance_for_version_1_function() {
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
                serde_json::json!("function-2"),
            ),
        ]
        .into_iter()
        .collect();
        let params: HashMap<String, serde_json::Value> = vec![
            (String::from("min_instance_count"), serde_json::json!(2)),
            (String::from("max_instance_count"), serde_json::json!(5)),
        ]
        .into_iter()
        .collect();
        let scaling_definition = ScalingComponentDefinition {
            kind: data_layer::types::object_kind::ObjectKind::ScalingComponent,
            db_id: String::from("db-id"),
            id: String::from("scaling-id"),
            component_kind: String::from("google-cloud-functions"),
            metadata,
            ..Default::default()
        };
        let cloud_functions_instance_scaling_component: Result<
            HashMap<String, serde_json::Value>,
            anyhow::Error,
        > = CloudFunctionsInstanceScalingComponent::new(scaling_definition)
            .apply(params, get_rquickjs_context().await)
            .await;

        assert!(cloud_functions_instance_scaling_component.is_err());
    }

    #[ignore]
    #[tokio::test]
    async fn apply_error_call_patch_cloud_functions_instance_for_version_2_function() {
        let metadata: HashMap<String, serde_json::Value> = vec![
            (String::from("function_version"), serde_json::json!("v2")),
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
                serde_json::json!("function-2"),
            ),
        ]
        .into_iter()
        .collect();
        let params: HashMap<String, serde_json::Value> = vec![
            (String::from("min_instance_count"), serde_json::json!(-5)),
            (String::from("max_instance_count"), serde_json::json!(-8)),
            (
                String::from("max_request_per_instance"),
                serde_json::json!(-3),
            ),
        ]
        .into_iter()
        .collect();
        let scaling_definition = ScalingComponentDefinition {
            kind: data_layer::types::object_kind::ObjectKind::ScalingComponent,
            db_id: String::from("db-id"),
            id: String::from("scaling-id"),
            component_kind: String::from("google-cloud-functions"),
            metadata,
            ..Default::default()
        };
        let cloud_functions_instance_scaling_component: Result<
            HashMap<String, serde_json::Value>,
            anyhow::Error,
        > = CloudFunctionsInstanceScalingComponent::new(scaling_definition)
            .apply(params, get_rquickjs_context().await)
            .await;

        assert!(cloud_functions_instance_scaling_component.is_err());
    }
}
