mod google_cloud_functions_instance_test {
    use std::collections::HashMap;

    use anyhow::Result;

    use data_layer::types::object_kind::ObjectKind::ScalingComponent;
    use data_layer::ScalingComponentDefinition;
    use wave_autoscale::scaling_component::google_cloud_functions_instance::CloudFunctionsInstanceScalingComponent;
    use wave_autoscale::scaling_component::ScalingComponentManager;

    async fn get_rquickjs_context() -> rquickjs::AsyncContext {
        rquickjs::AsyncContext::full(&rquickjs::AsyncRuntime::new().unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    #[ignore]
    async fn apply_instance_change_for_version_1_function() {
        let metadata: HashMap<String, serde_json::Value> = vec![
            (String::from("function_version"), serde_json::json!("v1")),
            (
                String::from("project_name"),
                serde_json::json!("wave-autoscale-test"),
            ),
            (
                String::from("location_name"),
                serde_json::json!("asia-northeast2"),
            ),
            (
                String::from("function_name"),
                serde_json::json!("function-1"),
            ),
        ]
        .into_iter()
        .collect();
        let scaling_component_definitions = vec![ScalingComponentDefinition {
            kind: ScalingComponent,
            db_id: String::from("db_id"),
            id: String::from("scaling_component_cloud_functions_instance"),
            component_kind: String::from("google-cloud-functions"),
            metadata,
            ..Default::default()
        }];

        let mut scaling_component_manager = ScalingComponentManager::new();
        scaling_component_manager.add_definitions(scaling_component_definitions);

        if let Some(scaling_component) = scaling_component_manager
            .get_scaling_component("scaling_component_cloud_functions_instance")
        {
            let name = scaling_component.get_scaling_component_kind();
            assert!(
                name == CloudFunctionsInstanceScalingComponent::SCALING_KIND,
                "Unexpected"
            );
        } else {
            assert!(false, "No scaling component found")
        }

        let params: HashMap<String, serde_json::Value> = vec![
            (String::from("min_instance_count"), serde_json::json!(4)),
            (String::from("max_instance_count"), serde_json::json!(5)),
        ]
        .into_iter()
        .collect();

        let result = scaling_component_manager
            .apply_to(
                "scaling_component_cloud_functions_instance",
                params,
                get_rquickjs_context().await,
            )
            .await;
        assert!(result.is_ok());
    }

    #[ignore]
    #[tokio::test]
    async fn apply_instance_change_for_version_2_function() {
        let metadata: HashMap<String, serde_json::Value> = vec![
            (String::from("function_version"), serde_json::json!("v2")),
            (
                String::from("project_name"),
                serde_json::json!("wave-autoscale-test"),
            ),
            (
                String::from("location_name"),
                serde_json::json!("asia-northeast2"),
            ),
            (
                String::from("function_name"),
                serde_json::json!("function-2"),
            ),
        ]
        .into_iter()
        .collect();
        let scaling_component_definitions = vec![ScalingComponentDefinition {
            kind: ScalingComponent,
            db_id: String::from("db_id"),
            id: String::from("scaling_component_cloud_functions_instance"),
            component_kind: String::from("google-cloud-functions"),
            metadata,
            ..Default::default()
        }];

        let mut scaling_component_manager = ScalingComponentManager::new();
        scaling_component_manager.add_definitions(scaling_component_definitions);

        if let Some(scaling_component) = scaling_component_manager
            .get_scaling_component("scaling_component_cloud_functions_instance")
        {
            let name = scaling_component.get_scaling_component_kind();
            assert!(
                name == CloudFunctionsInstanceScalingComponent::SCALING_KIND,
                "Unexpected"
            );
        } else {
            assert!(false, "No scaling component found")
        }

        let params: HashMap<String, serde_json::Value> = vec![
            (String::from("min_instance_count"), serde_json::json!(4)),
            (String::from("max_instance_count"), serde_json::json!(5)),
            (
                String::from("max_instance_request_concurrency"),
                serde_json::json!(5),
            ),
        ]
        .into_iter()
        .collect();

        let result = scaling_component_manager
            .apply_to(
                "scaling_component_cloud_functions_instance",
                params,
                get_rquickjs_context().await,
            )
            .await;
        assert!(result.is_ok());
    }
}
