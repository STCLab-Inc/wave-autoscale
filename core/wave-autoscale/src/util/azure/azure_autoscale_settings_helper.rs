use super::*;
use reqwest::{Client, Response};

#[derive(Clone)]
pub struct AzureAutoscaleSetting {
    pub azure_credential: AzureCredential,
    pub subscription_id: String,
    pub resource_group_name: String,
    pub autoscale_setting_name: Option<String>,
    pub payload: Option<serde_json::Value>,
}

// https://learn.microsoft.com/en-us/rest/api/monitor/autoscale-settings/list-by-resource-group?tabs=HTTP
pub async fn call_azure_get_autoscale_settings_list_by_resource_group(
    azure_autoscale_setting: AzureAutoscaleSetting,
) -> Result<Response, reqwest::Error> {
    Client::new()
        .get(format!(
            "https://management.azure.com/subscriptions/{subscriptionId}/resourcegroups/{resourceGroupName}/providers/Microsoft.Insights/autoscalesettings?api-version=2022-10-01"
            , subscriptionId = azure_autoscale_setting.subscription_id
            , resourceGroupName = azure_autoscale_setting.resource_group_name
        ))
        .bearer_auth(get_azure_credential_token(azure_autoscale_setting.azure_credential).await.unwrap_or(String::from("")))
        .send()
        .await
}

// https://learn.microsoft.com/en-us/rest/api/monitor/autoscale-settings/update?tabs=HTTP
pub async fn call_azure_patch_autoscale_settings_update(
    azure_autoscale_setting: AzureAutoscaleSetting,
) -> Result<Response, reqwest::Error> {
    Client::new()
        .patch(format!(
            "https://management.azure.com/subscriptions/{subscriptionId}/resourcegroups/{resourceGroupName}/providers/Microsoft.Insights/autoscalesettings/{autoscaleSettingName}?api-version=2022-10-01"
            , subscriptionId = azure_autoscale_setting.subscription_id
            , resourceGroupName = azure_autoscale_setting.resource_group_name
            , autoscaleSettingName = azure_autoscale_setting.autoscale_setting_name.unwrap_or(String::from(""))
        ))
        .bearer_auth(get_azure_credential_token(azure_autoscale_setting.azure_credential).await.unwrap_or(String::from("")))
        .json(&azure_autoscale_setting.payload)
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

    fn get_autoscale_setting_name(response_json: serde_json::Value, vmss_resource_id :String) -> (String, serde_json::Value) {
        let autoscale_list = response_json.get("value").unwrap();
        for item in autoscale_list.as_array().unwrap() {
            let item = serde_json::json!(item);
            if item.get("properties").unwrap().get("targetResourceUri").unwrap() == &serde_json::Value::String(vmss_resource_id.to_string()) {
                return (item.get("name").unwrap().as_str().unwrap().to_string(), item.get("properties").unwrap().get("profiles").unwrap().clone());
            }
        }
        ("".to_string(), serde_json::Value::Null)
    }


    #[ignore]
    #[tokio::test]
    async fn test_call_azure_get_autoscale_settings_list_by_resource_group() {
        let azure_autoscale_setting = AzureAutoscaleSetting {
            azure_credential: get_test_env_data().0,
            subscription_id: get_test_env_data().1,
            resource_group_name: "test-vmss-uniform-grp".to_string(),
            autoscale_setting_name: None,
            payload: None,
        };
        let response =
            call_azure_get_autoscale_settings_list_by_resource_group(azure_autoscale_setting).await;
        // println!("response: {:?}", response.unwrap().text().await.unwrap());
        assert!(response.unwrap().status().is_success());
    }

    #[ignore]
    #[tokio::test]
    async fn test_get_autoscale_setting_name() {
        let azure_autoscale_setting = AzureAutoscaleSetting {
            azure_credential: get_test_env_data().0,
            subscription_id: get_test_env_data().1,
            resource_group_name: "test-vmss-uniform-grp".to_string(),
            autoscale_setting_name: None,
            payload: None,
        };
        let vm_scale_set_name = "test-vmss-uniform".to_string();
        let vmss_resource_id = format!("/subscriptions/{subscriptionId}/resourceGroups/{resourceGroupName}/providers/Microsoft.Compute/virtualMachineScaleSets/{vmScaleSetName}",
            subscriptionId = azure_autoscale_setting.subscription_id
            , resourceGroupName = azure_autoscale_setting.resource_group_name
            , vmScaleSetName = vm_scale_set_name 
        );
        
        let response =
            call_azure_get_autoscale_settings_list_by_resource_group(azure_autoscale_setting).await;
        let response_json = response.unwrap().json::<serde_json::Value>().await.unwrap();
        assert!(!get_autoscale_setting_name(response_json, vmss_resource_id).0.is_empty());
    }

    #[ignore]
    #[tokio::test]
    async fn test_call_azure_patch_autoscale_settings_update_enabled() {
        let azure_autoscale_setting = AzureAutoscaleSetting {
            azure_credential: get_test_env_data().0,
            subscription_id: get_test_env_data().1,
            resource_group_name: "test-vmss-uniform-grp".to_string(),
            autoscale_setting_name: None,
            payload: None,
        };
        let vm_scale_set_name = "test-vmss-uniform".to_string();
        let azure_autoscale_setting_list = azure_autoscale_setting.clone();
        let response_autoscale_list =
            call_azure_get_autoscale_settings_list_by_resource_group(azure_autoscale_setting_list).await;
        let response_autoscale_list_json = response_autoscale_list.unwrap().json::<serde_json::Value>().await.unwrap();


        let mut azure_autoscale_setting_update = azure_autoscale_setting.clone();
        let vmss_resource_id = format!("/subscriptions/{subscriptionId}/resourceGroups/{resourceGroupName}/providers/Microsoft.Compute/virtualMachineScaleSets/{vmScaleSetName}",
            subscriptionId = azure_autoscale_setting_update.subscription_id
            , resourceGroupName = azure_autoscale_setting_update.resource_group_name
            , vmScaleSetName = vm_scale_set_name 
        );
        azure_autoscale_setting_update.autoscale_setting_name = Some(get_autoscale_setting_name(response_autoscale_list_json.clone(), vmss_resource_id.clone()).0);
        azure_autoscale_setting_update.payload = Some(serde_json::json!({
            "properties": {
                "enabled": false,
                "profiles": get_autoscale_setting_name(response_autoscale_list_json.clone(), vmss_resource_id.clone()).1,
                "targetResourceUri": vmss_resource_id.clone()
            },
        }));
        let response_autoscale_update =
        call_azure_patch_autoscale_settings_update(azure_autoscale_setting_update).await;
        assert!(response_autoscale_update.unwrap().status().is_success());
    }

}
