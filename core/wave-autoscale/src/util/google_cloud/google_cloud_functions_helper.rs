use super::*;

use reqwest::{Client, Error, Response};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GoogleCloudFunctionsSetting {
    pub function_version: String,
    pub project_name: String,
    pub location_name: String,
    pub function_name: String,
    pub payload: Option<serde_json::Value>,
    pub query: Option<Vec<(String, String)>>,
}

// v1   - https://cloud.google.com/functions/docs/reference/rest/v1/projects.locations.functions/patch
// v2   - https://cloud.google.com/functions/docs/reference/rest/v2/projects.locations.functions/patch
pub async fn call_patch_google_cloud_functions(
    google_cloud_functions_setting: GoogleCloudFunctionsSetting,
) -> Result<Response, Error> {
    Client::new()
        .patch(format!(
            "https://cloudfunctions.googleapis.com/{function_version}/projects/{project_name}/locations/{location_name}/functions/{function_name}",
            function_version = &google_cloud_functions_setting.function_version,
            project_name = &google_cloud_functions_setting.project_name,
            location_name = &google_cloud_functions_setting.location_name,
            function_name = &google_cloud_functions_setting.function_name,
        ))
        .bearer_auth(get_gcp_credential_token().await.unwrap_or("".to_string()))
        .query(&google_cloud_functions_setting.query)
        .json(&google_cloud_functions_setting.payload)
        .send()
        .await
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::google_cloud::google_cloud_functions_helper::GoogleCloudFunctionsSetting;

    #[ignore]
    #[tokio::test]
    async fn test_call_patch_google_cloud_functions_for_version_1_function() {
        let google_cloud_functions_setting = GoogleCloudFunctionsSetting {
            function_version: "v1".to_string(),
            project_name: "wave-autoscale-test".to_string(),
            location_name: "asia-northeast2".to_string(),
            function_name: "function-1".to_string(),
            payload: Some(serde_json::json!({
                "minInstances": 2,
                "maxInstances": 5,
            })),
            query: Some(vec![(
                String::from("updateMask"),
                String::from("minInstances, maxInstances"),
            )]),
        };

        let response = call_patch_google_cloud_functions(google_cloud_functions_setting)
            .await
            .unwrap();

        let status = response.status();
        let body = response.text().await.unwrap_or("".to_string());
        println!(
            "test_call_patch_google_cloud_functions_for_version_1_function contents: {:?}",
            body
        );

        assert!(status == reqwest::StatusCode::OK);
    }

    #[ignore]
    #[tokio::test]
    async fn test_call_patch_google_cloud_functions_for_version_2_function() {
        let google_cloud_functions_setting = GoogleCloudFunctionsSetting {
            function_version: "v2".to_string(),
            project_name: "wave-autoscale-test".to_string(),
            location_name: "asia-northeast2".to_string(),
            function_name: "function-2".to_string(),
            payload: Some(serde_json::json!({
                "serviceConfig": {"minInstanceCount":5, "maxInstanceCount":8, "maxInstanceRequestConcurrency":10}
            })),
            query: Some(vec![(
                String::from("updateMask"),
                String::from("serviceConfig.minInstanceCount, serviceConfig.maxInstanceCount, serviceConfig.maxInstanceRequestConcurrency"),
            )]),
        };

        let response = call_patch_google_cloud_functions(google_cloud_functions_setting)
            .await
            .unwrap();

        let status = response.status();
        let body = response.text().await.unwrap_or("".to_string());
        println!(
            "test_call_patch_google_cloud_functions_for_version_2_function contents: {:?}",
            body
        );

        assert!(status == reqwest::StatusCode::OK);
    }
}
