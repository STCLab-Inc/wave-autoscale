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
            MetricType, PolicyType, PredefinedMetricSpecification, ScalableDimension,
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
        //let capacity_mode = "PAY_PER_REQUEST";

        let capacity_mode = "PROVISIONED";

        //let capacity_unit = "READ";
        //let capacity_unit = "WRITE";
        let capacity_unit = "BOTH";

        /* let autoscaling_mode = "OFF"; */
        let read_capacity_units = 3;
        let write_capacity_units = 3;

        let autoscaling_mode = "ON";
        let read_target_value = 70.5;
        let read_min_capacity = 1;
        let read_max_capacity = 4;
        let write_target_value = 30.0;
        let write_min_capacity = 1;
        let write_max_capacity = 3;

        let parse_result = read_definition_yaml_file(DYNAMODB_TABLE_FILE_PATH).unwrap();

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
        /* async fn describe_scalable_targets_from_table(
            shared_config: &SdkConfig,
            table_name: &str,
        ) -> Result<(), anyhow::Error> {
            let client: ApplicationAutoScalingClient =
                ApplicationAutoScalingClient::new(shared_config);
            let result = client
                .describe_scalable_targets()
                .service_namespace(ServiceNamespace::Dynamodb)
                .resource_ids("table/".to_owned() + table_name);
            let result = result.send().await;
            if let Err(error) = result {
                let meta = error.meta();
                let json = json!({
                    "message": meta.message(),
                    "code": meta.code(),
                    "extras": meta.to_string()
                });
                return Err(anyhow::anyhow!(json));
            } else {
                println!("{:?}", result);
            }
            Ok(())
        } */
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
                let provider_name = "wave-autoscale";
                let credentials =
                    Credentials::new(access_key, secret_key, None, None, provider_name);
                let shared_config = aws_config::from_env()
                    .region(STATIC_REGION)
                    .credentials_provider(credentials)
                    .load()
                    .await;

                match capacity_mode {
                    "PAY_PER_REQUEST" => {
                        update_table_to_on_demand_mode(&shared_config, table_name).await?;
                    }
                    "PROVISIONED" => match autoscaling_mode {
                        "ON" => match capacity_unit {
                            "READ" => {
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
                                    MetricType::DynamoDbReadCapacityUtilization,
                                )
                                .await?;
                            }
                            "WRITE" => {
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
                                    MetricType::DynamoDbWriteCapacityUtilization,
                                )
                                .await?;
                            }
                            "BOTH" => {
                                register_scalable_target_to_table(
                                    &shared_config,
                                    table_name,
                                    read_min_capacity,
                                    read_max_capacity,
                                    DynamoDbTableReadCapacityUnits,
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
                                    DynamoDbTableReadCapacityUnits,
                                    read_target_value,
                                    MetricType::DynamoDbReadCapacityUtilization,
                                )
                                .await?;
                                put_scaling_policy_to_plan(
                                    &shared_config,
                                    table_name,
                                    DynamoDbTableWriteCapacityUnits,
                                    write_target_value,
                                    MetricType::DynamoDbWriteCapacityUtilization,
                                )
                                .await?;
                            }
                            _ => {
                                return Err(anyhow::anyhow!("Invalid capacity unit"));
                            }
                        },
                        "OFF" => match capacity_unit {
                            "READ" => {
                                describe_and_delete_scaling_policy(
                                    &shared_config,
                                    table_name,
                                    DynamoDbTableReadCapacityUnits,
                                )
                                .await?;
                                update_table_to_provisioned_mode(
                                    &shared_config,
                                    table_name,
                                    Some(read_capacity_units),
                                    None,
                                )
                                .await?;
                            }
                            "WRITE" => {
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
                                    Some(write_capacity_units),
                                )
                                .await?;
                            }
                            "BOTH" => {
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
                                    Some(read_capacity_units),
                                    Some(write_capacity_units),
                                )
                                .await?;
                            }
                            _ => {
                                return Err(anyhow::anyhow!("Invalid capacity unit"));
                            }
                        },
                        _ => {
                            return Err(anyhow::anyhow!("Invalid autoscaling mode"));
                        }
                    },
                    _ => {
                        return Err(anyhow::anyhow!("Invalid capacity mode"));
                    }
                }
            } else {
                return Err(anyhow::anyhow!("Missing required metadata"));
            }
        }
        Ok(())
    }
}
