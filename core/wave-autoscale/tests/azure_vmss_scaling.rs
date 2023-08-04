mod test_azure_vmss_scaling {
    use data_layer::types::object_kind::ObjectKind;
    use data_layer::ScalingComponentDefinition;
    use serde_json::json;
    use std::collections::HashMap;
    use wave_autoscale::scaling_component::{
        azure_vmss_autoscaling::VMSSAutoScalingComponent, ScalingComponentManager,
    };

    fn get_test_env_data() -> (String, String, String, String) {
        (
            std::env::var("AZURE_CLIENT_ID").unwrap(),
            std::env::var("AZURE_CLIENT_SECRET").unwrap(),
            std::env::var("AZURE_TENANT_ID").unwrap(),
            std::env::var("AZURE_SUBSCRIPTION_ID").unwrap(),
        )
    }

    #[tokio::test]
    #[ignore]
    async fn test_azure_vmss_autoscaling() {
        let mut scaling_component_metadata = HashMap::new();
        scaling_component_metadata.insert(
            "client_id".to_string(),
            serde_json::Value::String(get_test_env_data().0),
        );
        scaling_component_metadata.insert(
            "client_secret".to_string(),
            serde_json::Value::String(get_test_env_data().1),
        );
        scaling_component_metadata.insert(
            "tenant_id".to_string(),
            serde_json::Value::String(get_test_env_data().2),
        );
        scaling_component_metadata.insert(
            "subscription_id".to_string(),
            serde_json::Value::String(get_test_env_data().3),
        );
        scaling_component_metadata.insert(
            "resource_group_name".to_string(),
            serde_json::Value::String("test-vmss-uniform-grp".to_string()),
        );
        scaling_component_metadata.insert(
            "vm_scale_set_name".to_string(),
            serde_json::Value::String("test-vmss-uniform".to_string()),
        );

        let scaling_component_definitions = vec![ScalingComponentDefinition {
            kind: ObjectKind::ScalingComponent,
            db_id: "".to_string(),
            id: "azure_vmss_autoscaling_api_server".to_string(),
            component_kind: "azure-virtual-machine-scale-sets".to_string(),
            metadata: scaling_component_metadata,
        }];

        // create metric adapter
        let mut scaling_component_manager = ScalingComponentManager::new();
        scaling_component_manager.add_definitions(scaling_component_definitions);

        if let Some(scaling_component) =
            scaling_component_manager.get_scaling_component("azure_vmss_autoscaling_api_server")
        {
            let name = scaling_component.get_scaling_component_kind();
            assert!(name == VMSSAutoScalingComponent::SCALING_KIND, "Unexpected");
        } else {
            assert!(false, "No scaling component found");
        }

        // run scaling trigger
        let mut options: HashMap<String, serde_json::Value> = HashMap::new();
        options.insert("capacity".to_string(), json!(1));

        let result = scaling_component_manager
            .apply_to("azure_vmss_autoscaling_api_server", options)
            .await;
        assert!(result.is_ok());
    }
}
