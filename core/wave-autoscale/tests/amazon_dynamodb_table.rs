mod amazon_dynamodb_table_test {
    use std::collections::HashMap;

    use anyhow::Result;
    use data_layer::reader::wave_definition_reader::read_definition_yaml_file;
    use serde_json::json;

    use aws_config::SdkConfig;
    use aws_credential_types::Credentials;
    use aws_smithy_types::error::metadata::ProvideErrorMetadata;

    use aws_sdk_dynamodb::{
        types::{BillingMode, ProvisionedThroughput},
        Client as DynamoDbClient,
    };

    use aws_sdk_applicationautoscaling::{
        operation::describe_scaling_policies::DescribeScalingPoliciesOutput,
        types::{
            MetricType, MetricType::DynamoDbReadCapacityUtilization,
            MetricType::DynamoDbWriteCapacityUtilization, PolicyType,
            PredefinedMetricSpecification, ScalableDimension,
            ScalableDimension::DynamoDbTableReadCapacityUnits,
            ScalableDimension::DynamoDbTableWriteCapacityUnits, ServiceNamespace,
            TargetTrackingScalingPolicyConfiguration,
        },
        Client as ApplicationAutoScalingClient,
    };

    const DYNAMODB_TABLE_FILE_PATH: &str = "./tests/yaml/component_dynamodb_table.yaml";
    static STATIC_REGION: &str = "ap-northeast-3";

    #[tokio::test]
    async fn amazon_dynamodb_table_update() -> Result<()> {
        //let capacity_mode = "ON_DEMAND";

        let capacity_mode = "PROVISIONED";

        //let capacity_unit = "READ";
        //let capacity_unit = "WRITE";
        let capacity_unit = "READ_WRITE";

        /* let autoscaling_mode = "OFF"; */
        let read_capacity_units: Option<i64> = Some(3);
        let write_capacity_units: Option<i64> = Some(2);

        let autoscaling_mode = "ON";
        let read_target_value = 70.5;
        let read_min_capacity = 1;
        let read_max_capacity = 4;
        let write_target_value = 30.0;
        let write_min_capacity = 1;
        let write_max_capacity = 3;

        let parse_result = read_definition_yaml_file(DYNAMODB_TABLE_FILE_PATH).unwrap();

        #[derive(Debug)]
        enum CapacityMode {
            OnDemand,
            Provisioned,
        }
        impl CapacityMode {
            fn from_str(capacity_mode: Option<&str>) -> Option<Self> {
                match capacity_mode.map(|s| s.to_uppercase()).as_deref() {
                    Some("ON_DEMAND") => Some(CapacityMode::OnDemand),
                    Some("PROVISIONED") => Some(CapacityMode::Provisioned),
                    _ => None,
                }
            }
        }
        #[derive(Debug)]
        enum AutoscalingMode {
            On,
            Off,
        }
        impl AutoscalingMode {
            fn from_str(autoscaling_mode: Option<&str>) -> Option<Self> {
                match autoscaling_mode.map(|s| s.to_uppercase()).as_deref() {
                    Some("ON") => Some(AutoscalingMode::On),
                    Some("OFF") => Some(AutoscalingMode::Off),
                    _ => None,
                }
            }
        }
        #[derive(Debug)]
        enum CapacityUnit {
            Read,
            Write,
            ReadWrite,
        }
        impl CapacityUnit {
            fn from_str(capacity_unit: Option<&str>) -> Option<Self> {
                match capacity_unit.map(|s| s.to_uppercase()).as_deref() {
                    Some("READ") => Some(CapacityUnit::Read),
                    Some("WRITE") => Some(CapacityUnit::Write),
                    Some("READ_WRITE") => Some(CapacityUnit::ReadWrite),
                    _ => None,
                }
            }
        }
        #[derive(Debug)]
        enum DynamoDbScalingState {
            OnDemand,
            ProvisionedOnRead,
            ProvisionedOnWrite,
            ProvisionedOnReadWrite,
            ProvisionedOffRead,
            ProvisionedOffWrite,
            ProvisionedOffReadWrite,
        }
        impl DynamoDbScalingState {
            fn new(
                capacity_mode: Option<&str>,
                autoscaling_mode: Option<&str>,
                capacity_unit: Option<&str>,
            ) -> Option<Self> {
                match CapacityMode::from_str(capacity_mode) {
                    Some(CapacityMode::OnDemand) => Some(DynamoDbScalingState::OnDemand),
                    Some(CapacityMode::Provisioned) => match (
                        AutoscalingMode::from_str(autoscaling_mode),
                        CapacityUnit::from_str(capacity_unit),
                    ) {
                        (Some(AutoscalingMode::On), Some(CapacityUnit::Read)) => {
                            Some(DynamoDbScalingState::ProvisionedOnRead)
                        }
                        (Some(AutoscalingMode::On), Some(CapacityUnit::Write)) => {
                            Some(DynamoDbScalingState::ProvisionedOnWrite)
                        }
                        (Some(AutoscalingMode::On), Some(CapacityUnit::ReadWrite)) => {
                            Some(DynamoDbScalingState::ProvisionedOnReadWrite)
                        }
                        (Some(AutoscalingMode::Off), Some(CapacityUnit::Read)) => {
                            Some(DynamoDbScalingState::ProvisionedOffRead)
                        }
                        (Some(AutoscalingMode::Off), Some(CapacityUnit::Write)) => {
                            Some(DynamoDbScalingState::ProvisionedOffWrite)
                        }
                        (Some(AutoscalingMode::Off), Some(CapacityUnit::ReadWrite)) => {
                            Some(DynamoDbScalingState::ProvisionedOffReadWrite)
                        }
                        _ => None,
                    },
                    _ => None,
                }
            }
        }
        fn handle_dynamodb_scaling_state(
            capacity_mode: Option<&str>,
            autoscaling_mode: Option<&str>,
            capacity_unit: Option<&str>,
        ) -> Option<DynamoDbScalingState> {
            DynamoDbScalingState::new(capacity_mode, autoscaling_mode, capacity_unit)
        }

        async fn update_table_to_on_demand_mode(
            shared_config: &SdkConfig,
            table_name: &str,
        ) -> Result<(), anyhow::Error> {
            let client = DynamoDbClient::new(shared_config);
            // from provisioned mode to on-demand mode
            let result = client
                .update_table()
                .table_name(table_name)
                .billing_mode(BillingMode::PayPerRequest);
            let result = result.send().await;
            if let Err(error) = result {
                let meta = error.meta();
                let json = json!({
                    "message": meta.message(),
                    "code": meta.code(),
                    "extras": meta.to_string()
                });
                return Err(anyhow::anyhow!(json));
            }
            Ok(())
        }
        async fn update_table_to_provisioned_mode(
            shared_config: &SdkConfig,
            table_name: &str,
            read_capacity_units: Option<i64>,
            write_capacity_units: Option<i64>,
        ) -> Result<(), anyhow::Error> {
            let client = DynamoDbClient::new(shared_config);
            let mut result = client.update_table().table_name(table_name);

            match (read_capacity_units, write_capacity_units) {
                (Some(read_capacity), Some(write_capacity)) => {
                    result = result.provisioned_throughput(
                        ProvisionedThroughput::builder()
                            .read_capacity_units(read_capacity)
                            .write_capacity_units(write_capacity)
                            .build(),
                    );
                }
                (Some(read_capacity), None) => {
                    result = result.provisioned_throughput(
                        ProvisionedThroughput::builder()
                            .read_capacity_units(read_capacity)
                            .build(),
                    );
                }
                (None, Some(write_capacity)) => {
                    result = result.provisioned_throughput(
                        ProvisionedThroughput::builder()
                            .write_capacity_units(write_capacity)
                            .build(),
                    );
                }
                (None, None) => return Err(anyhow::anyhow!("No capacity units specified")),
            }

            let result = result.send().await;
            if let Err(error) = result {
                let meta = error.meta();
                let json = json!({
                    "message": meta.message(),
                    "code": meta.code(),
                    "extras": meta.to_string()
                });
                return Err(anyhow::anyhow!(json));
            }
            Ok(())
        }
        async fn describe_scaling_policies_from_table(
            shared_config: &SdkConfig,
            table_name: &str,
            scalable_dimension: ScalableDimension,
        ) -> Result<DescribeScalingPoliciesOutput, anyhow::Error> {
            let client = ApplicationAutoScalingClient::new(shared_config);
            let result = client
                .describe_scaling_policies()
                .set_scalable_dimension(Some(scalable_dimension))
                .set_service_namespace(Some(ServiceNamespace::Dynamodb))
                .set_resource_id(Some("table/".to_owned() + table_name));
            let result = result.send().await;
            if let Err(error) = result {
                let meta = error.meta();
                let json = json!({
                    "message": meta.message(),
                    "code": meta.code(),
                    "extras": meta.to_string()
                });
                Err(anyhow::anyhow!(json))
            } else {
                Ok(result?)
            }
        }
        async fn delete_scaling_policy_from_table(
            shared_config: &SdkConfig,
            table_name: &str,
            scalable_dimension: ScalableDimension,
        ) -> Result<(), anyhow::Error> {
            let client = ApplicationAutoScalingClient::new(shared_config);
            let result = client
                .delete_scaling_policy()
                .policy_name("$".to_owned() + table_name + "-scaling-policy")
                .set_scalable_dimension(Some(scalable_dimension))
                .set_service_namespace(Some(ServiceNamespace::Dynamodb))
                .set_resource_id(Some("table/".to_owned() + table_name));
            let result = result.send().await;
            if let Err(error) = result {
                let meta = error.meta();
                let json = json!({
                    "message": meta.message(),
                    "code": meta.code(),
                    "extras": meta.to_string()
                });
                return Err(anyhow::anyhow!(json));
            }
            Ok(())
        }
        async fn describe_and_delete_scaling_policy(
            shared_config: &SdkConfig,
            table_name: &str,
            scalable_dimension: ScalableDimension,
        ) -> Result<(), anyhow::Error> {
            if let Some(scaling_policies) = describe_scaling_policies_from_table(
                shared_config,
                table_name,
                scalable_dimension.clone(),
            )
            .await?
            .scaling_policies
            {
                if !scaling_policies.is_empty() {
                    delete_scaling_policy_from_table(shared_config, table_name, scalable_dimension)
                        .await?;
                }
            }
            Ok(())
        }
        async fn put_scaling_policy_to_plan(
            shared_config: &SdkConfig,
            table_name: &str,
            scalable_dimension: ScalableDimension,
            target_value: f64,
            predefined_metric_type: MetricType,
        ) -> Result<(), anyhow::Error> {
            let client = ApplicationAutoScalingClient::new(shared_config);
            let result = client
                .put_scaling_policy()
                .policy_name("$".to_owned() + table_name + "-scaling-policy")
                .set_policy_type(Some(PolicyType::TargetTrackingScaling))
                .set_scalable_dimension(Some(scalable_dimension))
                .set_service_namespace(Some(ServiceNamespace::Dynamodb))
                .set_resource_id(Some("table/".to_owned() + table_name))
                .set_target_tracking_scaling_policy_configuration(Some(
                    TargetTrackingScalingPolicyConfiguration::builder()
                        .set_target_value(Some(target_value))
                        .set_predefined_metric_specification(Some(
                            PredefinedMetricSpecification::builder()
                                .set_predefined_metric_type(Some(predefined_metric_type))
                                .build(),
                        ))
                        .build(),
                ));
            let result = result.send().await;
            if let Err(error) = result {
                let meta = error.meta();
                let json = json!({
                    "message": meta.message(),
                    "code": meta.code(),
                    "extras": meta.to_string()
                });
                return Err(anyhow::anyhow!(json));
            }
            Ok(())
        }
        async fn register_scalable_target_to_table(
            shared_config: &SdkConfig,
            table_name: &str,
            min_capacity: i32,
            max_capacity: i32,
            scalable_dimension: ScalableDimension,
        ) -> Result<(), anyhow::Error> {
            let client = ApplicationAutoScalingClient::new(shared_config);
            let result = client
                .register_scalable_target()
                .set_service_namespace(Some(ServiceNamespace::Dynamodb))
                .set_resource_id(Some("table/".to_owned() + table_name))
                .set_min_capacity(Some(min_capacity))
                .set_max_capacity(Some(max_capacity))
                .set_scalable_dimension(Some(scalable_dimension));
            let result = result.send().await;
            if let Err(error) = result {
                let meta = error.meta();
                let json = json!({
                    "message": meta.message(),
                    "code": meta.code(),
                    "extras": meta.to_string()
                });
                return Err(anyhow::anyhow!(json));
            }
            Ok(())
        }

        for scaling_component_definition in parse_result.scaling_component_definitions.clone() {
            let metadata: HashMap<String, serde_json::Value> =
                scaling_component_definition.metadata;

            if let (
                Some(serde_json::Value::String(access_key)),
                Some(serde_json::Value::String(secret_key)),
                Some(serde_json::Value::String(_region)),
                Some(serde_json::Value::String(table_name)),
            ) = (
                metadata.get("access_key"),
                metadata.get("secret_key"),
                metadata.get("region"),
                metadata.get("table_name"),
            ) {
                let credentials =
                    Credentials::new(access_key, secret_key, None, None, "wave-autoscale");
                // aws_config needs a static region string
                let shared_config = aws_config::from_env()
                    .region(STATIC_REGION)
                    .credentials_provider(credentials)
                    .load()
                    .await;
                let handle_dynamodb_scaling_state = handle_dynamodb_scaling_state(
                    Some(capacity_mode),
                    Some(autoscaling_mode),
                    Some(capacity_unit),
                );
                match handle_dynamodb_scaling_state {
                    Some(DynamoDbScalingState::OnDemand) => {
                        update_table_to_on_demand_mode(&shared_config, table_name).await?;
                    }
                    Some(DynamoDbScalingState::ProvisionedOnRead) => {
                        register_scalable_target_to_table(
                            &shared_config,
                            table_name,
                            read_min_capacity,
                            read_max_capacity,
                            DynamoDbTableReadCapacityUnits,
                        )
                        .await?;
                        put_scaling_policy_to_plan(
                            &shared_config,
                            table_name,
                            DynamoDbTableReadCapacityUnits,
                            read_target_value,
                            DynamoDbReadCapacityUtilization,
                        )
                        .await?;
                    }
                    Some(DynamoDbScalingState::ProvisionedOnWrite) => {
                        register_scalable_target_to_table(
                            &shared_config,
                            table_name,
                            write_min_capacity,
                            write_max_capacity,
                            DynamoDbTableWriteCapacityUnits,
                        )
                        .await?;
                        put_scaling_policy_to_plan(
                            &shared_config,
                            table_name,
                            DynamoDbTableWriteCapacityUnits,
                            write_target_value,
                            DynamoDbWriteCapacityUtilization,
                        )
                        .await?;
                    }
                    Some(DynamoDbScalingState::ProvisionedOnReadWrite) => {
                        register_scalable_target_to_table(
                            &shared_config,
                            table_name,
                            read_min_capacity,
                            read_max_capacity,
                            DynamoDbTableReadCapacityUnits,
                        )
                        .await?;
                        put_scaling_policy_to_plan(
                            &shared_config,
                            table_name,
                            DynamoDbTableReadCapacityUnits,
                            read_target_value,
                            DynamoDbReadCapacityUtilization,
                        )
                        .await?;
                        register_scalable_target_to_table(
                            &shared_config,
                            table_name,
                            write_min_capacity,
                            write_max_capacity,
                            DynamoDbTableWriteCapacityUnits,
                        )
                        .await?;
                        put_scaling_policy_to_plan(
                            &shared_config,
                            table_name,
                            DynamoDbTableWriteCapacityUnits,
                            write_target_value,
                            DynamoDbWriteCapacityUtilization,
                        )
                        .await?;
                    }
                    Some(DynamoDbScalingState::ProvisionedOffRead) => {
                        describe_and_delete_scaling_policy(
                            &shared_config,
                            table_name,
                            DynamoDbTableReadCapacityUnits,
                        )
                        .await?;
                        update_table_to_provisioned_mode(
                            &shared_config,
                            table_name,
                            read_capacity_units,
                            None,
                        )
                        .await?;
                    }
                    Some(DynamoDbScalingState::ProvisionedOffWrite) => {
                        describe_and_delete_scaling_policy(
                            &shared_config,
                            table_name,
                            DynamoDbTableWriteCapacityUnits,
                        )
                        .await?;
                        update_table_to_provisioned_mode(
                            &shared_config,
                            table_name,
                            None,
                            write_capacity_units,
                        )
                        .await?;
                    }
                    Some(DynamoDbScalingState::ProvisionedOffReadWrite) => {
                        describe_and_delete_scaling_policy(
                            &shared_config,
                            table_name,
                            DynamoDbTableReadCapacityUnits,
                        )
                        .await?;
                        describe_and_delete_scaling_policy(
                            &shared_config,
                            table_name,
                            DynamoDbTableWriteCapacityUnits,
                        )
                        .await?;
                        update_table_to_provisioned_mode(
                            &shared_config,
                            table_name,
                            read_capacity_units,
                            write_capacity_units,
                        )
                        .await?;
                    }
                    _ => {
                        return Err(anyhow::anyhow!("Invalid parameter"));
                    }
                }
            } else {
                return Err(anyhow::anyhow!("Missing required metadata"));
            }
        }
        Ok(())
    }
}
