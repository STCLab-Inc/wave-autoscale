use super::*;
use reqwest::{Client, Response};
use serde_json::json;

pub async fn call_azure_patch_virtual_machine_scale_sets_capacity(
    azure_credential: AzureCredential,
    subscription_id: String,
    resource_group_name: String,
    vm_scale_set_name: String,
    capacity: u32,
) -> Result<Response, reqwest::Error> {
    Client::new()
        .patch(format!(
            "https://management.azure.com/subscriptions/{subscriptionId}/resourceGroups/{resourceGroupName}/providers/Microsoft.Compute/virtualMachineScaleSets/{vmScaleSetName}?api-version=2023-03-01"
            , subscriptionId = subscription_id
            , resourceGroupName = resource_group_name
            , vmScaleSetName = vm_scale_set_name
        ))
        .bearer_auth(get_azure_credential_token(azure_credential).await.unwrap_or(String::from("")))
        .json(&json!({
            "sku": {
                "capacity": capacity
            },
        }))
        .send()
        .await
}

#[cfg(test)]
mod test {
    use super::*;

    fn get_test_env_data() -> (AzureCredential, String) {
        let azure_credential = AzureCredential {
            client_id: std::env::var("AZURE_CLIENT_ID").unwrap(),
            client_secret: std::env::var("AZURE_CLIENT_SECRET").unwrap(),
            tenant_id: std::env::var("AZURE_TENANT_ID").unwrap(),
        };
        let subscription_id = std::env::var("AZURE_SUBSCRIPTION_ID").unwrap();
        (azure_credential, subscription_id)
    }

    #[ignore]
    #[tokio::test]
    async fn test_azure_vmss_uniform_capacity() {
        let resource_group_name = "test-vmss-uniform-grp".to_string();
        let vm_scale_set_name = "test-vmss-uniform".to_string();
        let resp = call_azure_patch_virtual_machine_scale_sets_capacity(
            get_test_env_data().0,
            get_test_env_data().1,
            resource_group_name,
            vm_scale_set_name,
            1,
        )
        .await;
        assert!(resp.unwrap().status().is_success());
    }

    #[ignore]
    #[tokio::test]
    async fn test_azure_vmss_flexible_capacity() {
        let resource_group_name = "test-vmss-flexible-grp".to_string();
        let vm_scale_set_name = "test-vmss-flexible".to_string();
        let resp = call_azure_patch_virtual_machine_scale_sets_capacity(
            get_test_env_data().0,
            get_test_env_data().1,
            resource_group_name,
            vm_scale_set_name,
            1,
        )
        .await;
        assert!(resp.unwrap().status().is_success());
    }
}
