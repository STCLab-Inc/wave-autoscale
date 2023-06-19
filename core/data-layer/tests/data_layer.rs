/**
 * Now, each test function is responsible for a single assertion,
 * making it easier to identify and fix issues.
 * You can add more test functions in a similar manner to test other aspects of the ParserResult.
 */

#[cfg(test)]
mod data_layer {
    use std::{
        collections::HashMap,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
    };

    use anyhow::Result;
    use chrono::Utc;
    use data_layer::{
        data_layer::{DataLayer, DataLayerNewParam},
        reader::yaml_reader::{read_yaml_file, ParserResult},
        types::{
            autoscaling_history_definition::AutoscalingHistoryDefinition, object_kind::ObjectKind,
        },
        MetricDefinition, ScalingComponentDefinition,
    };
    use rand::Rng;
    use serde_json::json;
    use tokio::sync::Mutex;

    const EXAMPLE_FILE_PATH: &str = "./tests/yaml/example.yaml";
    const EXPECTED_METRICS_COUNT: usize = 1;
    const EXPECTED_SCALING_COMPONENTS_COUNT: usize = 1;
    const EXPECTED_SCALING_PLANS_COUNT: usize = 1;

    // Common function to read the example yaml file
    fn read_example_yaml_file() -> Result<ParserResult> {
        let yaml_file_path = EXAMPLE_FILE_PATH;
        read_yaml_file(yaml_file_path)
    }

    async fn get_data_layer() -> Result<DataLayer> {
        const TEST_DB: &str = "sqlite://./tests/temp/test.db";
        // Delete the test db if it exists
        let path = std::path::Path::new(TEST_DB.trim_start_matches("sqlite://"));
        let remove_result = std::fs::remove_file(path);
        if remove_result.is_err() {
            println!("Error removing file: {:?}", remove_result);
        }
        let data_layer = DataLayer::new(DataLayerNewParam {
            sql_url: TEST_DB.to_string(),
        })
        .await;
        Ok(data_layer)
    }

    #[tokio::test]
    async fn test_run_watch() -> Result<()> {
        let data_layer = get_data_layer().await?;

        let mut watch_receiver = data_layer.watch();
        let verification = Arc::new(AtomicBool::new(false));
        let verification_clone = verification.clone();

        tokio::spawn(async move {
            println!("Waiting for watch result");
            if watch_receiver.changed().await.is_ok() {
                println!("Received watch result");
                let result = watch_receiver.borrow();
                println!("Received watch result: {:?}", result);
                verification_clone.store(true, Ordering::Release);
            }
        });
        // sleep
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        data_layer
            .add_metrics(vec![MetricDefinition {
                id: "test".to_string(),
                db_id: "test".to_string(),
                kind: ObjectKind::Metric,
                metric_kind: "test".to_string(),
                metadata: HashMap::new(),
            }])
            .await?;
        println!("changed values - 1");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        data_layer
            .add_scaling_components(vec![ScalingComponentDefinition {
                id: "test".to_string(),
                db_id: "test".to_string(),
                component_kind: "test".to_string(),
                kind: ObjectKind::ScalingComponent,
                metadata: HashMap::new(),
            }])
            .await?;
        println!("changed values - 2");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        data_layer
            .add_scaling_components(vec![ScalingComponentDefinition {
                id: "test2".to_string(),
                db_id: "test2".to_string(),
                component_kind: "test".to_string(),
                kind: ObjectKind::ScalingComponent,
                metadata: HashMap::new(),
            }])
            .await?;
        println!("changed values - 3");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        assert!(verification.load(Ordering::Acquire));
        Ok(())
    }

