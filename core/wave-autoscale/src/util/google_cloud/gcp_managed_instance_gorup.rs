use serde::{Deserialize, Serialize};
use gcp_auth::{AuthenticationManager, Token, Error};
use reqwest::{Client, Response, StatusCode};
use log::{error};
use serde_json::{json, Value, Number, Map};
use super::*;

/*
 * Single area - zone
 * Multi area - region
 * precondition
 *  - resize
 *    => Autoscaler Mode: OFF
 *  - min/max
 *    => Target Distribution Shape: `Even` & instance Redistribution Type: `PROACTIVE`
 */


#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GcpMigAreaKind {
    Region,
    Zone
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
    pub area_kind: GcpMigAreaKind,
    pub area_name: String,
    pub group_name: String,
    pub payload: Option<Value>,
    pub query: Option<Vec<(String, String)>>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GcpMigPreconditionPayloadKind {
    ChangeTargetDistributionShapeEven,
    ChangeAutoscaleModeOff,
}

fn get_gcp_precondition_payload(payload_kind: GcpMigPreconditionPayloadKind) -> serde_json::Value {
    match payload_kind {
        GcpMigPreconditionPayloadKind::ChangeTargetDistributionShapeEven => {
            let json = json!({
                "distributionPolicy": {
                    "targetShape": "EVEN"
                },
                "updatePolicy": {
                    "instanceRedistributionType": "PROACTIVE",
                }
            });
            json
        },
        GcpMigPreconditionPayloadKind::ChangeAutoscaleModeOff => {
            let json = json!({
                "autoscalingPolicy": {
                    "mode": "OFF"
                }
            });
            json
        },
    }
}

// zone   - https://cloud.google.com/compute/docs/reference/rest/v1/autoscalers/patch
// region - https://cloud.google.com/compute/docs/reference/rest/v1/regionAutoscalers/patch
async fn call_gcp_patch_autoscaler(gcp_mig_setting: GcpMigSetting) -> Result<Response, reqwest::Error> {
    let response = Client::new()
        .patch(format!("https://compute.googleapis.com/compute/v1/projects/{project}/{areaKind}/{region}/autoscalers",
            project = &gcp_mig_setting.project, areaKind = &gcp_mig_setting.area_kind.to_string(), region = &gcp_mig_setting.area_name))
        .query(&gcp_mig_setting.query)
        .bearer_auth(get_gcp_credential_token().await.unwrap().as_str())
        .json(&gcp_mig_setting.payload)
        .send()
        .await;
    response
}

// zone   - https://cloud.google.com/compute/docs/reference/rest/v1/instanceGroupManagers/patch
// regoin - https://cloud.google.com/compute/docs/reference/rest/v1/regionInstanceGroupManagers/patch
async fn call_gcp_patch_instance_group_manager(gcp_mig_setting: GcpMigSetting) -> Result<Response, reqwest::Error> {
    let response = Client::new()
        .patch(format!("https://compute.googleapis.com/compute/v1/projects/{project}/{areaKind}/{region}/instanceGroupManagers/{instanceGroupManager}",
            project = &gcp_mig_setting.project, areaKind = &gcp_mig_setting.area_kind.to_string(),
            region = &gcp_mig_setting.area_name, instanceGroupManager = &gcp_mig_setting.group_name))
        .query(&gcp_mig_setting.query)
        .bearer_auth(get_gcp_credential_token().await.unwrap().as_str())
        .json(&gcp_mig_setting.payload)
        .send()
        .await;
    response
}

