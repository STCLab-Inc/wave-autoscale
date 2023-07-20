use super::*;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GcpMigAreaKind {
    Region,
    Zone,
}

impl std::fmt::Display for GcpMigAreaKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GcpMigAreaKind::Region => write!(f, "regions"),
            GcpMigAreaKind::Zone => write!(f, "zones"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct GcpMigSetting {
    pub project: String,
    pub location_kind: GcpMigAreaKind,
    pub location_name: String,
    pub group_name: String,
    pub payload: Option<Value>,
    pub query: Option<Vec<(String, String)>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GcpMigPreconditionPayloadKind {
    ChangeTargetDistributionShapeEven,
    ChangeAutoscaleModeOff,
}

// zone   - https://cloud.google.com/compute/docs/reference/rest/v1/autoscalers/patch
// region - https://cloud.google.com/compute/docs/reference/rest/v1/regionAutoscalers/patch
pub async fn call_gcp_patch_autoscaler(
    gcp_mig_setting: GcpMigSetting,
) -> Result<Response, reqwest::Error> {
    Client::new()
        .patch(format!("https://compute.googleapis.com/compute/v1/projects/{project}/{areaKind}/{region}/autoscalers",
            project = &gcp_mig_setting.project, areaKind = &gcp_mig_setting.location_kind.to_string(), region = &gcp_mig_setting.location_name))
        .query(&gcp_mig_setting.query)
        .bearer_auth(get_gcp_credential_token().await.unwrap().as_str())
        .json(&gcp_mig_setting.payload)
        .send()
        .await
}

// zone   - https://cloud.google.com/compute/docs/reference/rest/v1/instanceGroupManagers/patch
// regoin - https://cloud.google.com/compute/docs/reference/rest/v1/regionInstanceGroupManagers/patch
pub async fn call_gcp_patch_instance_group_manager(
    gcp_mig_setting: GcpMigSetting,
) -> Result<Response, reqwest::Error> {
    Client::new()
        .patch(format!("https://compute.googleapis.com/compute/v1/projects/{project}/{areaKind}/{region}/instanceGroupManagers/{instanceGroupManager}",
            project = &gcp_mig_setting.project, areaKind = &gcp_mig_setting.location_kind.to_string(),
            region = &gcp_mig_setting.location_name, instanceGroupManager = &gcp_mig_setting.group_name))
        .query(&gcp_mig_setting.query)
        .bearer_auth(get_gcp_credential_token().await.unwrap().as_str())
        .json(&gcp_mig_setting.payload)
        .send()
        .await
}

// zone   - https://cloud.google.com/compute/docs/reference/rest/v1/instanceGroupManagers/resize
// region - https://cloud.google.com/compute/docs/reference/rest/v1/regionInstanceGroupManagers/resize
pub async fn call_gcp_post_instance_group_manager_resize(
    gcp_mig_setting: GcpMigSetting,
) -> Result<Response, reqwest::Error> {
    let empty_payload = json!({});

    Client::new()
        .post(format!("https://compute.googleapis.com/compute/v1/projects/{project}/{areaKind}/{region}/instanceGroupManagers/{instanceGroupManager}/resize",
            project = &gcp_mig_setting.project, areaKind = &gcp_mig_setting.location_kind.to_string(),
            region = &gcp_mig_setting.location_name, instanceGroupManager = &gcp_mig_setting.group_name))
        .bearer_auth(get_gcp_credential_token().await.unwrap().as_str())
        .query(&gcp_mig_setting.query)
        .json(&empty_payload)
        .send()
        .await
}

pub async fn testtest() -> Result<Response, reqwest::Error> {
    let empty_payload = json!({});
    let query = vec![(String::from("returnPartialSuccess"), String::from("true"))];

    Client::new()
        .get("https://compute.googleapis.com/compute/v1/projects/wave-autoscale-test")
        .bearer_auth(get_gcp_credential_token().await.unwrap().as_str())
        //.query(&query)
        //.json(&empty_payload)
        .send()
        .await
}

#[cfg(test)]
mod test {
    use super::*;
    use reqwest::StatusCode;

    #[ignore]
    #[tokio::test]
    async fn test_call_gcp_patch_instance_group_manager() {
        let gcp_mig_setting = GcpMigSetting {
            project: "wave-autoscale-test".to_string(),
            location_kind: GcpMigAreaKind::Region,
            location_name: "asia-northeast2".to_string(),
            group_name: "test-instance-group-1".to_string(),
            payload: Some(json!({
                "distributionPolicy": {
                    "targetShape": "EVEN"
                },
                "updatePolicy": {
                    "instanceRedistributionType": "PROACTIVE",
                }
            })),
            query: None,
        };
        let response = call_gcp_patch_instance_group_manager(gcp_mig_setting)
            .await
            .unwrap();
        let status = response.status();
        let body = response.text().await.unwrap();
        println!(
            "test_call_gcp_patch_instance_group_manager response: {:?}",
            body
        );
        assert!(status == StatusCode::OK);
    }

    #[ignore]
    #[tokio::test]
    async fn test_call_gcp_patch_autoscaler() {
        let gcp_mig_setting = GcpMigSetting {
            project: "wave-autoscale-test".to_string(),
            location_kind: GcpMigAreaKind::Region,
            location_name: "asia-northeast2".to_string(),
            group_name: "test-instance-group-1".to_string(),
            payload: Some(json!({
                "autoscalingPolicy": {
                    "minNumReplicas": 2,
                    "mode": "OFF"
                },
            })),
            query: Some(vec![(
                String::from("autoscaler"),
                String::from("test-instance-group-1"),
            )]),
        };
        let response = call_gcp_patch_autoscaler(gcp_mig_setting).await.unwrap();
        let status = response.status();
        let body = response.text().await.unwrap();
        println!("test_call_gcp_patch_autoscaler response: {:?}", body);
        assert!(status == StatusCode::OK);
    }

    #[ignore]
    #[tokio::test]
    async fn test_call_gcp_post_instance_group_manager_resize() {
        let gcp_mig_setting = GcpMigSetting {
            project: "wave-autoscale-test".to_string(),
            location_kind: GcpMigAreaKind::Region,
            location_name: "asia-northeast2".to_string(),
            group_name: "test-instance-group-1".to_string(),
            payload: None,
            query: Some(vec![(String::from("size"), String::from("3"))]),
        };
        let response = call_gcp_post_instance_group_manager_resize(gcp_mig_setting)
            .await
            .unwrap();
        let status = response.status();
        let body = response.text().await.unwrap();
        println!(
            "test_call_gcp_post_instance_group_manager_resize response: {:?}",
            body
        );
        assert!(status == StatusCode::OK);
    }
}
