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
    const EXPECTED_SLOS_COUNT: usize = 1;

    fn read_example_yaml_file() -> Result<ParserResult> {
        let yaml_file_path = EXAMPLE_FILE_PATH;
        read_yaml_file(yaml_file_path)
    }

    #[tokio::test]
    async fn test_data_layer_sqlite() -> Result<()> {
        let data_layer = DataLayer::new(DataLayerNewParam {
            sql_url: "sqlite://./tests/data-layer/test.db".to_string(),
        })
        .await;
        let result = read_example_yaml_file()?;
        assert_eq!(
            result.metric_definitions.len(),
            EXPECTED_METRICS_COUNT,
            "Unexpected metrics count"
        );
        // Delete all metrics
        data_layer.delete_all_metrics().await;

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
        let mut metric = result.metric_definitions[0].clone();
        // Change the name in the metadata of HashMap
        metric
            .metadata
            .insert("name".to_string(), json!("new_name"));
        let update_metric_result = data_layer.update_metric(metric.clone()).await;
        if update_metric_result.is_err() {
            assert!(false, "Unexpected error: {:?}", update_metric_result);
        }
        Ok(())
    }
}