// zone   - https://cloud.google.com/compute/docs/reference/rest/v1/instanceGroupManagers/resize
// region - https://cloud.google.com/compute/docs/reference/rest/v1/regionInstanceGroupManagers/resize
async fn call_gcp_post_instance_group_manager_resize(gcp_mig_setting: GcpMigSetting) -> Result<Response, reqwest::Error> {
    let empty_payload = json!({});
    let response = Client::new()
        .post(format!("https://compute.googleapis.com/compute/v1/projects/{project}/{areaKind}/{region}/instanceGroupManagers/{instanceGroupManager}/resize",
            project = &gcp_mig_setting.project, areaKind = &gcp_mig_setting.area_kind.to_string(),
            region = &gcp_mig_setting.area_name, instanceGroupManager = &gcp_mig_setting.group_name))
        .bearer_auth(get_gcp_credential_token().await.unwrap().as_str())
        .query(&gcp_mig_setting.query)
        .json(&empty_payload)
        .send()
        .await;
    response
}


#[cfg(test)]
mod test {
    use super::*;


    //#[ignore]
    #[tokio::test]
    async fn test_gcp_mig() {
        let autoscale_min = Some(2);
        let autoscale_max = Some(11);
        let autoscale_instance = 3;


        // TODO API rollback?
        let gcp_mig_setting_common = GcpMigSetting {
            project: "wave-autoscale-test".to_string(),
            area_kind: GcpMigAreaKind::Region,
            area_name: "asia-northeast2".to_string(),
            group_name: "test-instance-group-1".to_string(),
            payload: None,
            query: None,
        };
        let mut gcp_mig_setting = gcp_mig_setting_common.clone();

        // precondition call - instance group manager patch: Target Distribution Shape: `Even` & instance Redistribution Type: `PROACTIVE`
        // targetShape - depending on the value set in updatePolicy.instanceRedistributionType
        gcp_mig_setting.payload = Some(json!({
            "distributionPolicy": {
                "targetShape": "EVEN"
            },
            "updatePolicy": {
                "instanceRedistributionType": "PROACTIVE",
            }
        }));
        gcp_mig_setting.query = Some(vec![("autoscaler".to_string(), "test-instance-group-1".to_string())]);
        let precondition_instance_group_manager_response = call_gcp_patch_instance_group_manager(gcp_mig_setting).await;
        
        let response = precondition_instance_group_manager_response.unwrap();
        if response.status() == StatusCode::OK {
            // precondition & min/max call - autoscaler patch: autoscaling mod `OFF` & replicas min/max
            let mut gcp_mig_setting = gcp_mig_setting_common.clone();
            let mut payload_map = Map::new();
            let mut autoscaling_policy_map = Map::new();
            if autoscale_min.is_some() {
                autoscaling_policy_map.insert("minNumReplicas".to_string(), Value::Number(Number::from(autoscale_min.unwrap())));
            }
            if autoscale_max.is_some() {
                autoscaling_policy_map.insert("maxNumReplicas".to_string(), Value::Number(Number::from(autoscale_max.unwrap())));
            }
            autoscaling_policy_map.insert("mode".to_string(), Value::String("OFF".to_string()));
            payload_map.insert("autoscalingPolicy".to_string(), Value::Object(autoscaling_policy_map));
            gcp_mig_setting.payload = Some(json!(payload_map));
            gcp_mig_setting.query = Some(vec![("autoscaler".to_string(), "test-instance-group-1".to_string())]);
            let precondition_autoscaler_response = call_gcp_patch_autoscaler(gcp_mig_setting).await;

            let response = precondition_autoscaler_response.unwrap();
            if response.status() == StatusCode::OK {
                // call resize
                let mut gcp_mig_setting = gcp_mig_setting_common.clone();
                gcp_mig_setting.payload = None;
                gcp_mig_setting.query = Some(vec![(String::from("size"), String::from(autoscale_instance.to_string()))]);
                let resize_response = call_gcp_post_instance_group_manager_resize(gcp_mig_setting).await;
                let response = resize_response.unwrap();
                
                if response.status() == StatusCode::OK {
                    assert!(true);
                } else {
                    println!("GCP API CALL ERROR - resize_response: {:?}", response.text().await.unwrap());
                    assert!(false);
                }
            } else {
                println!("GCP API CALL ERROR - precondition_autoscaler_response: {:?}", response.text().await.unwrap());
                assert!(false);
            }

        } else {
            println!("GCP API CALL ERROR - precondition_instance_group_manager_response: {:?}", response.text().await.unwrap());
            assert!(false);
        }

    }

