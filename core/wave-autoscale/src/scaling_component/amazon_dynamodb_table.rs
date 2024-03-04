use super::ScalingComponent;
use crate::util::aws::get_aws_config;
use anyhow::{Ok, Result};
use async_trait::async_trait;

use aws_config::SdkConfig;

use aws_smithy_types::error::metadata::ProvideErrorMetadata;

use aws_sdk_dynamodb::{
    operation::describe_table::DescribeTableOutput,
    types::{BillingMode, ProvisionedThroughput},
    Client as DynamoDbClient,
};

use aws_sdk_applicationautoscaling::{
    operation::describe_scaling_policies::DescribeScalingPoliciesOutput,
    types::{
        MetricType, MetricType::DynamoDbReadCapacityUtilization,
        MetricType::DynamoDbWriteCapacityUtilization, PolicyType, PredefinedMetricSpecification,
        ScalableDimension, ScalableDimension::DynamoDbTableReadCapacityUnits,
        ScalableDimension::DynamoDbTableWriteCapacityUnits, ServiceNamespace,
        TargetTrackingScalingPolicyConfiguration,
    },
    Client as ApplicationAutoScalingClient,
};

use data_layer::ScalingComponentDefinition;
use serde_json::json;
use std::collections::HashMap;

pub struct DynamoDbTableScalingComponent {
    definition: ScalingComponentDefinition,
}

impl DynamoDbTableScalingComponent {
    // Static variables
    pub const SCALING_KIND: &'static str = "amazon-dynamodb";

