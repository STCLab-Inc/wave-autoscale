use super::*;
use reqwest::{Client, Response};

#[derive(Clone)]
pub struct AzureFunctionsPatchAppSetting {
    pub azure_credential: AzureCredential,
    pub subscription_id: String,
    pub resource_group_name: String,
    pub app_name: String,
    pub payload: Option<serde_json::Value>,
}

// https://learn.microsoft.com/en-us/rest/api/appservice/web-apps/update
pub async fn call_patch_azure_functions_app(
    azure_functions_app_setting: AzureFunctionsPatchAppSetting,
) -> Result<Response, reqwest::Error> {
    Client::new()
        .patch(format!(
            "https://management.azure.com/subscriptions/{subscriptionId}/resourceGroups/{resourceGroupName}/providers/Microsoft.Web/sites/{app_name}?api-version=2022-03-01",           
            subscriptionId = azure_functions_app_setting.subscription_id,
            resourceGroupName = azure_functions_app_setting.resource_group_name,
            app_name = azure_functions_app_setting.app_name
        ))
        .bearer_auth(get_azure_credential_token(azure_functions_app_setting.azure_credential).await.unwrap_or("".to_string()))
        .json(&azure_functions_app_setting.payload)
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
    async fn test_call_azure_functions_app_for_consumption_plan() {
        let azure_functions_app_setting = AzureFunctionsPatchAppSetting {
            azure_credential: get_test_env_data().0,
            subscription_id: get_test_env_data().1,
            resource_group_name: "test-azure-functions".to_string(),
            app_name: "functions-app-3".to_string(),
            payload: Some(serde_json::json!({
              "properties": {
                "siteConfig" : {
                  "functionAppScaleLimit": 10,
                },
              },
            })),
        };

        let response = call_patch_azure_functions_app(azure_functions_app_setting)
            .await
            .unwrap();

        let status = response.status();
        let body = response.text().await.unwrap_or("".to_string());
        println!(
            "test_call_azure_functions_app_for_consumption_plan response status: {:?}",
            status
        );
        println!(
            "test_call_azure_functions_app_for_consumption_plan response body: {:?}",
            body
        );

        assert!(status == reqwest::StatusCode::OK);
    }

    #[ignore]
    #[tokio::test]
    async fn test_call_azure_functions_app_for_premium_plan() {
        let azure_functions_app_setting = AzureFunctionsPatchAppSetting {
            azure_credential: get_test_env_data().0,
            subscription_id: get_test_env_data().1,
            resource_group_name: "test-azure-functions".to_string(),
            app_name: "functions-app-2".to_string(),
            payload: Some(serde_json::json!({
              "properties": {
                "siteConfig" : {
                  "functionAppScaleLimit": 10,
                  "minimumElasticInstanceCount": 5,
                },
              },
            })),
        };

        let response = call_patch_azure_functions_app(azure_functions_app_setting)
            .await
            .unwrap();

        let status = response.status();
        let body = response.text().await.unwrap_or("".to_string());
        println!(
            "test_call_azure_functions_app_for_premium_plan response status: {:?}",
            status
        );
        println!(
            "test_call_azure_functions_app_for_premium_plan response body: {:?}",
            body
        );

        assert!(status == reqwest::StatusCode::OK);
    }
}
