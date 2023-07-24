use super::super::util::google_cloud::gcp_managed_instance_group::{
    call_gcp_patch_autoscaler, call_gcp_patch_instance_group_manager,
    call_gcp_post_instance_group_manager_resize, GcpMigLocationKind, GcpMigSetting,
};
use super::ScalingComponent;
use anyhow::{Ok, Result};
use async_trait::async_trait;
use data_layer::ScalingComponentDefinition;
use log::error;
use serde_json::{json, Map};
use std::collections::HashMap;

pub struct MIGAutoScalingComponent {
    definition: ScalingComponentDefinition,
}

impl MIGAutoScalingComponent {
    pub const SCALING_KIND: &'static str = "gcp-compute-engine-mig";

    pub fn new(definition: ScalingComponentDefinition) -> Self {
        MIGAutoScalingComponent { definition }
    }
}

#[async_trait]
impl ScalingComponent for MIGAutoScalingComponent {
    fn get_scaling_component_kind(&self) -> &str {
        &self.definition.component_kind
    }
    fn get_id(&self) -> &str {
        &self.definition.id
    }

    async fn apply(&self, params: HashMap<String, serde_json::Value>) -> Result<()> {
        let metadata: HashMap<String, serde_json::Value> = self.definition.metadata.clone();
        if let (
            Some(serde_json::Value::String(project)),
            Some(location_kind),
            Some(serde_json::Value::String(location_name)),
            Some(serde_json::Value::String(group_name)),
            Some(resize),
        ) = (
            metadata.get("project"),
            metadata.get("location_kind"),
            metadata.get("location_name"),
            metadata.get("group_name"),
            params.get("resize").and_then(serde_json::Value::as_i64),
        ) {
            let gcp_mig_setting_common = GcpMigSetting {
                project: project.to_string(),
                location_kind: match location_kind {
                    s if s == "single_zone" => GcpMigLocationKind::Zone,
                    s if s == "region" => GcpMigLocationKind::Region,
                    _ => return Err(anyhow::anyhow!("Invalid location_kind")),
                },
                location_name: location_name.to_string(),
                group_name: group_name.to_string(),
                payload: None,
                query: None,
            };

            match gcp_mig_setting_common.location_kind {
                GcpMigLocationKind::Zone => {
                    let integrate_all_response = integrate_call_gcp_mig_zone_resize(
                        params
                            .get("min_num_replicas")
                            .and_then(serde_json::Value::as_i64),
                        params
                            .get("max_num_replicas")
                            .and_then(serde_json::Value::as_i64),
                        resize,
                        gcp_mig_setting_common,
                    )
                    .await;
                    return integrate_all_response;
                }
                GcpMigLocationKind::Region => {
                    let integrate_all_response = integrate_call_gcp_mig_region_resize(
                        params
                            .get("min_num_replicas")
                            .and_then(serde_json::Value::as_i64),
                        params
                            .get("max_num_replicas")
                            .and_then(serde_json::Value::as_i64),
                        resize,
                        gcp_mig_setting_common,
                    )
                    .await;
                    return integrate_all_response;
                }
                _ => {
                    return Err(anyhow::anyhow!("Invalid location_kind"));
                }
            }
        } else {
            Err(anyhow::anyhow!("Invalid metadata"))
        }
    }
}

/*
 * GcpMigLocationKind::Region
 * precondition
 *  => Target Distribution Shape: `Even`
 *  => instance Redistribution Type: `PROACTIVE`
 *  => Autoscaler Mode: OFF
 *  => The resource(instance group) is ready (resource is not ready -> fail)
 */
async fn integrate_call_gcp_mig_region_resize(
    min_num_replicas: Option<i64>,
    max_num_replicas: Option<i64>,
    resize: i64,
    gcp_mig_setting_common: GcpMigSetting,
) -> Result<()> {
    // TODO API rollback?
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
    gcp_mig_setting.query = Some(vec![(
        "autoscaler".to_string(),
        gcp_mig_setting.group_name.clone(),
    )]);
    let precondition_instance_group_manager_response =
        call_gcp_patch_instance_group_manager(gcp_mig_setting).await;
    if precondition_instance_group_manager_response.is_err() {
        return Err(anyhow::anyhow!(json!({
            "message": "GCP API Call Error - instance group manager",
            "code": "500",
            "extras": precondition_instance_group_manager_response.unwrap_err().is_body().to_string()
        })));
    }
    let precondition_instance_group_manager_response =
        precondition_instance_group_manager_response.unwrap();
    let precondition_instance_group_manager_response_status_code =
        precondition_instance_group_manager_response.status();
    let core::result::Result::Ok(precondition_instance_group_manager_response_body) = precondition_instance_group_manager_response.text().await else {
        return Err(anyhow::anyhow!(json!({
            "message": "GCP API Call Error - instance group manager",
            "code": "500",
            "extras": "not found response text",
        })));
    };
    if !precondition_instance_group_manager_response_status_code.is_success() {
        error!(
            "GCP API Call Error - precondition_instance_group_manager_response: {:?}",
            precondition_instance_group_manager_response_body
        );
        let json = json!({
            "message": "GCP API Call Error: not success - instance group manager",
            "code": precondition_instance_group_manager_response_status_code.as_str(),
            "extras": precondition_instance_group_manager_response_body
        });
        return Err(anyhow::anyhow!(json));
    }
    let gcp_mig_setting = gcp_mig_setting_common.clone();
    integrate_call_gcp_mig_zone_resize(min_num_replicas, max_num_replicas, resize, gcp_mig_setting)
        .await
}

