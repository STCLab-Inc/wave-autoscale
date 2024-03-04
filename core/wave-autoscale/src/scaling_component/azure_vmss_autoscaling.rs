use super::super::util::azure::{
    azure_autoscale_settings_helper::{
        call_azure_get_autoscale_settings_list_by_resource_group,
        call_azure_patch_autoscale_settings_update, AzureAutoscaleSetting,
    },
    azure_virtual_machine_scale_sets::{
        call_azure_patch_virtual_machine_scale_sets_capacity, AzureVmssSetting,
    },
    AzureCredential,
};
use super::ScalingComponent;
use async_trait::async_trait;
use data_layer::ScalingComponentDefinition;
use std::collections::HashMap;
use tracing::error;

pub struct VMSSAutoScalingComponent {
    definition: ScalingComponentDefinition,
}

impl VMSSAutoScalingComponent {
    pub const SCALING_KIND: &'static str = "azure-virtual-machine-scale-sets";

    pub fn new(definition: ScalingComponentDefinition) -> Self {
        VMSSAutoScalingComponent { definition }
    }
}

#[async_trait]
impl ScalingComponent for VMSSAutoScalingComponent {
    fn get_scaling_component_kind(&self) -> &str {
        &self.definition.component_kind
    }
    fn get_id(&self) -> &str {
        &self.definition.id
    }

