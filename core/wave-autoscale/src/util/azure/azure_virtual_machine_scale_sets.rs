use super::*;
use reqwest::{Client, Response};

#[derive(Clone)]
pub struct AzureVmssSetting {
    pub azure_credential: AzureCredential,
    pub subscription_id: String,
    pub resource_group_name: String,
    pub vm_scale_set_name: Option<String>,
    pub payload: Option<serde_json::Value>,
}

// https://learn.microsoft.com/en-us/rest/api/compute/virtual-machine-scale-sets/update?tabs=HTTP
pub async fn call_azure_patch_virtual_machine_scale_sets_capacity(
    azure_vmss_settting: AzureVmssSetting,
) -> Result<Response, reqwest::Error> {
    Client::new()
        .patch(format!(
            "https://management.azure.com/subscriptions/{subscriptionId}/resourceGroups/{resourceGroupName}/providers/Microsoft.Compute/virtualMachineScaleSets/{vmScaleSetName}?api-version=2023-03-01"
            , subscriptionId = azure_vmss_settting.subscription_id
            , resourceGroupName = azure_vmss_settting.resource_group_name
            , vmScaleSetName = azure_vmss_settting.vm_scale_set_name.unwrap_or(String::from(""))
        ))
        .bearer_auth(get_azure_credential_token(azure_vmss_settting.azure_credential).await.unwrap_or(String::from("")))
        .json(&azure_vmss_settting.payload)
        .send()
        .await
}

#[cfg(test)]
mod test {
    use super::*;

    fn get_test_env_data() -> (AzureCredential, String) {
        let azure_credential = AzureCredential {
            client_id: None,
            client_secret: None,
            tenant_id: None,
        };
        let subscription_id = std::env::var("AZURE_SUBSCRIPTION_ID").unwrap();
        (azure_credential, subscription_id)
    }

    #[ignore]
    #[tokio::test]
    async fn test_azure_vmss_uniform_capacity() {
        let azure_vmss_setting = AzureVmssSetting {
            azure_credential: get_test_env_data().0,
            subscription_id: get_test_env_data().1,
            resource_group_name: "test-vmss-uniform-grp".to_string(),
            vm_scale_set_name: Some("test-vmss-uniform".to_string()),
            payload: Some(serde_json::json!({
                "sku": {
                    "capacity": 1
                },
            })),
        };

        let resp = call_azure_patch_virtual_machine_scale_sets_capacity(azure_vmss_setting).await;
        let result = resp.unwrap();
        let result_status = result.status();
        let _result_text = result.text();
        assert!(result_status.is_success());
    }

    #[ignore]
    #[tokio::test]
    async fn test_azure_vmss_flexible_capacity() {
        let azure_vmss_setting = AzureVmssSetting {
            azure_credential: get_test_env_data().0,
            subscription_id: get_test_env_data().1,
            resource_group_name: "test-vmss-flexible-grp".to_string(),
            vm_scale_set_name: Some("test-vmss-flexible".to_string()),
            payload: Some(serde_json::json!({
                "sku": {
                    "capacity": 1
                },
            })),
        };
        let resp = call_azure_patch_virtual_machine_scale_sets_capacity(azure_vmss_setting).await;
        assert!(resp.unwrap().status().is_success());
    }
}
