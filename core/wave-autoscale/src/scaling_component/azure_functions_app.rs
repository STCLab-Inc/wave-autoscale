use super::super::util::azure::{
    azure_funtions_app_helper::{call_patch_azure_functions_app, AzureFunctionsPatchAppSetting},
    AzureCredential,
};
use super::ScalingComponent;
use anyhow::{Ok, Result};
use async_trait::async_trait;
use data_layer::ScalingComponentDefinition;

use std::collections::HashMap;

pub struct AzureFunctionsAppScalingComponent {
    definition: ScalingComponentDefinition,
}

impl AzureFunctionsAppScalingComponent {
    // Static variables
    pub const SCALING_KIND: &'static str = "azure-functions";

    // Functions
    pub fn new(definition: ScalingComponentDefinition) -> Self {
        AzureFunctionsAppScalingComponent { definition }
    }
}

#[async_trait]
impl ScalingComponent for AzureFunctionsAppScalingComponent {
    fn get_scaling_component_kind(&self) -> &str {
        &self.definition.component_kind
    }

    fn get_id(&self) -> &str {
        &self.definition.id
    }

    async fn apply(&self, params: HashMap<String, serde_json::Value>) -> Result<()> {
        let metadata: HashMap<String, serde_json::Value> = self.definition.metadata.clone();

        if let (
            Some(serde_json::Value::String(subscription_id)),
            Some(serde_json::Value::String(resource_group_name)),
            Some(serde_json::Value::String(app_name)),
            min_instance_count,
            max_instance_count,
        ) = (
            metadata.get("subscription_id"),
            metadata.get("resource_group_name"),
            metadata.get("app_name"),
            params
                .get("min_instance_count")
                .and_then(serde_json::Value::as_u64)
                .map(|v| v as u32),
            params
                .get("max_instance_count")
                .and_then(serde_json::Value::as_u64)
                .map(|v| v as u32),
        ) {
            let client_id = metadata
                .get("client_id")
                .map(|client_id| client_id.to_string());
            let client_secret = metadata
                .get("client_secret")
                .map(|client_secret| client_secret.to_string());
            let tenant_id = metadata
                .get("tenant_id")
                .map(|tenant_id| tenant_id.to_string());

            // Call patch azure functions app api
            let azure_credential = AzureCredential {
                client_id,
                client_secret,
                tenant_id,
            };
            let azure_functions_app_setting = AzureFunctionsPatchAppSetting {
                azure_credential: azure_credential.clone(),
                subscription_id: subscription_id.to_string(),
                resource_group_name: resource_group_name.to_string(),
                app_name: app_name.to_string(),
                payload: Some(create_payload(
                    min_instance_count
                        .map(|value| value.to_string())
                        .as_deref()
                        .or(None),
                    max_instance_count
                        .map(|value| value.to_string())
                        .as_deref()
                        .or(None),
                )),
            };
            let result = call_patch_azure_functions_app(azure_functions_app_setting).await;
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

            Ok(())
        } else {
            Err(anyhow::anyhow!("Invalid metadata"))
        }
    }
}

// Create payload to patch azure functions app api
fn create_payload(
    min_instance_count: Option<&str>,
    max_instance_count: Option<&str>,
) -> serde_json::Value {
    match (min_instance_count, max_instance_count) {
        (Some(min_instance_count), Some(max_instance_count)) => {
            serde_json::json!({
                "properties": {
                    "siteConfig": {
                        "functionAppScaleLimit": max_instance_count,
                        "minimumElasticInstanceCount": min_instance_count
                    }
                }
            })
        }
        (_, Some(max_instance_count)) => {
            serde_json::json!({
                "properties": {
                    "siteConfig": {
                        "functionAppScaleLimit": max_instance_count
                    }
                }
            })
        }
        _ => serde_json::Value::Null,
    }
}

#[cfg(test)]
mod test {
    use super::AzureFunctionsAppScalingComponent;
    use crate::scaling_component::ScalingComponent;
    use data_layer::ScalingComponentDefinition;
    use std::collections::HashMap;

    #[ignore]
    #[tokio::test]
    async fn apply_call_patch_azure_functions_app_for_consumption_plan() {
        let metadata: HashMap<String, serde_json::Value> = vec![
            (String::from("client_id"), serde_json::json!("CLIENT_ID")),
            (
                String::from("client_secret"),
                serde_json::json!("CLIENT_SECRET"),
            ),
            (String::from("tenant_id"), serde_json::json!("TENANT_ID")),
            (
                String::from("subscription_id"),
                serde_json::json!("SUBSCRIPTION_ID"),
            ),
            (
                String::from("resource_group_name"),
                serde_json::json!("RESOURCE_GROUP_NAME"),
            ),
            (String::from("app_name"), serde_json::json!("APP_NAME")),
        ]
        .into_iter()
        .collect();
        let params: HashMap<String, serde_json::Value> =
            vec![(String::from("max_instance_count"), serde_json::json!(5))]
                .into_iter()
                .collect();
        let scaling_definition = ScalingComponentDefinition {
            kind: data_layer::types::object_kind::ObjectKind::ScalingComponent,
            db_id: String::from("db-id"),
            id: String::from("scaling-id"),
            component_kind: String::from("azure-functions"),
            metadata,
        };
        let azure_functions_app_scaling_component: Result<(), anyhow::Error> =
            AzureFunctionsAppScalingComponent::new(scaling_definition)
                .apply(params)
                .await;

        assert!(azure_functions_app_scaling_component.is_ok());
    }

    #[ignore]
    #[tokio::test]
    async fn apply_call_patch_azure_functions_app_for_premium_plan() {
        let metadata: HashMap<String, serde_json::Value> = vec![
            (String::from("client_id"), serde_json::json!("CLIENT_ID")),
            (
                String::from("client_secret"),
                serde_json::json!("CLIENT_SECRET"),
            ),
            (String::from("tenant_id"), serde_json::json!("TENANT_ID")),
            (
                String::from("subscription_id"),
                serde_json::json!("SUBSCRIPTION_ID"),
            ),
            (
                String::from("resource_group_name"),
                serde_json::json!("RESOURCE_GROUP_NAME"),
            ),
            (String::from("app_name"), serde_json::json!("APP_NAME")),
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
            component_kind: String::from("azure-functions"),
            metadata,
        };
        let azure_functions_app_scaling_component: Result<(), anyhow::Error> =
            AzureFunctionsAppScalingComponent::new(scaling_definition)
                .apply(params)
                .await;

        assert!(azure_functions_app_scaling_component.is_ok());
    }

    #[ignore]
    #[tokio::test]
    async fn apply_error_call_patch_azure_functions_app_for_consumption_plan() {
        let metadata: HashMap<String, serde_json::Value> = vec![
            (String::from("client_id"), serde_json::json!("CLIENT_ID")),
            (
                String::from("client_secret"),
                serde_json::json!("CLIENT_SECRET"),
            ),
            (String::from("tenant_id"), serde_json::json!("TENANT_ID")),
            (
                String::from("subscription_id"),
                serde_json::json!("SUBSCRIPTION_ID"),
            ),
            (
                String::from("resource_group_name"),
                serde_json::json!("RESOURCE_GROUP_NAME"),
            ),
            (String::from("app_name"), serde_json::json!("APP_NAME")),
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
            component_kind: String::from("azure-functions"),
            metadata,
        };
        let azure_functions_app_scaling_component: Result<(), anyhow::Error> =
            AzureFunctionsAppScalingComponent::new(scaling_definition)
                .apply(params)
                .await;

        assert!(azure_functions_app_scaling_component.is_ok());
    }
}
