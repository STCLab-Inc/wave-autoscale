mod test_gcp_mig_scaling {
    use anyhow::Result;
    use data_layer::types::object_kind::ObjectKind;
    use data_layer::ScalingComponentDefinition;
    use serde_json::json;
    use std::collections::HashMap;
    use wave_autoscale::scaling_component::{
        gcp_mig_autoscaling::MIGAutoScalingComponent, ScalingComponentManager,
    };

    async fn get_rquickjs_context() -> rquickjs::AsyncContext {
        rquickjs::AsyncContext::full(&rquickjs::AsyncRuntime::new().unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    #[ignore]
    async fn test_gcp_mig_autoscaling() {
        let mut scaling_component_metadata = HashMap::new();
        scaling_component_metadata.insert(
            "project".to_string(),
            serde_json::Value::String("wave-autoscale-test".to_string()),
        );
        scaling_component_metadata.insert(
            "location_kind".to_string(),
            serde_json::Value::String("region".to_string()),
        );
        scaling_component_metadata.insert(
            "location_name".to_string(),
            serde_json::Value::String("asia-northeast2".to_string()),
        );
        scaling_component_metadata.insert(
            "group_name".to_string(),
            serde_json::Value::String("test-instance-group-1".to_string()),
        );

        let scaling_component_definitions = vec![ScalingComponentDefinition {
            kind: ObjectKind::ScalingComponent,
            db_id: "".to_string(),
            id: "gcp_mig_region_autoscaling_api_server".to_string(),
            component_kind: "gcp-compute-engine-mig".to_string(),
            metadata: scaling_component_metadata,
            ..Default::default()
        }];

        // create metric adapter
        let mut scaling_component_manager = ScalingComponentManager::new();
        scaling_component_manager.add_definitions(scaling_component_definitions);

        if let Some(scaling_component) =
            scaling_component_manager.get_scaling_component("gcp_mig_region_autoscaling_api_server")
        {
            let name = scaling_component.get_scaling_component_kind();
            assert!(name == MIGAutoScalingComponent::SCALING_KIND, "Unexpected");
        } else {
            assert!(false, "No scaling component found")
        }

        // run scaling trigger
        let mut options: HashMap<String, serde_json::Value> = HashMap::new();
        options.insert("resize".to_string(), json!(2));

        let result = scaling_component_manager
            .apply_to(
                "gcp_mig_region_autoscaling_api_server",
                options,
                get_rquickjs_context().await,
            )
            .await;
        assert!(result.is_ok());
    }
}
