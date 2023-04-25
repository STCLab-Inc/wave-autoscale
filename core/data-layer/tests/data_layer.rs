/**
 * Now, each test function is responsible for a single assertion,
 * making it easier to identify and fix issues.
 * You can add more test functions in a similar manner to test other aspects of the ParserResult.
 */

#[cfg(test)]
mod data_layer {
    use anyhow::Result;
    use data_layer::{
        data_layer::{DataLayer, DataLayerNewParam},
        reader::yaml_reader::{read_yaml_file, ParserResult},
    };
    use serde_json::json;

    const EXAMPLE_FILE_PATH: &str = "./tests/yaml/example.yaml";
    const EXPECTED_METRICS_COUNT: usize = 1;

    fn read_example_yaml_file() -> Result<ParserResult> {
        let yaml_file_path = EXAMPLE_FILE_PATH;
        read_yaml_file(yaml_file_path)
    }

    #[tokio::test]
    async fn test_data_layer_sqlite() -> Result<()> {
        const TEST_DB: &str = "sqlite://./tests/data-layer/test.db";
        // Delete the test db if it exists
        let path = std::path::Path::new(TEST_DB.trim_start_matches("sqlite://"));
        let _ = std::fs::remove_file(path);

        let data_layer = DataLayer::new(DataLayerNewParam {
            sql_url: TEST_DB.to_string(),
        })
        .await;
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
            assert!(false, "Unexpected error: {:?}", add_metrics_result);
        }

        // Check that the metrics were added
        let metrics_result = data_layer.get_all_metrics().await;
        if metrics_result.is_err() {
            assert!(false, "Unexpected error: {:?}", metrics_result);
        }
        let metrics_result = metrics_result.unwrap();
        assert_eq!(
            metrics_result.len(),
            result.metric_definitions.clone().len(),
            "Unexpected metrics count"
        );

        // Update a metric
        let mut metric = metrics_result[0].clone();

        // Change the name in the metadata of HashMap
        metric
            .metadata
            .insert("name".to_string(), json!("new_name"));
        let update_metric_result = data_layer.update_metric(metric.clone()).await;
        if update_metric_result.is_err() {
            assert!(false, "Unexpected error: {:?}", update_metric_result);
        }

        // Add the scaling components
        let add_scaling_components_result = data_layer
            .add_scaling_components(result.scaling_component_definitions.clone())
            .await;
        if add_scaling_components_result.is_err() {
            assert!(
                false,
                "Unexpected error: {:?}",
                add_scaling_components_result
            );
        }

        // Check that the scaling components were added
        let scaling_components_result = data_layer.get_all_scaling_components().await;
        if scaling_components_result.is_err() {
            assert!(false, "Unexpected error: {:?}", scaling_components_result);
        }
        let scaling_components_result = scaling_components_result.unwrap();
        assert_eq!(
            scaling_components_result.len(),
            result.scaling_component_definitions.clone().len(),
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
            assert!(
                false,
                "Unexpected error: {:?}",
                update_scaling_component_result
            );
        }

        // Add the scaling plans
        let add_scaling_plans_result = data_layer
            .add_plans(result.scaling_plan_definitions.clone())
            .await;
        if add_scaling_plans_result.is_err() {
            assert!(false, "Unexpected error: {:?}", add_scaling_plans_result);
        }

        // Check that the scaling plans were added
        let scaling_plans_result = data_layer.get_all_plans().await;
        if scaling_plans_result.is_err() {
            assert!(false, "Unexpected error: {:?}", scaling_plans_result);
        }
        let scaling_plans_result = scaling_plans_result.unwrap();
        assert_eq!(
            scaling_plans_result.len(),
            result.scaling_plan_definitions.clone().len(),
            "Unexpected scaling plans count"
        );

        // Update a scaling plan
        // let mut scaling_plan = scaling_plans_result[0].clone();
        // scaling_plan
        //     .plans
        //     .insert("name".to_string(), json!("new_name"));
        // let update_scaling_plan_result = data_layer.update_plan(scaling_plan.clone()).await;
        // if update_scaling_plan_result.is_err() {
        //     assert!(false, "Unexpected error: {:?}", update_scaling_plan_result);
        // }

        Ok(())
    }
}