    // Functions
    pub fn new(definition: ScalingComponentDefinition) -> Self {
        DynamoDbTableScalingComponent { definition }
    }
}

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
    let mut result = client
        .update_table()
        .table_name(table_name)
        .billing_mode(BillingMode::Provisioned);

    match (read_capacity_units, write_capacity_units) {
        (Some(read_capacity_units), Some(write_capacity_units)) => {
            result = result.provisioned_throughput(
                ProvisionedThroughput::builder()
                    .read_capacity_units(read_capacity_units)
                    .write_capacity_units(write_capacity_units)
                    .build(),
            );
        }
        (Some(read_capacity_units), None) => {
            result = result.provisioned_throughput(
                ProvisionedThroughput::builder()
                    .read_capacity_units(read_capacity_units)
                    .build(),
            );
        }
        (None, Some(write_capacity_units)) => {
            result = result.provisioned_throughput(
                ProvisionedThroughput::builder()
                    .write_capacity_units(write_capacity_units)
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
async fn describe_data_from_table(
    shared_config: &SdkConfig,
    table_name: &str,
) -> Result<DescribeTableOutput, anyhow::Error> {
    let client = DynamoDbClient::new(shared_config);
    let result = client.describe_table().table_name(table_name);
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
async fn update_recent_table_to_provisioned_mode(
    shared_config: &SdkConfig,
    table_name: &str,
    read_capacity_units: Option<i64>,
    write_capacity_units: Option<i64>,
) -> Result<(), anyhow::Error> {
    match (read_capacity_units, write_capacity_units) {
        (Some(read_capacity_units), Some(write_capacity_units)) => {
            update_table_to_provisioned_mode(
                shared_config,
                table_name,
                Some(read_capacity_units),
                Some(write_capacity_units),
            )
            .await?;
        }
        (Some(read_capacity_units), None) => {
            update_table_to_provisioned_mode(
                shared_config,
                table_name,
                Some(read_capacity_units),
                describe_data_from_table(shared_config, table_name)
                    .await?
                    .table
                    .unwrap()
                    .provisioned_throughput()
                    .unwrap()
                    .write_capacity_units(),
            )
            .await?;
        }
        (None, Some(write_capacity_units)) => {
            update_table_to_provisioned_mode(
                shared_config,
                table_name,
                describe_data_from_table(shared_config, table_name)
                    .await?
                    .table
                    .unwrap()
                    .provisioned_throughput()
                    .unwrap()
                    .read_capacity_units(),
                Some(write_capacity_units),
            )
            .await?;
        }
        (None, None) => {
            update_table_to_provisioned_mode(
                shared_config,
                table_name,
                describe_data_from_table(shared_config, table_name)
                    .await?
                    .table
                    .unwrap()
                    .provisioned_throughput()
                    .unwrap()
                    .read_capacity_units(),
                describe_data_from_table(shared_config, table_name)
                    .await?
                    .table
                    .unwrap()
                    .provisioned_throughput()
                    .unwrap()
                    .write_capacity_units(),
            )
            .await?;
        }
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
    if let Some(scaling_policies) =
        describe_scaling_policies_from_table(shared_config, table_name, scalable_dimension.clone())
            .await?
            .scaling_policies
    {
        if !scaling_policies.is_empty() {
            delete_scaling_policy_from_table(shared_config, table_name, scalable_dimension).await?;
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

#[async_trait]
impl ScalingComponent for DynamoDbTableScalingComponent {
    fn get_scaling_component_kind(&self) -> &str {
        &self.definition.component_kind
    }
    fn get_id(&self) -> &str {
        &self.definition.id
    }
    async fn apply(
        &self,
        params: HashMap<String, serde_json::Value>,
        _context: rquickjs::AsyncContext,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let metadata: HashMap<String, serde_json::Value> = self.definition.metadata.clone();

        if let (
            Some(serde_json::Value::String(region)),
            Some(serde_json::Value::String(table_name)),
            capacity_mode,
            autoscaling_mode,
            capacity_unit,
            read_capacity_units,
            write_capacity_units,
            read_target_value,
            read_min_capacity,
            read_max_capacity,
            write_target_value,
            write_min_capacity,
            write_max_capacity,
        ) = (
            metadata.get("region"),
            metadata.get("table_name"),
            params
                .get("capacity_mode")
                .and_then(serde_json::Value::as_str)
                .map(|s| s as &str),
            params
                .get("autoscaling_mode")
                .and_then(serde_json::Value::as_str)
                .map(|s| s as &str),
            params
                .get("capacity_unit")
                .and_then(serde_json::Value::as_str)
                .map(|s| s as &str),
            params
                .get("read_capacity_units")
                .and_then(serde_json::Value::as_i64),
            params
                .get("write_capacity_units")
                .and_then(serde_json::Value::as_i64),
            params
                .get("read_target_value")
                .and_then(serde_json::Value::as_f64),
            params
                .get("read_min_capacity")
                .and_then(serde_json::Value::as_i64)
                .map(|v| v as i32),
            params
                .get("read_max_capacity")
                .and_then(serde_json::Value::as_i64)
                .map(|v| v as i32),
            params
                .get("write_target_value")
                .and_then(serde_json::Value::as_f64),
            params
                .get("write_min_capacity")
                .and_then(serde_json::Value::as_i64)
                .map(|v| v as i32),
            params
                .get("write_max_capacity")
                .and_then(serde_json::Value::as_i64)
                .map(|v| v as i32),
        ) {
            let access_key = metadata
                .get("access_key")
                .map(|access_key| access_key.to_string());
            let secret_key = metadata
                .get("secret_key")
                .map(|secret_key| secret_key.to_string());

            let config =
                get_aws_config(Some(region.to_string()), access_key, secret_key, None, None).await;
            if config.is_err() {
                let config_err = config.err().unwrap();
                return Err(anyhow::anyhow!(config_err));
            }
            let shared_config = config.unwrap();

            let handle_dynamodb_scaling_state =
                handle_dynamodb_scaling_state(capacity_mode, autoscaling_mode, capacity_unit);
            match handle_dynamodb_scaling_state {
                Some(DynamoDbScalingState::OnDemand) => {
                    update_table_to_on_demand_mode(&shared_config, table_name).await?;
                }
                Some(DynamoDbScalingState::ProvisionedOnRead) => {
                    update_table_to_provisioned_mode(
                        &shared_config,
                        table_name,
                        read_capacity_units,
                        None,
                    )
                    .await?;
                    register_scalable_target_to_table(
                        &shared_config,
                        table_name,
                        read_min_capacity.unwrap(),
                        read_max_capacity.unwrap(),
                        DynamoDbTableReadCapacityUnits,
                    )
                    .await?;
                    put_scaling_policy_to_plan(
                        &shared_config,
                        table_name,
                        DynamoDbTableReadCapacityUnits,
                        read_target_value.unwrap(),
                        DynamoDbReadCapacityUtilization,
                    )
                    .await?;
                }
                Some(DynamoDbScalingState::ProvisionedOnWrite) => {
                    update_table_to_provisioned_mode(
                        &shared_config,
                        table_name,
                        None,
                        write_capacity_units,
                    )
                    .await?;
                    register_scalable_target_to_table(
                        &shared_config,
                        table_name,
                        write_min_capacity.unwrap(),
                        write_max_capacity.unwrap(),
                        DynamoDbTableWriteCapacityUnits,
                    )
                    .await?;
                    put_scaling_policy_to_plan(
                        &shared_config,
                        table_name,
                        DynamoDbTableWriteCapacityUnits,
                        write_target_value.unwrap(),
                        DynamoDbWriteCapacityUtilization,
                    )
                    .await?;
                }
                Some(DynamoDbScalingState::ProvisionedOnReadWrite) => {
                    update_table_to_provisioned_mode(
                        &shared_config,
                        table_name,
                        read_capacity_units,
                        write_capacity_units,
                    )
                    .await?;
                    register_scalable_target_to_table(
                        &shared_config,
                        table_name,
                        read_min_capacity.unwrap(),
                        read_max_capacity.unwrap(),
                        DynamoDbTableReadCapacityUnits,
                    )
                    .await?;
                    put_scaling_policy_to_plan(
                        &shared_config,
                        table_name,
                        DynamoDbTableReadCapacityUnits,
                        read_target_value.unwrap(),
                        DynamoDbReadCapacityUtilization,
                    )
                    .await?;
                    register_scalable_target_to_table(
                        &shared_config,
                        table_name,
                        write_min_capacity.unwrap(),
                        write_max_capacity.unwrap(),
                        DynamoDbTableWriteCapacityUnits,
                    )
                    .await?;
                    put_scaling_policy_to_plan(
                        &shared_config,
                        table_name,
                        DynamoDbTableWriteCapacityUnits,
                        write_target_value.unwrap(),
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
                    update_recent_table_to_provisioned_mode(
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
                    update_recent_table_to_provisioned_mode(
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
            return Err(anyhow::anyhow!("Invalid metadata"));
        }
        Ok(params)
    }
}

#[cfg(test)]
mod test {
    use super::DynamoDbTableScalingComponent;
    use crate::scaling_component::test::get_rquickjs_context;
    use crate::scaling_component::ScalingComponent;
    use data_layer::ScalingComponentDefinition;
    use std::collections::HashMap;

    // capacity_mode: ON_DEMAND -> ok
    #[tokio::test]
    #[ignore]
    async fn apply_on_demand() {
        let metadata: HashMap<String, serde_json::Value> = vec![
            (String::from("region"), serde_json::json!("region")),
            (String::from("access_key"), serde_json::json!("access_key")),
            (String::from("secret_key"), serde_json::json!("secret_key")),
            (String::from("table_name"), serde_json::json!("table_name")),
        ]
        .into_iter()
        .collect();
        let params: HashMap<String, serde_json::Value> = vec![(
            String::from("capacity_mode"),
            serde_json::json!("ON_DEMAND"),
        )]
        .into_iter()
        .collect();

        let scaling_definition = ScalingComponentDefinition {
            kind: data_layer::types::object_kind::ObjectKind::ScalingComponent,
            db_id: String::from("db_id"),
            id: String::from("scaling-id"),
            component_kind: String::from("amazon-dynamodb"),
            metadata,
            ..Default::default()
        };
        let dynamodb_table_scaling_component: Result<
            HashMap<String, serde_json::Value>,
            anyhow::Error,
        > = DynamoDbTableScalingComponent::new(scaling_definition)
            .apply(params, get_rquickjs_context().await)
            .await;
        assert!(dynamodb_table_scaling_component.is_ok());
    }
    // capacity_mode: PROVISIONED, autoscaling_mode: ON, capacity_unit: READ -> ok
    #[tokio::test]
    #[ignore]
    async fn apply_provisioned_on_read() {
        let metadata: HashMap<String, serde_json::Value> = vec![
            (String::from("region"), serde_json::json!("region")),
            (String::from("access_key"), serde_json::json!("access_key")),
            (String::from("secret_key"), serde_json::json!("secret_key")),
            (String::from("table_name"), serde_json::json!("table_name")),
        ]
        .into_iter()
        .collect();
        let params: HashMap<String, serde_json::Value> = vec![
            (
                String::from("capacity_mode"),
                serde_json::json!("PROVISIONED"),
            ),
            (String::from("autoscaling_mode"), serde_json::json!("ON")),
            (String::from("capacity_unit"), serde_json::json!("READ")),
            (String::from("read_target_value"), serde_json::json!(70)),
            (String::from("read_min_capacity"), serde_json::json!(3)),
            (String::from("read_max_capacity"), serde_json::json!(10)),
        ]
        .into_iter()
        .collect();

        let scaling_definition = ScalingComponentDefinition {
            kind: data_layer::types::object_kind::ObjectKind::ScalingComponent,
            db_id: String::from("db_id"),
            id: String::from("scaling-id"),
            component_kind: String::from("amazon-dynamodb"),
            metadata,
            ..Default::default()
        };
        let dynamodb_table_scaling_component: Result<
            HashMap<String, serde_json::Value>,
            anyhow::Error,
        > = DynamoDbTableScalingComponent::new(scaling_definition)
            .apply(params, get_rquickjs_context().await)
            .await;
        assert!(dynamodb_table_scaling_component.is_ok());
    }
    // capacity_mode: PROVISIONED, autoscaling_mode: ON, capacity_unit: WRITE -> ok
    #[tokio::test]
    #[ignore]
    async fn apply_provisioned_on_write() {
        let metadata: HashMap<String, serde_json::Value> = vec![
            (String::from("region"), serde_json::json!("region")),
            (String::from("access_key"), serde_json::json!("access_key")),
            (String::from("secret_key"), serde_json::json!("secret_key")),
            (String::from("table_name"), serde_json::json!("table_name")),
        ]
        .into_iter()
        .collect();
        let params: HashMap<String, serde_json::Value> = vec![
            (
                String::from("capacity_mode"),
                serde_json::json!("PROVISIONED"),
            ),
            (String::from("autoscaling_mode"), serde_json::json!("ON")),
            (String::from("capacity_unit"), serde_json::json!("WRITE")),
            (String::from("write_target_value"), serde_json::json!(80)),
            (String::from("write_min_capacity"), serde_json::json!(2)),
            (String::from("write_max_capacity"), serde_json::json!(10)),
        ]
        .into_iter()
        .collect();

        let scaling_definition = ScalingComponentDefinition {
            kind: data_layer::types::object_kind::ObjectKind::ScalingComponent,
            db_id: String::from("db_id"),
            id: String::from("scaling-id"),
            component_kind: String::from("amazon-dynamodb"),
            metadata,
            ..Default::default()
        };
        let dynamodb_table_scaling_component: Result<
            HashMap<String, serde_json::Value>,
            anyhow::Error,
        > = DynamoDbTableScalingComponent::new(scaling_definition)
            .apply(params, get_rquickjs_context().await)
            .await;
        assert!(dynamodb_table_scaling_component.is_ok());
    }
    // capacity_mode: PROVISIONED, autoscaling_mode: ON, capacity_unit: READ_WRITE -> ok
    #[tokio::test]
    #[ignore]
    async fn apply_provisioned_on_read_write() {
        let metadata: HashMap<String, serde_json::Value> = vec![
            (String::from("region"), serde_json::json!("region")),
            (String::from("access_key"), serde_json::json!("access_key")),
            (String::from("secret_key"), serde_json::json!("secret_key")),
            (String::from("table_name"), serde_json::json!("table_name")),
        ]
        .into_iter()
        .collect();
        let params: HashMap<String, serde_json::Value> = vec![
            (
                String::from("capacity_mode"),
                serde_json::json!("PROVISIONED"),
            ),
            (String::from("autoscaling_mode"), serde_json::json!("ON")),
            (
                String::from("capacity_unit"),
                serde_json::json!("READ_WRITE"),
            ),
            (String::from("read_capacity_units"), serde_json::json!(5)),
            (String::from("read_target_value"), serde_json::json!(70)),
            (String::from("read_min_capacity"), serde_json::json!(3)),
            (String::from("read_max_capacity"), serde_json::json!(10)),
            (String::from("write_capacity_units"), serde_json::json!(3)),
            (String::from("write_target_value"), serde_json::json!(80)),
            (String::from("write_min_capacity"), serde_json::json!(2)),
            (String::from("write_max_capacity"), serde_json::json!(10)),
        ]
        .into_iter()
        .collect();

        let scaling_definition = ScalingComponentDefinition {
            kind: data_layer::types::object_kind::ObjectKind::ScalingComponent,
            db_id: String::from("db_id"),
            id: String::from("scaling-id"),
            component_kind: String::from("amazon-dynamodb"),
            metadata,
            ..Default::default()
        };
        let dynamodb_table_scaling_component: Result<
            HashMap<String, serde_json::Value>,
            anyhow::Error,
        > = DynamoDbTableScalingComponent::new(scaling_definition)
            .apply(params, get_rquickjs_context().await)
            .await;
        assert!(dynamodb_table_scaling_component.is_ok());
    }
    // capacity_mode: PROVISIONED, autoscaling_mode: OFF, capacity_unit: READ -> ok
    #[tokio::test]
    #[ignore]
    async fn apply_provisioned_off_read() {
        let metadata: HashMap<String, serde_json::Value> = vec![
            (String::from("region"), serde_json::json!("region")),
            (String::from("access_key"), serde_json::json!("access_key")),
            (String::from("secret_key"), serde_json::json!("secret_key")),
            (String::from("table_name"), serde_json::json!("table_name")),
        ]
        .into_iter()
        .collect();
        let params: HashMap<String, serde_json::Value> = vec![
            (
                String::from("capacity_mode"),
                serde_json::json!("PROVISIONED"),
            ),
            (String::from("autoscaling_mode"), serde_json::json!("OFF")),
            (String::from("capacity_unit"), serde_json::json!("READ")),
            (String::from("read_capacity_units"), serde_json::json!(7)),
        ]
        .into_iter()
        .collect();

        let scaling_definition = ScalingComponentDefinition {
            kind: data_layer::types::object_kind::ObjectKind::ScalingComponent,
            db_id: String::from("db_id"),
            id: String::from("scaling-id"),
            component_kind: String::from("amazon-dynamodb"),
            metadata,
            ..Default::default()
        };
        let dynamodb_table_scaling_component: Result<
            HashMap<String, serde_json::Value>,
            anyhow::Error,
        > = DynamoDbTableScalingComponent::new(scaling_definition)
            .apply(params, get_rquickjs_context().await)
            .await;
        assert!(dynamodb_table_scaling_component.is_ok());
    }
    // capacity_mode: PROVISIONED, autoscaling_mode: OFF, capacity_unit: WRITE -> ok
    #[tokio::test]
    #[ignore]
    async fn apply_provisioned_off_write() {
        let metadata: HashMap<String, serde_json::Value> = vec![
            (String::from("region"), serde_json::json!("region")),
            (String::from("access_key"), serde_json::json!("access_key")),
            (String::from("secret_key"), serde_json::json!("secret_key")),
            (String::from("table_name"), serde_json::json!("table_name")),
        ]
        .into_iter()
        .collect();
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

        let scaling_definition = ScalingComponentDefinition {
            kind: data_layer::types::object_kind::ObjectKind::ScalingComponent,
            db_id: String::from("db_id"),
            id: String::from("scaling-id"),
            component_kind: String::from("amazon-dynamodb"),
            metadata,
            ..Default::default()
        };
        let dynamodb_table_scaling_component: Result<
            HashMap<String, serde_json::Value>,
            anyhow::Error,
        > = DynamoDbTableScalingComponent::new(scaling_definition)
            .apply(params, get_rquickjs_context().await)
            .await;
        assert!(dynamodb_table_scaling_component.is_ok());
    }
    // capacity_mode: PROVISIONED, autoscaling_mode: OFF, capacity_unit: READ_WRITE -> ok
    #[tokio::test]
    #[ignore]
    async fn apply_provisioned_off_read_write() {
        let metadata: HashMap<String, serde_json::Value> = vec![
            (String::from("region"), serde_json::json!("region")),
            (String::from("access_key"), serde_json::json!("access_key")),
            (String::from("secret_key"), serde_json::json!("secret_key")),
            (String::from("table_name"), serde_json::json!("table_name")),
        ]
        .into_iter()
        .collect();
        let params: HashMap<String, serde_json::Value> = vec![
            (
                String::from("capacity_mode"),
                serde_json::json!("PROVISIONED"),
            ),
            (String::from("autoscaling_mode"), serde_json::json!("OFF")),
            (
                String::from("capacity_unit"),
                serde_json::json!("READ_WRITE"),
            ),
            (String::from("read_capacity_units"), serde_json::json!(10)),
            (String::from("write_capacity_units"), serde_json::json!(10)),
        ]
        .into_iter()
        .collect();

        let scaling_definition = ScalingComponentDefinition {
            kind: data_layer::types::object_kind::ObjectKind::ScalingComponent,
            db_id: String::from("db_id"),
            id: String::from("scaling-id"),
            component_kind: String::from("amazon-dynamodb"),
            metadata,
            ..Default::default()
        };
        let dynamodb_table_scaling_component: Result<
            HashMap<String, serde_json::Value>,
            anyhow::Error,
        > = DynamoDbTableScalingComponent::new(scaling_definition)
            .apply(params, get_rquickjs_context().await)
            .await;
        assert!(dynamodb_table_scaling_component.is_ok());
    }
    // capacity_mode: PROVISIONED, autoscaling_mode: ON, capacity_unit: WRITE, without write_capacity_units -> error
    #[tokio::test]
    #[ignore]
    async fn apply_provisioned_on_write_without_write_capacity_units() {
        let metadata: HashMap<String, serde_json::Value> = vec![
            (String::from("region"), serde_json::json!("region")),
            (String::from("access_key"), serde_json::json!("access_key")),
            (String::from("secret_key"), serde_json::json!("secret_key")),
            (String::from("table_name"), serde_json::json!("table_name")),
        ]
        .into_iter()
        .collect();
        let params: HashMap<String, serde_json::Value> = vec![
            (
                String::from("capacity_mode"),
                serde_json::json!("PROVISIONED"),
            ),
            (String::from("autoscaling_mode"), serde_json::json!("ON")),
            (String::from("capacity_unit"), serde_json::json!("WRITE")),
        ]
        .into_iter()
        .collect();

        let scaling_definition = ScalingComponentDefinition {
            kind: data_layer::types::object_kind::ObjectKind::ScalingComponent,
            db_id: String::from("db_id"),
            id: String::from("scaling-id"),
            component_kind: String::from("amazon-dynamodb"),
            metadata,
            ..Default::default()
        };
        let dynamodb_table_scaling_component: Result<
            HashMap<String, serde_json::Value>,
            anyhow::Error,
        > = DynamoDbTableScalingComponent::new(scaling_definition)
            .apply(params, get_rquickjs_context().await)
            .await;
        assert!(dynamodb_table_scaling_component.is_err());
    }
    // capacity_mode: PROVISIONED, autoscaling_mode: OFF, capacity_unit: WRITE, without write_capacity_units -> error
    #[tokio::test]
    #[ignore]
    async fn apply_provisioned_off_write_without_write_capacity_units() {
        let metadata: HashMap<String, serde_json::Value> = vec![
            (String::from("region"), serde_json::json!("region")),
            (String::from("access_key"), serde_json::json!("access_key")),
            (String::from("secret_key"), serde_json::json!("secret_key")),
            (String::from("table_name"), serde_json::json!("table_name")),
        ]
        .into_iter()
        .collect();
        let params: HashMap<String, serde_json::Value> = vec![
            (
                String::from("capacity_mode"),
                serde_json::json!("PROVISIONED"),
            ),
            (String::from("autoscaling_mode"), serde_json::json!("OFF")),
            (String::from("capacity_unit"), serde_json::json!("WRITE")),
        ]
        .into_iter()
        .collect();

        let scaling_definition = ScalingComponentDefinition {
            kind: data_layer::types::object_kind::ObjectKind::ScalingComponent,
            db_id: String::from("db_id"),
            id: String::from("scaling-id"),
            component_kind: String::from("amazon-dynamodb"),
            metadata,
            ..Default::default()
        };
        let dynamodb_table_scaling_component: Result<
            HashMap<String, serde_json::Value>,
            anyhow::Error,
        > = DynamoDbTableScalingComponent::new(scaling_definition)
            .apply(params, get_rquickjs_context().await)
            .await;
        assert!(dynamodb_table_scaling_component.is_err());
    }
}