/*
 * GcpMigLocationKind::Zone
 * precondition - None
 */
async fn integrate_call_gcp_mig_zone_resize(
    min_num_replicas: Option<i64>,
    max_num_replicas: Option<i64>,
    resize: i64,
    gcp_mig_setting_common: GcpMigSetting,
) -> Result<()> {
    // TODO API rollback?

    // min/max call - autoscaler patch: autoscaling mod `OFF` & replicas min/max
    let mut gcp_mig_setting = gcp_mig_setting_common.clone();
    let mut payload_map = Map::new();
    let mut autoscaling_policy_map = Map::new();
    if min_num_replicas.is_some() {
        autoscaling_policy_map.insert(
            "minNumReplicas".to_string(),
            json!(min_num_replicas.unwrap()),
        );
    }
    if max_num_replicas.is_some() {
        autoscaling_policy_map.insert(
            "maxNumReplicas".to_string(),
            json!(max_num_replicas.unwrap()),
        );
    }
    // region precondition
    if gcp_mig_setting.location_kind.to_string() == GcpMigLocationKind::Region.to_string() {
        autoscaling_policy_map.insert(
            "mode".to_string(),
            serde_json::Value::String("OFF".to_string()),
        );
    }
    payload_map.insert(
        "autoscalingPolicy".to_string(),
        serde_json::Value::Object(autoscaling_policy_map),
    );
    gcp_mig_setting.payload = Some(json!(payload_map));
    gcp_mig_setting.query = Some(vec![(
        "autoscaler".to_string(),
        gcp_mig_setting.group_name.clone(),
    )]);
    let autoscaler_response = call_gcp_patch_autoscaler(gcp_mig_setting).await;
    if autoscaler_response.is_err() {
        return Err(anyhow::anyhow!(json!({
            "message": "GCP API Call Error - autoscaler",
            "code": "500",
            "extras": autoscaler_response.unwrap_err().is_body().to_string()
        })));
    }
    let autoscaler_response = autoscaler_response.unwrap();
    let autoscaler_response_status_code = autoscaler_response.status();
    let core::result::Result::Ok(autoscaler_response_body) = autoscaler_response.text().await else {
        return Err(anyhow::anyhow!(json!({
            "message": "GCP API Call Error - autoscaler",
            "code": "500",
            "extras": "not found response text",
        })));
    };

    if !autoscaler_response_status_code.is_success() {
        error!(
            "GCP API Call Error - autoscaler_response: {:?}",
            autoscaler_response_body
        );
        let json = json!({
            "message": "GCP API Call Error: not success - autoscaler",
            "code": autoscaler_response_status_code.as_str(),
            "extras": autoscaler_response_body
        });
        return Err(anyhow::anyhow!(json));
    }

    // call resize
    let mut gcp_mig_setting = gcp_mig_setting_common.clone();
    gcp_mig_setting.payload = None;
    gcp_mig_setting.query = Some(vec![(String::from("size"), resize.to_string())]);
    let resize_response = call_gcp_post_instance_group_manager_resize(gcp_mig_setting).await;
    if resize_response.is_err() {
        return Err(anyhow::anyhow!(json!({
            "message": "GCP API Call Error - resize",
            "code": "500",
            "extras": resize_response.unwrap_err().is_body().to_string()
        })));
    }
    let resize_response = resize_response.unwrap();
    let resize_response_status_code = resize_response.status();
    let core::result::Result::Ok(resize_response_body) = resize_response.text().await else {
        return Err(anyhow::anyhow!(json!({
            "message": "GCP API Call Error - resize",
            "code": "500",
            "extras": "not found response text",
        })));
    };
    if !resize_response_status_code.is_success() {
        error!(
            "GCP API Call Error - resize_response: {:?}",
            resize_response_body
        );
        let json = json!({
            "message": "GCP API Call Error: not success - resize",
            "code": resize_response_status_code.as_str(),
            "extras": resize_response_body
        });
        return Err(anyhow::anyhow!(json));
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::super::super::util::google_cloud::gcp_managed_instance_group::{
        GcpMigLocationKind, GcpMigSetting,
    };
    use super::{integrate_call_gcp_mig_region_resize, integrate_call_gcp_mig_zone_resize};

    #[ignore]
    #[tokio::test]
    async fn test_gcp_mig_region() {
        let gcp_mig_setting_common = GcpMigSetting {
            project: "wave-autoscale-test".to_string(),
            location_kind: GcpMigLocationKind::Region,
            location_name: "asia-northeast2".to_string(),
            group_name: "test-instance-group-1".to_string(),
            payload: None,
            query: None,
        };

        let integrate_all_response =
            integrate_call_gcp_mig_region_resize(None, None, 2, gcp_mig_setting_common).await;
        assert!(integrate_all_response.is_ok());
    }

    #[ignore]
    #[tokio::test]
    async fn test_gcp_mig_zone() {
        let gcp_mig_setting_common = GcpMigSetting {
            project: "wave-autoscale-test".to_string(),
            location_kind: GcpMigLocationKind::Zone,
            location_name: "asia-northeast2-a".to_string(),
            group_name: "instance-group-2".to_string(),
            payload: None,
            query: None,
        };

        let integrate_all_response =
            integrate_call_gcp_mig_zone_resize(Some(2), Some(11), 2, gcp_mig_setting_common).await;
        assert!(integrate_all_response.is_ok());
    }
}