    async fn apply(
        &self,
        params: HashMap<String, serde_json::Value>,
        context: rquickjs::AsyncContext,
    ) -> anyhow::Result<HashMap<String, serde_json::Value>> {
        let metadata: HashMap<String, serde_json::Value> = self.definition.metadata.clone();
        if let (
            Some(serde_json::Value::String(subscription_id)),
            Some(serde_json::Value::String(resource_group_name)),
            Some(serde_json::Value::String(vm_scale_set_name)),
            Some(capacity),
        ) = (
            metadata.get("subscription_id"),
            metadata.get("resource_group_name"),
            metadata.get("vm_scale_set_name"),
            params.get("capacity").and_then(serde_json::Value::as_u64),
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

            let azure_credential = AzureCredential {
                client_id,
                client_secret,
                tenant_id,
            };
            let azure_vmss_setting = AzureVmssSetting {
                azure_credential: azure_credential.clone(),
                subscription_id: subscription_id.to_string(),
                resource_group_name: resource_group_name.to_string(),
                vm_scale_set_name: Some(vm_scale_set_name.to_string()),
                payload: Some(serde_json::json!({
                    "sku": {
                        "capacity": capacity
                    },
                })),
            };
            let azure_autoscale_setting = AzureAutoscaleSetting {
                azure_credential: azure_credential.clone(),
                subscription_id: subscription_id.to_string(),
                resource_group_name: resource_group_name.to_string(),
                autoscale_setting_name: None,
                payload: None,
            };
            let vmss_resource_id = format!("/subscriptions/{subscriptionId}/resourceGroups/{resourceGroupName}/providers/Microsoft.Compute/virtualMachineScaleSets/{vmScaleSetName}",
                subscriptionId = subscription_id,
                resourceGroupName = resource_group_name,
                vmScaleSetName = vm_scale_set_name
            );

            // check autoscale setting enabled
            let azure_autoscale_setting_check = azure_autoscale_setting.clone();
            let azure_autoscale_setting_enabled_check =
                integrate_call_azure_autoscale_setting_enabled_check(
                    azure_autoscale_setting_check,
                    vmss_resource_id.clone(),
                )
                .await;
            if azure_autoscale_setting_enabled_check.is_err() {
                error!("Azure Autoscale Setting API Call Error - enabled check");
                return Err(anyhow::anyhow!(serde_json::json!({
                    "message": "Azure Autoscale Setting API Call Error - enabled check",
                    "code": "500",
                    "extras": "Azure Autoscale Setting API Call Error - enabled check",
                })));
            }

            if let Ok(Some(azure_autoscale_setting_enabled_check)) =
                azure_autoscale_setting_enabled_check
            {
                if azure_autoscale_setting_enabled_check.enabled {
                    // update autoscale setting enabled - off
                    let mut azure_autoscale_setting_update = azure_autoscale_setting.clone();
                    azure_autoscale_setting_update.autoscale_setting_name =
                        Some(azure_autoscale_setting_enabled_check.name);
                    azure_autoscale_setting_update.payload = Some(serde_json::json!({
                        "properties": {
                            "enabled": false,
                            "profiles": azure_autoscale_setting_enabled_check.profiles,
                            "targetResourceUri": vmss_resource_id.clone()
                        },
                    }));
                    let response_update =
                        call_azure_patch_autoscale_settings_update(azure_autoscale_setting_update)
                            .await;
                    if response_update.is_err() {
                        error!("Azure Autoscale Setting API Call Error - update");
                        return Err(anyhow::anyhow!(serde_json::json!({
                            "message": "Azure Autoscale Setting API Call Error - update",
                            "code": "500",
                            "extras": response_update.unwrap_err().is_body().to_string(),
                        })));
                    }
                    let response_update = response_update.unwrap();
                    let response_update_status = response_update.status();
                    let response_update_body = response_update.text().await.unwrap();
                    if !response_update_status.is_success() {
                        error!("Azure Autoscale Setting API Call Fail - update");
                        return Err(anyhow::anyhow!(serde_json::json!({
                            "message": "Azure Autoscale Setting API Call Fail - update",
                            "code": "500",
                            "extras": response_update_body
                        })));
                    }
                }
            }

            // update vmss capacity
            let response_capacity =
                call_azure_patch_virtual_machine_scale_sets_capacity(azure_vmss_setting).await;

            if response_capacity.is_err() {
                error!("Azure Autoscale Setting API Call Error - update capacity");
                return Err(anyhow::anyhow!(serde_json::json!({
                    "message": "Azure VMSS API Call Error - update capacity",
                    "code": "500",
                    "extras": response_capacity.unwrap_err().is_body().to_string(),
                })));
            }
            let response_capacity = response_capacity.unwrap();
            let response_capacity_status = response_capacity.status();
            let response_capacity_body = response_capacity.text().await.unwrap_or(String::from(
                "Azure Autoscale Setting API Call Fail - update capacity",
            ));
            if !response_capacity_status.is_success() {
                error!("Azure Autoscale Setting API Call Fail - update capacity");
                return Err(anyhow::anyhow!(serde_json::json!({
                    "message": "Azure Autoscale Setting API Call Fail - update capacity",
                    "code": "500",
                    "extras": response_capacity_body
                })));
            }
        } else {
            error!("Invalid metadata");
            return Err(anyhow::anyhow!("Invalid metadata"));
        }
        Ok(params)
    }
}

/*
 * precondition check
 *  => autoscale setting enabled Y/N or not exist
 */
async fn integrate_call_azure_autoscale_setting_enabled_check(
    azure_autoscale_setting: AzureAutoscaleSetting,
    vmss_resource_id: String,
) -> Result<Option<AzureAutoscaleSettingItems>, anyhow::Error> {
    // get autoscale setting list
    let response_autoscale_list =
        call_azure_get_autoscale_settings_list_by_resource_group(azure_autoscale_setting).await;
    if response_autoscale_list.is_err() {
        error!("ERROR: get azure autoscale setting list");
        return Err(anyhow::anyhow!("ERROR: get azure autoscale setting list"));
    }
    let response_autoscale_list_json = response_autoscale_list
        .unwrap()
        .json::<serde_json::Value>()
        .await;
    if response_autoscale_list_json.is_err() {
        error!("ERROR: get azure autoscale setting list - json");
        return Err(anyhow::anyhow!(
            "ERROR: get azure autoscale setting list - json"
        ));
    }

    // get autoscale setting info
    let azure_autoscale_setting_items = get_autoscale_setting_items_from_list(
        response_autoscale_list_json.unwrap(),
        vmss_resource_id,
    );
    if azure_autoscale_setting_items.is_err() {
        error!("ERROR: get azure autoscale setting list - items");
        return Err(anyhow::anyhow!(
            "ERROR: get azure autoscale setting list - items"
        ));
    }
    let result_items = azure_autoscale_setting_items.unwrap();
    // auto scale not exists - None.
    Ok(result_items)
}

