mod test_amazon_emr_ec2 {
    use data_layer::types::object_kind::ObjectKind;
    use data_layer::ScalingComponentDefinition;
    use serde_json::json;
    use std::collections::HashMap;
    use wave_autoscale::scaling_component::{
        amazon_emr_ec2::EMREC2AutoScalingComponent, ScalingComponentManager,
    };

    struct TestEmrInfo {
        region: String,
        access_key: String,
        secret_key: String,
        instance_group_cluster_id: String,
        instance_group_core_id: String,
        instance_fleet_cluster_id: String,
        instance_fleet_core_id: String,
        instance_count: i64,
        on_demand_capacity: i64,
        spot_capacity: i64,
        step_concurrency_level: i64,
        on_demand_timeout_duration_minutes: i64,
        spot_timeout_duration_minutes: i64,
    }

    fn emr_info() -> TestEmrInfo {
        TestEmrInfo {
            region: "ap-northeast-1".to_string(),
            access_key: "AWS_ACCESS_KEY".to_string(),
            secret_key: "AWS_SECRET_KEY".to_string(),
            instance_group_cluster_id: "j-3LH6Z84089JC3".to_string(),
            instance_group_core_id: "ig-2S31LAZVPDDUE".to_string(),
            instance_fleet_cluster_id: "j-2T6BQIJU088IA".to_string(),
            instance_fleet_core_id: "if-3U3AS4Q3S5DME".to_string(),
            instance_count: 2,
            on_demand_capacity: 2,
            spot_capacity: 2,
            step_concurrency_level: 2,
            on_demand_timeout_duration_minutes: 10,
            spot_timeout_duration_minutes: 10,
        }
    }

    fn get_test_instance_group_map() -> HashMap<String, serde_json::Value> {
        vec![
            (String::from("region"), serde_json::json!(emr_info().region)),
            (
                String::from("access_key"),
                serde_json::json!(emr_info().access_key),
            ),
            (
                String::from("secret_key"),
                serde_json::json!(emr_info().secret_key),
            ),
            (
                String::from("cluster_id"),
                serde_json::json!(emr_info().instance_group_cluster_id),
            ),
            (
                String::from("instance_group_id"),
                serde_json::json!(emr_info().instance_group_core_id),
            ),
        ]
        .into_iter()
        .collect()
    }

    fn get_test_instance_fleet_map() -> HashMap<String, serde_json::Value> {
        vec![
            (String::from("region"), serde_json::json!(emr_info().region)),
            (
                String::from("access_key"),
                serde_json::json!(emr_info().access_key),
            ),
            (
                String::from("secret_key"),
                serde_json::json!(emr_info().secret_key),
            ),
            (
                String::from("cluster_id"),
                serde_json::json!(emr_info().instance_fleet_cluster_id),
            ),
            (
                String::from("instance_group_id"),
                serde_json::json!(emr_info().instance_fleet_core_id),
            ),
        ]
        .into_iter()
        .collect()
    }

    #[ignore]
    #[tokio::test]
    async fn test_amazon_emr_ec2_instance_group() {
        let scaling_component_definitions = vec![ScalingComponentDefinition {
            kind: ObjectKind::ScalingComponent,
            db_id: "".to_string(),
            id: "amazon_emr_ec2_server".to_string(),
            component_kind: "amazon-emr-ec2".to_string(),
            metadata: get_test_instance_group_map(),
            ..Default::default()
        }];

        // create metric adapter
        let mut scaling_component_manager = ScalingComponentManager::new();
        scaling_component_manager.add_definitions(scaling_component_definitions);

        if let Some(scaling_component) =
            scaling_component_manager.get_scaling_component("amazon_emr_ec2_server")
        {
            let name = scaling_component.get_scaling_component_kind();
            assert!(
                name == EMREC2AutoScalingComponent::SCALING_KIND,
                "Unexpected"
            );
        } else {
            assert!(false, "No scaling component found");
        }

        // run scaling trigger
        let mut options: HashMap<String, serde_json::Value> = HashMap::new();
        options.insert(
            "instance_count".to_string(),
            json!(emr_info().instance_count),
        );
        options.insert(
            "step_concurrency_level".to_string(),
            json!(emr_info().step_concurrency_level),
        );

        let result = scaling_component_manager
            .apply_to("amazon_emr_ec2_server", options)
            .await;
        assert!(result.is_ok());
    }

    #[ignore]
    #[tokio::test]
    async fn test_amazon_emr_ec2_instance_fleet() {
        let scaling_component_definitions = vec![ScalingComponentDefinition {
            kind: ObjectKind::ScalingComponent,
            db_id: "".to_string(),
            id: "amazon_emr_ec2_server".to_string(),
            component_kind: "amazon-emr-ec2".to_string(),
            metadata: get_test_instance_fleet_map(),
            ..Default::default()
        }];

        // create metric adapter
        let mut scaling_component_manager = ScalingComponentManager::new();
        scaling_component_manager.add_definitions(scaling_component_definitions);

        if let Some(scaling_component) =
            scaling_component_manager.get_scaling_component("amazon_emr_ec2_server")
        {
            let name = scaling_component.get_scaling_component_kind();
            assert!(
                name == EMREC2AutoScalingComponent::SCALING_KIND,
                "Unexpected"
            );
        } else {
            assert!(false, "No scaling component found");
        }

        // run scaling trigger
        let mut options: HashMap<String, serde_json::Value> = HashMap::new();
        options.insert(
            "on_demand_capacity".to_string(),
            json!(emr_info().on_demand_capacity),
        );
        options.insert("spot_capacity".to_string(), json!(emr_info().spot_capacity));
        options.insert(
            "on_demand_timeout_duration_minutes".to_string(),
            json!(emr_info().on_demand_timeout_duration_minutes),
        );
        options.insert(
            "spot_timeout_duration_minutes".to_string(),
            json!(emr_info().spot_timeout_duration_minutes),
        );
        options.insert(
            "step_concurrency_level".to_string(),
            json!(emr_info().step_concurrency_level),
        );

        let result = scaling_component_manager
            .apply_to("amazon_emr_ec2_server", options)
            .await;
        assert!(result.is_ok());
    }
}