    #[ignore]
    #[tokio::test]
    async fn test_gcp_mig_autoscaler_min_max() {
        let mut gcp_mig_precondition_setting = GcpMigSetting {
            project: "wave-autoscale-test".to_string(),
            area_kind: GcpMigAreaKind::Region,
            area_name: "asia-northeast2".to_string(),
            group_name: "test-instance-group-1".to_string(),
            payload: None,
            query: None,
        };
        gcp_mig_precondition_setting.payload = Some(get_gcp_precondition_payload(GcpMigPreconditionPayloadKind::ChangeTargetDistributionShapeEven));
        let mut gcp_mig_setting = gcp_mig_precondition_setting.clone();

        // ausoscaler min/max precondition - Target Distribution Shape: `Even` & instance Redistribution Type: `PROACTIVE`
        let precondition_response = call_gcp_patch_instance_group_manager(gcp_mig_precondition_setting).await;
        if precondition_response.unwrap().status() == StatusCode::OK {
            // resize call
            gcp_mig_setting.payload = Some(json!({
                "autoscalingPolicy": {
                    "minNumReplicas": 2,
                    "maxNumReplicas": 11,
                },
            }));
            gcp_mig_setting.query = Some(vec![("autoscaler".to_string(), "test-instance-group-1".to_string())]);
            let response = call_gcp_patch_autoscaler(gcp_mig_setting).await;
            assert_eq!(response.unwrap().status() == StatusCode::OK, true);
        } else {
            assert!(false);
        }
    }

    #[ignore]
    #[tokio::test]
    async fn test_call_gcp_patch_instance_group_manager() {
        let gcp_mig_setting = GcpMigSetting {
            project: "wave-autoscale-test".to_string(),
            area_kind: GcpMigAreaKind::Region,
            area_name: "asia-northeast2".to_string(),
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
        let response = call_gcp_patch_instance_group_manager(gcp_mig_setting).await;
        let body = response.unwrap().text().await.unwrap();
        println!("test_call_gcp_patch_instance_group_manager response: {:?}", body);
        //assert_eq!(response.unwrap().status() == StatusCode::OK, true);
    }

    #[ignore]
    #[tokio::test]
    async fn test_call_gcp_patch_autoscaler() {
        let gcp_mig_setting = GcpMigSetting {
            project: "wave-autoscale-test".to_string(),
            area_kind: GcpMigAreaKind::Region,
            area_name: "asia-northeast2".to_string(),
            group_name: "test-instance-group-1".to_string(),
            payload: Some(json!({
                "autoscalingPolicy": {
                    "minNumReplicas": 2,
                    "mode": "OFF"
                },
            })),
            query: Some(vec![(String::from("autoscaler"), String::from("test-instance-group-1"))]),
        };
        let response = call_gcp_patch_autoscaler(gcp_mig_setting).await;
        let body = response.unwrap().text().await.unwrap();
        println!("test_call_gcp_patch_autoscaler response: {:?}", body);
        //assert_eq!(response.unwrap().status() == StatusCode::OK, true);
    }

    #[ignore]
    #[tokio::test]
    async fn test_call_gcp_post_instance_group_manager_resize() {
        let gcp_mig_setting = GcpMigSetting {
            project: "wave-autoscale-test".to_string(),
            area_kind: GcpMigAreaKind::Region,
            area_name: "asia-northeast2".to_string(),
            group_name: "test-instance-group-1".to_string(),
            payload: None,
            query: Some(vec![(String::from("size"), String::from("3"))])
        };
        let response = call_gcp_post_instance_group_manager_resize(gcp_mig_setting).await;
        let body = response.unwrap().text().await.unwrap();
        println!("test_call_gcp_post_instance_group_manager_resize response: {:?}", body);
        //assert_eq!(response.unwrap().status() == StatusCode::OK, true);
    }

    

}