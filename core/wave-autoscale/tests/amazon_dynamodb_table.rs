mod amazon_dynamodb_table_test {
    use std::collections::HashMap;

    use anyhow::Result;

    use data_layer::types::object_kind::ObjectKind::ScalingComponent;
    use data_layer::ScalingComponentDefinition;
    use wave_autoscale::scaling_component::amazon_dynamodb_table::DynamoDbTableScalingComponent;
    use wave_autoscale::scaling_component::ScalingComponentManager;

    #[tokio::test]
    #[ignore]
    async fn apply_provisioned_off_write() -> Result<()> {
        let metadata: HashMap<String, serde_json::Value> = vec![
            (String::from("region"), serde_json::json!("region")),
            (String::from("access_key"), serde_json::json!("access_key")),
            (String::from("secret_key"), serde_json::json!("secret_key")),
            (String::from("table_name"), serde_json::json!("table_name")),
        ]
        .into_iter()
        .collect();
        let scaling_component_definitions = vec![ScalingComponentDefinition {
            kind: ScalingComponent,
            db_id: String::from("db_id"),
            id: String::from("scaling_component_dynamodb_table"),
            component_kind: String::from("amazon-dynamodb"),
            metadata,
            ..Default::default()
        }];

        let mut scaling_component_manager = ScalingComponentManager::new();
        scaling_component_manager.add_definitions(scaling_component_definitions);

        if let Some(scaling_component) =
            scaling_component_manager.get_scaling_component("scaling_component_dynamodb_table")
        {
            let name = scaling_component.get_scaling_component_kind();
            assert!(
                name == DynamoDbTableScalingComponent::SCALING_KIND,
                "Unexpected"
            );
        } else {
            assert!(false, "No scaling component found")
        }

        let params: HashMap<String, serde_json::Value> = vec![
            (
                String::from("capacity_mode"),
                serde_json::json!("PROVISIONED"),
            ),
            (String::from("autoscaling_mode"), serde_json::json!("OFF")),
            (String::from("capacity_unit"), serde_json::json!("WRITE")),
            (String::from("write_capacity_units"), serde_json::json!(7)),
        ]
        .into_iter()
        .collect();

        scaling_component_manager
            .apply_to("scaling_component_dynamodb_table", params)
            .await
    }
}
