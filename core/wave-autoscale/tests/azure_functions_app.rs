mod google_cloud_run_service_test {
    use std::collections::HashMap;

    use anyhow::Result;

    use data_layer::types::object_kind::ObjectKind::ScalingComponent;
    use data_layer::ScalingComponentDefinition;
    use wave_autoscale::scaling_component::azure_functions_app::AzureFunctionsAppScalingComponent;
    use wave_autoscale::scaling_component::ScalingComponentManager;

    async fn get_rquickjs_context() -> rquickjs::AsyncContext {
        rquickjs::AsyncContext::full(&rquickjs::AsyncRuntime::new().unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    #[ignore]
    async fn apply_maximum_scale_out_limit_for_consumption_plan() {
        let metadata: HashMap<String, serde_json::Value> = vec![
            (String::from("client_id"), serde_json::json!("CLIENT_ID")),
            (
                String::from("client_secret"),
                serde_json::json!("CLIENT_SECRET"),
            ),
            (String::from("tenant_id"), serde_json::json!("TENANT_ID")),
            (
                String::from("subscription_id"),
                serde_json::json!("SUBSCRIPTION_ID"),
            ),
            (
                String::from("resource_group_name"),
                serde_json::json!("RESOURCE_GROUP_NAME"),
            ),
            (String::from("app_name"), serde_json::json!("APP_NAME")),
        ]
        .into_iter()
        .collect();
        let scaling_component_definitions = vec![ScalingComponentDefinition {
            kind: ScalingComponent,
            db_id: String::from("db_id"),
            id: String::from("scaling_component_azure_functions_app"),
            component_kind: String::from("azure-functions-app"),
            metadata,
            ..Default::default()
        }];

        let mut scaling_component_manager = ScalingComponentManager::new();
        scaling_component_manager.add_definitions(scaling_component_definitions);

        if let Some(scaling_component) =
            scaling_component_manager.get_scaling_component("scaling_component_azure_functions_app")
        {
            let name = scaling_component.get_scaling_component_kind();
            assert!(
                name == AzureFunctionsAppScalingComponent::SCALING_KIND,
                "Unexpected"
            );
        } else {
            assert!(false, "No scaling component found")
        }

        let params: HashMap<String, serde_json::Value> =
            vec![(String::from("max_instance_count"), serde_json::json!(200))]
                .into_iter()
                .collect();

        let result = scaling_component_manager
            .apply_to(
                "scaling_component_azure_functions_app",
                params,
                get_rquickjs_context().await,
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn apply_maximum_always_ready_instance_for_premium_plan() {
        let metadata: HashMap<String, serde_json::Value> = vec![
            (String::from("client_id"), serde_json::json!("CLIENT_ID")),
            (
                String::from("client_secret"),
                serde_json::json!("CLIENT_SECRET"),
            ),
            (String::from("tenant_id"), serde_json::json!("TENANT_ID")),
            (
                String::from("subscription_id"),
                serde_json::json!("SUBSCRIPTION_ID"),
            ),
            (
                String::from("resource_group_name"),
                serde_json::json!("RESOURCE_GROUP_NAME"),
            ),
            (String::from("app_name"), serde_json::json!("APP_NAME")),
        ]
        .into_iter()
        .collect();
        let scaling_component_definitions = vec![ScalingComponentDefinition {
            kind: ScalingComponent,
            db_id: String::from("db_id"),
            id: String::from("scaling_component_azure_functions_app"),
            component_kind: String::from("azure-functions-app"),
            metadata,
            ..Default::default()
        }];

        let mut scaling_component_manager = ScalingComponentManager::new();
        scaling_component_manager.add_definitions(scaling_component_definitions);

        if let Some(scaling_component) =
            scaling_component_manager.get_scaling_component("scaling_component_azure_functions_app")
        {
            let name = scaling_component.get_scaling_component_kind();
            assert!(
                name == AzureFunctionsAppScalingComponent::SCALING_KIND,
                "Unexpected"
            );
        } else {
            assert!(false, "No scaling component found")
        }

        let params: HashMap<String, serde_json::Value> = vec![
            (String::from("min_instance_count"), serde_json::json!(20)),
            (String::from("max_instance_count"), serde_json::json!(200)),
        ]
        .into_iter()
        .collect();

        let result = scaling_component_manager
            .apply_to(
                "scaling_component_azure_functions_app",
                params,
                get_rquickjs_context().await,
            )
            .await;
        assert!(result.is_ok());
    }
}