struct AzureAutoscaleSettingItems {
    name: String,
    profiles: serde_json::Value,
    enabled: bool,
}

fn get_autoscale_setting_items_from_list(
    autoscale_setting_list: serde_json::Value,
    vmss_resource_id: String,
) -> Result<Option<AzureAutoscaleSettingItems>, anyhow::Error> {
    let Some(list_value_array) = autoscale_setting_list.get("value").unwrap().as_array() else {
        error!("ERROR: get azure autoscale setting list - value");
        return Err(anyhow::anyhow!("ERROR: get azure autoscale setting list - value"));
    };

    for value_item in list_value_array {
        let item = serde_json::json!(value_item);

        let Some(item_properties) = item.get("properties") else {
            error!("ERROR: get azure autoscale setting list - properties");
            return Err(anyhow::anyhow!("ERROR: get azure autoscale setting list - properties"));
        };
        let Some(item_properties_target_resource_uri) = item_properties.get("targetResourceUri") else {
            error!("ERROR: get azure autoscale setting list - properties.targetResourceUri");
            return Err(anyhow::anyhow!("ERROR: get azure autoscale setting list - properties.targetResourceUri"));
        };
        if item_properties_target_resource_uri
            == &serde_json::Value::String(vmss_resource_id.to_string())
        {
            let Some(item_name) = item.get("name").unwrap().as_str() else {
                error!("ERROR: get azure autoscale setting list - name");
                return Err(anyhow::anyhow!("ERROR: get azure autoscale setting list - name"));
            };
            let Some(item_properties_profiles) = item_properties.get("profiles") else {
                error!("ERROR: get azure autoscale setting list - properties.profiles");
                return Err(anyhow::anyhow!("ERROR: get azure autoscale setting list - properties.profiles"));
            };
            let Some(item_properties_enabled) = item_properties.get("enabled").unwrap().as_bool() else {
                error!("ERROR: get azure autoscale setting list - properties.enabled");
                return Err(anyhow::anyhow!("ERROR: get azure autoscale setting list - properties.enabled"));
            };
            return Ok(Some(AzureAutoscaleSettingItems {
                name: item_name.to_string(),
                profiles: item_properties_profiles.clone(),
                enabled: item_properties_enabled,
            }));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_autoscale_setting_items_from_list() {
        let target_resource_uri_ok = "/subscriptions/aaaaaaaa-bbbb-4974-9099-a26bd6ffeda3/resourceGroups/TestingMetricsScaleSet/providers/Microsoft.Compute/virtualMachineScaleSets/testingsc";
        let target_resource_uri_err = "test";
        let json_data_ok = serde_json::json!({
          "value": [
            {
              "name": "MySetting",
              "properties": {
                "profiles": [
                  {
                    "rules": [
                    ],
                  },
                ],
                "enabled": true,
                "targetResourceUri": "/subscriptions/aaaaaaaa-bbbb-4974-9099-a26bd6ffeda3/resourceGroups/TestingMetricsScaleSet/providers/Microsoft.Compute/virtualMachineScaleSets/testingsc",
              }
            }
          ],
        });
        let json_data_empty = serde_json::json!({
          "value": [
          ],
        });
        assert!(get_autoscale_setting_items_from_list(
            json_data_ok.clone(),
            target_resource_uri_ok.to_string()
        )
        .unwrap()
        .is_some());
        assert!(get_autoscale_setting_items_from_list(
            json_data_ok,
            target_resource_uri_err.to_string()
        )
        .unwrap()
        .is_none());
        assert!(get_autoscale_setting_items_from_list(
            json_data_empty,
            target_resource_uri_err.to_string()
        )
        .unwrap()
        .is_none());
    }
}
