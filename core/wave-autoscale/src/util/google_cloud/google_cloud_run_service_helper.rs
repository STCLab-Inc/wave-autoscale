use super::*;

use reqwest::{Client, Error, Response};
use serde::{Deserialize, Serialize};

// v1 and v2    - https://cloud.google.com/run/docs/reference/about-apis

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CloudRunFirstVersionGetServiceSetting {
    pub location_name: String,
    pub project_name: String,
    pub service_name: String,
}
// v1   - https://cloud.google.com/run/docs/reference/rest/v1/namespaces.services/get
pub async fn call_get_cloud_run_first_version_service(
    cloud_run_service_setting: CloudRunFirstVersionGetServiceSetting,
) -> Result<Response, Error> {
    Client::new()
        .get(format!(
            "https://{location_name}-run.googleapis.com/apis/serving.knative.dev/v1/namespaces/{project_name}/services/{service_name}",            
            location_name = &cloud_run_service_setting.location_name,
            project_name = &cloud_run_service_setting.project_name,            
            service_name = &cloud_run_service_setting.service_name,            
        ))
        .bearer_auth(get_gcp_credential_token().await.unwrap_or("".to_string()))
        .send()
        .await
}

pub struct CloudRunFirstVersionPutServiceSetting {
    pub location_name: String,
    pub project_name: String,
    pub service_name: String,
    pub payload: Option<serde_json::Value>,
}
// v1   - https://cloud.google.com/run/docs/reference/rest/v1/namespaces.services/replaceService
pub async fn call_put_cloud_run_first_version_service(
    cloud_run_service_setting: CloudRunFirstVersionPutServiceSetting,
) -> Result<Response, Error> {
    Client::new()
        .put(format!(
            "https://{location_name}-run.googleapis.com/apis/serving.knative.dev/v1/namespaces/{project_name}/services/{service_name}",            
            location_name = &cloud_run_service_setting.location_name,
            project_name = &cloud_run_service_setting.project_name,
            service_name = &cloud_run_service_setting.service_name,            
        ))
        .bearer_auth(get_gcp_credential_token().await.unwrap_or("".to_string()))
        .json(&cloud_run_service_setting.payload)
        .send()
        .await
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CloudRunSecondVersionGetServiceSetting {
    pub project_name: String,
    pub location_name: String,
    pub service_name: String,
}
// v2   - https://cloud.google.com/run/docs/reference/rest/v2/projects.locations.services/get
pub async fn call_get_cloud_run_second_version_service(
    cloud_run_service_setting: CloudRunSecondVersionGetServiceSetting,
) -> Result<Response, Error> {
    Client::new()
        .get(format!(
            "https://run.googleapis.com/v2/projects/{project_name}/locations/{location_name}/services/{service_name}",
            project_name = &cloud_run_service_setting.project_name,
            location_name = &cloud_run_service_setting.location_name,
            service_name = &cloud_run_service_setting.service_name,
        ))
        .bearer_auth(get_gcp_credential_token().await.unwrap_or("".to_string()))
        .send()
        .await
}

pub struct CloudRunSecondVersionPatchServiceSetting {
    pub project_name: String,
    pub location_name: String,    
    pub service_name: String,
    pub payload: Option<serde_json::Value>,
}
// v2   - https://cloud.google.com/run/docs/reference/rest/v2/projects.locations.services/patch
pub async fn call_patch_cloud_run_second_version_service(
    cloud_run_service_setting: CloudRunSecondVersionPatchServiceSetting,
) -> Result<Response, Error> {
    Client::new()
        .patch(format!(
            "https://run.googleapis.com/v2/projects/{project_name}/locations/{location_name}/services/{service_name}",
            project_name = &cloud_run_service_setting.project_name,
            location_name = &cloud_run_service_setting.location_name,
            service_name = &cloud_run_service_setting.service_name,         
        ))
        .bearer_auth(get_gcp_credential_token().await.unwrap_or("".to_string()))
        .json(&cloud_run_service_setting.payload)
        .send()
        .await
}

#[cfg(test)]
mod test {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn test_call_get_cloud_run_first_version_service() {
        let cloud_run_service_setting = CloudRunFirstVersionGetServiceSetting {            
            location_name: "asia-northeast2".to_string(),
            project_name: "wave-autoscale-test".to_string(),               
            service_name: "service-1".to_string(),                           
        };

        let response = call_get_cloud_run_first_version_service(cloud_run_service_setting)
            .await
            .unwrap();

        let status = response.status();
        let body = response.text().await.unwrap_or("".to_string());
        println!(
            "test_call_get_cloud_run_first_version_service contents: {:?}",
            body
        );

        assert!(status == reqwest::StatusCode::OK);
    }

    #[ignore]
    #[tokio::test]
    async fn test_call_put_cloud_run_first_version_service() {
        let cloud_run_service_setting = CloudRunFirstVersionPutServiceSetting {
            location_name: "asia-northeast2".to_string(),
            project_name: "wave-autoscale-test".to_string(),            
            service_name: "service-1".to_string(),
            payload: Some(serde_json::json!({
                "apiVersion": "serving.knative.dev/v1",
                "kind": "Service",     
                "metadata": {
                    "name": "service-1",
                    "namespace": "wave-autoscale-test",
                },           
                "spec": {
                    "template": {
                        "metadata": {      
                            "annotations": {                                
                                "autoscaling.knative.dev/minScale": "5",
                                "autoscaling.knative.dev/maxScale": "20",
                                "run.googleapis.com/execution-environment": "gen1"
                            }
                        },
                        "spec": {
                            "containerConcurrency": "20",
                            "containers": [
                                {
                                    "image": "us-docker.pkg.dev/cloudrun/container/hello",                                   
                                }
                            ]
                        }
                    },
                },                                
            })),            
        };

        let response = call_put_cloud_run_first_version_service(cloud_run_service_setting)
            .await
            .unwrap();

        let status = response.status();
        let body = response.text().await.unwrap_or("".to_string());
        println!(
            "test_call_put_cloud_run_first_version_service contents: {:?}",
            body
        );

        assert!(status == reqwest::StatusCode::OK);
    }
    
    #[ignore]
    #[tokio::test]
    async fn test_call_get_cloud_run_second_version_service() {
        let cloud_run_service_setting = CloudRunSecondVersionGetServiceSetting {            
            project_name: "wave-autoscale-test".to_string(),               
            location_name: "asia-northeast2".to_string(),
            service_name: "service-1".to_string(),                           
        };

        let response = call_get_cloud_run_second_version_service(cloud_run_service_setting)
            .await
            .unwrap();

        let status = response.status();
        let body = response.text().await.unwrap_or("".to_string());
        println!(
            "test_call_get_cloud_run_second_version_service contents: {:?}",
            body
        );

        assert!(status == reqwest::StatusCode::OK);
    }

    #[ignore]
    #[tokio::test]
    async fn test_call_patch_cloud_run_second_version_service() {
        let cloud_run_service_setting = CloudRunSecondVersionPatchServiceSetting {
            project_name: "wave-autoscale-test".to_string(),
            location_name: "asia-northeast2".to_string(),
            service_name: "service-1".to_string(),
            payload: Some(serde_json::json!({
                "template":{
                    "maxInstanceRequestConcurrency" : "8",
                    "scaling": {
                        "minInstanceCount": "3",
                        "maxInstanceCount": "4",
                    },
                    "containers": [{
                        "image": "us-docker.pkg.dev/cloudrun/container/hello",                                   
                    }],
                    /* "executionEnvironment": "EXECUTION_ENVIRONMENT_UNSPECIFIED", */
                    /* "executionEnvironment": "EXECUTION_ENVIRONMENT_GEN1", */
                    "executionEnvironment": "EXECUTION_ENVIRONMENT_GEN2",
                },                               
            })),            
        };

        let response = call_patch_cloud_run_second_version_service(cloud_run_service_setting)
            .await
            .unwrap();

        let status = response.status();
        let body = response.text().await.unwrap_or("".to_string());
        println!(
            "test_call_patch_cloud_run_second_version_service contents: {:?}",
            body
        );

        assert!(status == reqwest::StatusCode::OK);
    }

}
