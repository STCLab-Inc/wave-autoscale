use super::ScalingComponent;
use crate::util::aws::get_aws_config;
use anyhow::{Ok, Result};
use async_trait::async_trait;
use aws_sdk_emr::types::{
    InstanceCollectionType, InstanceFleetModifyConfig, InstanceFleetResizingSpecifications,
    InstanceGroupModifyConfig, OnDemandResizingSpecification, SpotResizingSpecification,
};
use aws_sdk_emr::Client;
use data_layer::ScalingComponentDefinition;
use log::{debug, error};
use serde_json::Value;
use std::collections::HashMap;

pub struct EMREC2AutoScalingComponent {
    definition: ScalingComponentDefinition,
}

impl EMREC2AutoScalingComponent {
    pub const SCALING_KIND: &'static str = "amazon-emr-ec2";

    pub fn new(definition: ScalingComponentDefinition) -> Self {
        EMREC2AutoScalingComponent { definition }
    }
}

#[async_trait]
impl ScalingComponent for EMREC2AutoScalingComponent {
    fn get_scaling_component_kind(&self) -> &str {
        &self.definition.component_kind
    }
    fn get_id(&self) -> &str {
        &self.definition.id
    }
    async fn apply(&self, params: HashMap<String, Value>) -> Result<()> {
        let metadata = self.definition.metadata.clone();

        if let (
            Some(Value::String(region)),
            Some(Value::String(access_key)),
            Some(Value::String(secret_key)),
            Some(Value::String(cluster_id)),
            Some(Value::String(instance_group_id)),
            instance_count,
            on_demand_capacity,
            spot_capacity,
            step_concurrency_level, // step concurrency level: min 1 / max 256
            on_demand_timeout_duration_minutes, // timeout duration: min 5 / max 10,080 (7days)
            spot_timeout_duration_minutes, // timeout duration: min 5 / max 10,080 (7days)
        ) = (
            metadata.get("region"),
            metadata.get("access_key"),
            metadata.get("secret_key"),
            metadata.get("cluster_id"),
            metadata.get("instance_group_id"),
            params.get("instance_count").and_then(Value::as_u64),
            params.get("on_demand_capacity").and_then(Value::as_u64),
            params.get("spot_capacity").and_then(Value::as_u64),
            params.get("step_concurrency_level").and_then(Value::as_u64),
            params
                .get("on_demand_timeout_duration_minutes")
                .and_then(Value::as_u64),
            params
                .get("spot_timeout_duration_minutes")
                .and_then(Value::as_u64),
        ) {
            let config = get_aws_config(
                Some(region.to_string()),
                Some(access_key.to_string()),
                Some(secret_key.to_string()),
                None,
                None,
            )
            .await;
            if config.is_err() {
                let config_err = config.err().unwrap();
                error!("EMR - EC2 :: get_aws_config: {:?}", config_err);
                return Err(anyhow::anyhow!(config_err));
            }
            let config = config.unwrap();
            let client = Client::new(&config);

            // 1. managed scaling check
            let managed_scaling = client
                .get_managed_scaling_policy()
                .cluster_id(cluster_id)
                .send()
                .await;
            if managed_scaling.is_err() {
                let err = managed_scaling.err().unwrap();
                let err_raw_response = format!("{:?}", err.raw_response());
                error!(
                    "EMR - EC2 :: managed scaling check error - {:?}",
                    err_raw_response
                );
                return Err(anyhow::anyhow!(serde_json::json!({
                    "message": "EMR - EC2 :: managed scaling check error",
                    "code": "500",
                    "extras": err_raw_response
                })));
            }

            if managed_scaling.unwrap().managed_scaling_policy().is_some() {
                // 1.1 remove managed scaling
                let remove_managed_scaling_result = client
                    .remove_managed_scaling_policy()
                    .cluster_id(cluster_id)
                    .send()
                    .await;
                if remove_managed_scaling_result.is_err() {
                    let err = remove_managed_scaling_result.err().unwrap();
                    let err_raw_response = format!("{:?}", err.raw_response());
                    error!(
                        "EMR - EC2 :: remove_managed_scaling_policy error - {:?}",
                        err_raw_response
                    );
                    return Err(anyhow::anyhow!(serde_json::json!({
                        "message": "EMR - EC2 :: remove_managed_scaling_policy error",
                        "code": "500",
                        "extras": err_raw_response
                    })));
                } else {
                    debug!("EMR - EC2 :: remove_managed_scaling_policy success");
                }
            }

            // 2. instance collection type check
            let describe_cluster = client
                .describe_cluster()
                .cluster_id(cluster_id)
                .send()
                .await;
            if describe_cluster.is_err() {
                let err = describe_cluster.err().unwrap();
                let err_raw_response = format!("{:?}", err.raw_response());
                error!(
                    "EMR - EC2 :: describe_cluster error - {:?}",
                    err_raw_response
                );
                return Err(anyhow::anyhow!(serde_json::json!({
                    "message": "EMR - EC2 :: describe_cluster error",
                    "code": "500",
                    "extras": err_raw_response
                })));
            }
            let describe_cluster = describe_cluster.unwrap();
            let cluster = describe_cluster.cluster();
            if cluster.is_none() {
                error!("EMR - EC2 :: not found cluster");
                return Err(anyhow::anyhow!(serde_json::json!({
                    "message": "EMR - EC2 :: not found cluster",
                    "code": "500",
                    "extras": "EMR - EC2 :: not found cluster",
                })));
            }
            let instance_collection_type = cluster.unwrap().instance_collection_type();
            match instance_collection_type {
                // 2.1 update instance fleet
                Some(InstanceCollectionType::InstanceFleet) => {
                    debug!("EMR - EC2 :: InstanceCollectionType::InstanceFleet");
                    let on_demand_spec =
                        on_demand_timeout_duration_minutes.map(|timeout_duration| {
                            OnDemandResizingSpecification::builder()
                                .timeout_duration_minutes(timeout_duration as i32)
                                .build()
                        });
                    let spot_spec = spot_timeout_duration_minutes.map(|timeout_duration| {
                        SpotResizingSpecification::builder()
                            .timeout_duration_minutes(timeout_duration as i32)
                            .build()
                    });
                    let mut resize_spec = InstanceFleetResizingSpecifications::builder();
                    if on_demand_spec.is_some() {
                        resize_spec =
                            resize_spec.set_on_demand_resize_specification(on_demand_spec);
                    }
                    if spot_spec.is_some() {
                        resize_spec = resize_spec.set_spot_resize_specification(spot_spec);
                    }
                    let option_on_demand_capacity = on_demand_capacity.map_or(0, |v| v as i32);
                    let option_spot_capacity = spot_capacity.map_or(0, |v| v as i32);
                    let modify_config = InstanceFleetModifyConfig::builder()
                        .instance_fleet_id(instance_group_id)
                        .resize_specifications(resize_spec.build())
                        .target_on_demand_capacity(option_on_demand_capacity)
                        .target_spot_capacity(option_spot_capacity)
                        .build();
                    let modify_instance_fleet = client
                        .modify_instance_fleet()
                        .cluster_id(cluster_id)
                        .instance_fleet(modify_config)
                        .send()
                        .await;
                    if modify_instance_fleet.is_err() {
                        let err = modify_instance_fleet.err().unwrap();
                        let err_raw_response = format!("{:?}", err.raw_response());
                        error!(
                            "EMR - EC2 :: modify_instance_fleet error - {:?}",
                            err_raw_response
                        );
                        return Err(anyhow::anyhow!(serde_json::json!({
                            "message": "EMR - EC2 :: modify_instance_fleet error",
                            "code": "500",
                            "extras": err_raw_response,
                        })));
                    }
                }
                // 2.2 update instance group
                Some(InstanceCollectionType::InstanceGroup) => {
                    debug!("EMR - EC2 :: InstanceCollectionType::InstanceGroup");
                    let option_instance_count = instance_count.map(|v| v as i32);
                    let modify_config = InstanceGroupModifyConfig::builder()
                        .instance_group_id(instance_group_id)
                        .set_instance_count(option_instance_count)
                        .build();
                    let modify_instance_group = client
                        .modify_instance_groups()
                        .cluster_id(cluster_id)
                        .instance_groups(modify_config)
                        .send()
                        .await;
                    if modify_instance_group.is_err() {
                        let err = modify_instance_group.err().unwrap();
                        let err_raw_response = format!("{:?}", err.raw_response());
                        error!(
                            "EMR - EC2 :: modify_instance_group error - {:?}",
                            err_raw_response
                        );
                        return Err(anyhow::anyhow!(serde_json::json!({
                            "message": "EMR - EC2 :: modify_instance_group error",
                            "code": "500",
                            "extras": err_raw_response,
                        })));
                    }
                }
                _ => {}
            }

            // 3. update cluster step concurrency level (optional)
            if step_concurrency_level.is_some() {
                let step_concurrency_level_result = client
                    .modify_cluster()
                    .cluster_id(cluster_id)
                    .step_concurrency_level(step_concurrency_level.unwrap() as i32)
                    .send()
                    .await;
                if step_concurrency_level_result.is_err() {
                    let err = step_concurrency_level_result.err().unwrap();
                    let err_raw_response = format!("{:?}", err.raw_response());
                    error!(
                        "EMR - EC2 :: step_concurrency_level_result error - {:?}",
                        err_raw_response
                    );
                    return Err(anyhow::anyhow!(serde_json::json!({
                        "message": "EMR - EC2 :: step_concurrency_level_result error",
                        "code": "500",
                        "extras": err_raw_response,
                    })));
                }
            }

            Ok(())
        } else {
            Err(anyhow::anyhow!("Invalid metadata"))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::util::aws::get_aws_config;
    use aws_config::SdkConfig;
    use aws_sdk_emr::types::{
        InstanceFleetModifyConfig, InstanceFleetResizingSpecifications, InstanceGroupModifyConfig,
        OnDemandResizingSpecification, SpotResizingSpecification,
    };

    async fn get_test_aws_config() -> SdkConfig {
        get_aws_config(
            Some("ap-northeast-1".to_string()),
            Some("AWS_ACCESS_KEY".to_string()),
            Some("AWS_SECRET_KEY".to_string()),
            None,
            None,
        )
        .await
        .unwrap()
    }

    struct TestEmrInfo {
        instance_group_cluster_id: String,
        instance_group_master_id: String,
        instance_group_core_id: String,
        instance_fleet_cluster_id: String,
        instance_fleet_master_id: String,
        instance_fleet_core_id: String,
    }

    fn emr_info() -> TestEmrInfo {
        TestEmrInfo {
            instance_group_cluster_id: "j-210Z5K271XHCG".to_string(),
            instance_group_master_id: "ig-2FEIL8F8BF9HB".to_string(),
            instance_group_core_id: "ig-1PCH4S0A9AF4".to_string(),
            instance_fleet_cluster_id: "j-2KQDT4NSA8QZD".to_string(),
            instance_fleet_master_id: "if-7U0YO6B57LMR".to_string(),
            instance_fleet_core_id: "if-35S6BLEOFEGI5".to_string(),
        }
    }

    #[ignore]
    #[tokio::test]
    async fn test_emr_managed_scaling_check_and_off() {
        let config = get_test_aws_config().await;
        let client = aws_sdk_emr::Client::new(&config);
        let managed_scaling = client
            .get_managed_scaling_policy()
            .cluster_id(emr_info().instance_group_cluster_id)
            .send()
            .await;

        if managed_scaling.unwrap().managed_scaling_policy().is_some() {
            // remove managed scaling
            let result = client
                .remove_managed_scaling_policy()
                .cluster_id(emr_info().instance_group_cluster_id)
                .send()
                .await;
            assert!(result.is_ok());
        }
    }

    #[ignore]
    #[tokio::test]
    async fn test_emr_update_step_concurrency_level_ok() {
        let step_concurrency_level = 2;
        let config = get_test_aws_config().await;
        let client = aws_sdk_emr::Client::new(&config);
        let step_concurrency_level_result = client
            .modify_cluster()
            .cluster_id(emr_info().instance_group_cluster_id)
            .step_concurrency_level(step_concurrency_level)
            .send()
            .await;
        assert!(step_concurrency_level_result
            .unwrap()
            .step_concurrency_level()
            .eq(&Some(step_concurrency_level)));
    }

    #[ignore]
    #[tokio::test]
    async fn test_emr_update_step_concurrency_level_err_param_out_of_range() {
        let step_concurrency_level = 300;
        let config = get_test_aws_config().await;
        let client = aws_sdk_emr::Client::new(&config);
        let step_concurrency_level_result = client
            .modify_cluster()
            .cluster_id(emr_info().instance_group_cluster_id)
            .step_concurrency_level(step_concurrency_level)
            .send()
            .await;
        assert!(step_concurrency_level_result.is_err());
    }

    #[ignore]
    #[tokio::test]
    async fn test_emr_update_instance_group_ok() {
        let config = get_test_aws_config().await;
        let client = aws_sdk_emr::Client::new(&config);
        let modify_config = InstanceGroupModifyConfig::builder()
            .instance_group_id(emr_info().instance_group_core_id)
            .instance_count(2)
            .build();
        let modify_instance_group = client
            .modify_instance_groups()
            .cluster_id(emr_info().instance_group_cluster_id)
            .instance_groups(modify_config)
            .send()
            .await;
        assert!(modify_instance_group.is_ok());
    }

    #[ignore]
    #[tokio::test]
    async fn test_emr_update_instance_group_err_not_instance_group_id() {
        let config = get_test_aws_config().await;
        let client = aws_sdk_emr::Client::new(&config);
        let modify_config = InstanceGroupModifyConfig::builder()
            .instance_group_id(emr_info().instance_fleet_core_id)
            .instance_count(1)
            .build();
        let modify_instance_group = client
            .modify_instance_groups()
            .cluster_id(emr_info().instance_group_cluster_id)
            .instance_groups(modify_config)
            .send()
            .await;
        assert!(modify_instance_group.is_err());
    }

    #[ignore]
    #[tokio::test]
    async fn test_emr_update_instance_group_err_master_instance_group() {
        let config = get_test_aws_config().await;
        let client = aws_sdk_emr::Client::new(&config);
        let modify_config = InstanceGroupModifyConfig::builder()
            .instance_group_id(emr_info().instance_group_master_id)
            .instance_count(1)
            .build();
        let modify_instance_group = client
            .modify_instance_groups()
            .cluster_id(emr_info().instance_group_cluster_id)
            .instance_groups(modify_config)
            .send()
            .await;
        assert!(modify_instance_group.is_err());
    }

    #[ignore]
    #[tokio::test]
    async fn test_emr_update_instance_fleet_ok() {
        let on_demand_timeout = 5;
        let spot_timeout = 5;
        let on_demand_capacity = 3;
        let spot_capacity = 3;
        let config = get_test_aws_config().await;
        let client = aws_sdk_emr::Client::new(&config);
        // min 5 / max 10,080 (7days)
        let ondemand_spec = OnDemandResizingSpecification::builder()
            .timeout_duration_minutes(on_demand_timeout)
            .build();
        let spot_spec = SpotResizingSpecification::builder()
            .timeout_duration_minutes(spot_timeout)
            .build();
        let resize_spec = InstanceFleetResizingSpecifications::builder()
            .on_demand_resize_specification(ondemand_spec)
            .spot_resize_specification(spot_spec)
            .build();
        let modify_config = InstanceFleetModifyConfig::builder()
            .instance_fleet_id(emr_info().instance_fleet_core_id)
            .target_on_demand_capacity(on_demand_capacity)
            .target_spot_capacity(spot_capacity)
            .resize_specifications(resize_spec)
            .build();
        let modify_instance_fleet = client
            .modify_instance_fleet()
            .cluster_id(emr_info().instance_fleet_cluster_id)
            .instance_fleet(modify_config)
            .send()
            .await;
        assert!(modify_instance_fleet.is_ok());
    }
}
