mod scaling_component;
mod google_cloud_run_service_test {
    use crate::scaling_component::scaling_component_common::get_rquickjs_context;
    use data_layer::types::object_kind::ObjectKind::ScalingComponent;
    use data_layer::ScalingComponentDefinition;
    use std::collections::HashMap;
    use wave_autoscale::scaling_component::google_cloud_run_service::CloudRunServiceScalingComponent;
    use wave_autoscale::scaling_component::ScalingComponentManager;

    #[tokio::test]
    #[ignore]
    async fn apply_minimum_service_without_coldstart_for_version_1_api() {
        let metadata: HashMap<String, serde_json::Value> = vec![
            (String::from("api_version"), serde_json::json!("v1")),
            (
                String::from("location_name"),
                serde_json::json!("asia-northeast2"),
            ),
            (
                String::from("project_name"),
                serde_json::json!("wave-autoscale-test"),
            ),
            (String::from("service_name"), serde_json::json!("service-1")),
        ]
        .into_iter()
        .collect();
        let scaling_component_definitions = vec![ScalingComponentDefinition {
            kind: ScalingComponent,
            db_id: String::from("db_id"),
            id: String::from("scaling_component_cloud_run_service"),
            component_kind: String::from("google-cloud-run"),
            metadata,
            ..Default::default()
        }];

        let mut scaling_component_manager = ScalingComponentManager::new();
        scaling_component_manager.add_definitions(scaling_component_definitions);

        if let Some(scaling_component) =
            scaling_component_manager.get_scaling_component("scaling_component_cloud_run_service")
        {
            let name = scaling_component.get_scaling_component_kind();
            assert!(
                name == CloudRunServiceScalingComponent::SCALING_KIND,
                "Unexpected"
            );
        } else {
            assert!(false, "No scaling component found")
        }

        let params: HashMap<String, serde_json::Value> = vec![
            (String::from("min_instance_count"), serde_json::json!(1)),
            (String::from("max_instance_count"), serde_json::json!(100)),
            (
                String::from("max_request_per_instance"),
                serde_json::json!(1),
            ),
            (
                String::from("execution_environment"),
                serde_json::json!("EXECUTION_ENVIRONMENT_GEN1"),
            ),
        ]
        .into_iter()
        .collect();

        let result = scaling_component_manager
            .apply_to(
                "scaling_component_cloud_run_service",
                params,
                get_rquickjs_context().await,
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn apply_service_with_max_request_per_instance_for_version_2_api() {
        let metadata: HashMap<String, serde_json::Value> = vec![
            (String::from("api_version"), serde_json::json!("v2")),
            (
                String::from("location_name"),
                serde_json::json!("asia-northeast2"),
            ),
            (
                String::from("project_name"),
                serde_json::json!("wave-autoscale-test"),
            ),
            (String::from("service_name"), serde_json::json!("service-1")),
        ]
        .into_iter()
        .collect();
        let scaling_component_definitions = vec![ScalingComponentDefinition {
            kind: ScalingComponent,
            db_id: String::from("db_id"),
            id: String::from("scaling_component_cloud_run_service"),
            component_kind: String::from("google-cloud-run"),
            metadata,
            ..Default::default()
        }];

        let mut scaling_component_manager = ScalingComponentManager::new();
        scaling_component_manager.add_definitions(scaling_component_definitions);

        if let Some(scaling_component) =
            scaling_component_manager.get_scaling_component("scaling_component_cloud_run_service")
        {
            let name = scaling_component.get_scaling_component_kind();
            assert!(
                name == CloudRunServiceScalingComponent::SCALING_KIND,
                "Unexpected"
            );
        } else {
            assert!(false, "No scaling component found")
        }

        let params: HashMap<String, serde_json::Value> = vec![
            (String::from("min_instance_count"), serde_json::json!(1)),
            (String::from("max_instance_count"), serde_json::json!(100)),
            (
                String::from("max_request_per_instance"),
                serde_json::json!(100),
            ),
            (
                String::from("execution_environment"),
                serde_json::json!("EXECUTION_ENVIRONMENT_UNSPECIFIED"),
            ),
        ]
        .into_iter()
        .collect();

        let result = scaling_component_manager
            .apply_to(
                "scaling_component_cloud_run_service",
                params,
                get_rquickjs_context().await,
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn apply_service_with_changed_instance_count_for_version_2_api() {
        let metadata: HashMap<String, serde_json::Value> = vec![
            (String::from("api_version"), serde_json::json!("v2")),
            (
                String::from("location_name"),
                serde_json::json!("asia-northeast2"),
            ),
            (
                String::from("project_name"),
                serde_json::json!("wave-autoscale-test"),
            ),
            (String::from("service_name"), serde_json::json!("service-1")),
        ]
        .into_iter()
        .collect();
        let scaling_component_definitions = vec![ScalingComponentDefinition {
            kind: ScalingComponent,
            db_id: String::from("db_id"),
            id: String::from("scaling_component_cloud_run_service"),
            component_kind: String::from("google-cloud-run"),
            metadata,
            ..Default::default()
        }];

        let mut scaling_component_manager = ScalingComponentManager::new();
        scaling_component_manager.add_definitions(scaling_component_definitions);

        if let Some(scaling_component) =
            scaling_component_manager.get_scaling_component("scaling_component_cloud_run_service")
        {
            let name = scaling_component.get_scaling_component_kind();
            assert!(
                name == CloudRunServiceScalingComponent::SCALING_KIND,
                "Unexpected"
            );
        } else {
            assert!(false, "No scaling component found")
        }

        let params: HashMap<String, serde_json::Value> = vec![
            (String::from("min_instance_count"), serde_json::json!(5)),
            (String::from("max_instance_count"), serde_json::json!(10)),
            (
                String::from("max_request_per_instance"),
                serde_json::json!(100),
            ),
            (
                String::from("execution_environment"),
                serde_json::json!("EXECUTION_ENVIRONMENT_UNSPECIFIED"),
            ),
        ]
        .into_iter()
        .collect();

        let result = scaling_component_manager
            .apply_to(
                "scaling_component_cloud_run_service",
                params,
                get_rquickjs_context().await,
            )
            .await;
        assert!(result.is_ok());
    }
}