    #[tokio::test]
    async fn test_metrics() -> Result<()> {
        let data_layer = get_data_layer().await?;
        let result = read_example_yaml_file()?;
        assert_eq!(
            result.metric_definitions.len(),
            EXPECTED_METRICS_COUNT,
            "Unexpected metrics count"
        );

        // Add the metrics
        let add_metrics_result = data_layer
            .add_metrics(result.metric_definitions.clone())
            .await;
        if add_metrics_result.is_err() {
            panic!("Unexpected error: {:?}", add_metrics_result);
        }

        // Check that the metrics were added
        let metrics_result = data_layer.get_all_metrics().await;
        if metrics_result.is_err() {
            panic!("Unexpected error: {:?}", metrics_result);
        }
        let metrics_result = metrics_result.unwrap();
        assert_eq!(
            metrics_result.len(),
            result.metric_definitions.clone().len(),
            "Unexpected metrics count"
        );
        assert_eq!(
            metrics_result[0].metadata, result.metric_definitions[0].metadata,
            "Unexpected metrics metadata"
        );

        // Add the metrics that already exist but with different metadata
        let mut changed_metric_definitions = result.metric_definitions.clone();
        changed_metric_definitions[0]
            .metadata
            .insert("name".to_string(), json!("new_name"));
        let add_metrics_result = data_layer
            .add_metrics(changed_metric_definitions.clone())
            .await;
        if add_metrics_result.is_err() {
            panic!("Unexpected error: {:?}", add_metrics_result);
        }
        let metrics_result = data_layer.get_all_metrics().await;
        if metrics_result.is_err() {
            panic!("Unexpected error: {:?}", metrics_result);
        }
        let metrics_result = metrics_result.unwrap();
        assert_eq!(
            metrics_result[0].metadata, changed_metric_definitions[0].metadata,
            "Unexpected metrics count"
        );

        // Check that the metrics were updated
        let metrics_result = data_layer.get_all_metrics().await;
        if metrics_result.is_err() {
            panic!("Unexpected error: {:?}", metrics_result);
        }
        let metrics_result = metrics_result.unwrap();
        assert_eq!(
            changed_metric_definitions[0].metadata, metrics_result[0].metadata,
            "Unexpected metrics metadata"
        );

        // Update a metric
        let mut metric = metrics_result[0].clone();

        // Change the name in the metadata of HashMap
        metric
            .metadata
            .insert("name".to_string(), json!("new_name"));
        let update_metric_result = data_layer.update_metric(metric.clone()).await;
        if update_metric_result.is_err() {
            panic!("Unexpected error: {:?}", update_metric_result);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_scaling_components() -> Result<()> {
        let data_layer = get_data_layer().await?;
        let result = read_example_yaml_file()?;
        assert_eq!(
            result.scaling_component_definitions.len(),
            EXPECTED_SCALING_COMPONENTS_COUNT,
            "Unexpected metrics count"
        );
        // Add the scaling components
        let add_scaling_components_result = data_layer
            .add_scaling_components(result.scaling_component_definitions.clone())
            .await;
        if add_scaling_components_result.is_err() {
            panic!("Unexpected error: {:?}", add_scaling_components_result);
        }

        // Check that the scaling components were added
        let scaling_components_result = data_layer.get_all_scaling_components().await;
        if scaling_components_result.is_err() {
            panic!("Unexpected error: {:?}", scaling_components_result);
        }
        let scaling_components_result = scaling_components_result.unwrap();
        assert_eq!(
            scaling_components_result.len(),
            result.scaling_component_definitions.clone().len(),
            "Unexpected scaling components count"
        );
        assert_eq!(
            scaling_components_result[0].metadata, result.scaling_component_definitions[0].metadata,
            "Unexpected scaling components metadata"
        );

        // Add the scaling components that already exist but with different metadata
        let mut changed_scaling_component_definitions =
            result.scaling_component_definitions.clone();
        changed_scaling_component_definitions[0]
            .metadata
            .insert("name".to_string(), json!("new_name"));
        let add_scaling_components_result = data_layer
            .add_scaling_components(changed_scaling_component_definitions.clone())
            .await;
        if add_scaling_components_result.is_err() {
            panic!("Unexpected error: {:?}", add_scaling_components_result);
        }
        let scaling_components_result = data_layer.get_all_scaling_components().await;
        if scaling_components_result.is_err() {
            panic!("Unexpected error: {:?}", scaling_components_result);
        }
        let scaling_components_result = scaling_components_result.unwrap();
        assert_eq!(
            scaling_components_result[0].metadata,
            changed_scaling_component_definitions[0].metadata,
            "Unexpected scaling components count"
        );

        // Update a scaling component
        let mut scaling_component = scaling_components_result[0].clone();
        scaling_component
            .metadata
            .insert("name".to_string(), json!("new_name"));
        let update_scaling_component_result = data_layer
            .update_scaling_component(scaling_component.clone())
            .await;
        if update_scaling_component_result.is_err() {
            panic!("Unexpected error: {:?}", update_scaling_component_result);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_scaling_plans() -> Result<()> {
        let data_layer = get_data_layer().await?;
        let result = read_example_yaml_file()?;
        assert_eq!(
            result.scaling_plan_definitions.len(),
            EXPECTED_SCALING_PLANS_COUNT,
            "Unexpected metrics count"
        );

        // Add the scaling plans
        let add_scaling_plans_result = data_layer
            .add_plans(result.scaling_plan_definitions.clone())
            .await;
        if add_scaling_plans_result.is_err() {
            panic!("Unexpected error: {:?}", add_scaling_plans_result);
        }

        // Check that the scaling plans were added
        let scaling_plans_result = data_layer.get_all_plans().await;
        if scaling_plans_result.is_err() {
            panic!("Unexpected error: {:?}", scaling_plans_result);
        }
        let scaling_plans_result = scaling_plans_result.unwrap();
        assert_eq!(
            scaling_plans_result.len(),
            result.scaling_plan_definitions.clone().len(),
            "Unexpected scaling plans count"
        );
        let origin_plan_json =
            serde_json::to_string(&result.scaling_plan_definitions[0].plans).unwrap();
        let updated_plan_json = serde_json::to_string(&scaling_plans_result[0].plans).unwrap();
        assert_eq!(
            origin_plan_json, updated_plan_json,
            "Unexpected scaling plans metadata"
        );

        // Update a scaling plan
        let mut scaling_plan = scaling_plans_result[0].clone();
        scaling_plan.plans[0].priority = rand::thread_rng().gen_range(0..100);
        scaling_plan.plans[0].scaling_components.push(json!({
            "name": "new_name",
            "value": 1
        }));

        let update_scaling_plan_result = data_layer.update_plan(scaling_plan.clone()).await;
        if update_scaling_plan_result.is_err() {
            panic!("Unexpected error: {:?}", update_scaling_plan_result);
        }
        let scaling_plans_result = data_layer.get_all_plans().await;
        if scaling_plans_result.is_err() {
            panic!("Unexpected error: {:?}", scaling_plans_result);
        }
        let scaling_plans_result = scaling_plans_result.unwrap();
        let origin_plan_json = serde_json::to_string(&scaling_plan.plans).unwrap();
        let updated_plan_json = serde_json::to_string(&scaling_plans_result[0].plans).unwrap();
        assert_eq!(
            origin_plan_json, updated_plan_json,
            "Unexpected scaling plans metadata"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_autoscaling_history() -> Result<()> {
        let data_layer = get_data_layer().await?;

        // Add a AutoscalingHistory
        // let scaling_plan = scaling_plans_result[0].clone();
        // let plan_db_id = scaling_plan.db_id.clone();
        let autoscaling_history = AutoscalingHistoryDefinition {
            id: "".to_string(),
            plan_db_id: "".to_string(),
            plan_id: "".to_string(),
            plan_item_json: "".to_string(),
            metric_values_json: "".to_string(),
            metadata_values_json: "".to_string(),
            fail_message: Some(String::from("fail_message")),
            created_at: Utc::now(),
        };
        let add_autoscaling_history_result = data_layer
            .add_autoscaling_history(autoscaling_history.clone())
            .await;
        if add_autoscaling_history_result.is_err() {
            panic!("Unexpected error: {:?}", add_autoscaling_history_result);
        }

        Ok(())
    }
}
