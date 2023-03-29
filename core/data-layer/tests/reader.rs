#[cfg(test)]
mod reader {
    use anyhow::Result;
    use data_layer::reader::yaml_reader::read_yaml_file;

    const EXAMPLE_FILE_PATH: &str = "./tests/yaml/example.yaml";
    const EXPECTED_METRICS_COUNT: usize = 1;
    const EXPECTED_SLOS_COUNT: usize = 1;

    #[test]
    fn test_yaml_reader() -> Result<()> {
        // Given
        let yaml_file_path = EXAMPLE_FILE_PATH;

        // When
        let result = read_yaml_file(yaml_file_path)?;

        // Then
        assert!(
            result.metric_definitions.len() == EXPECTED_METRICS_COUNT,
            "Unexpected metrics count. Expected {}, but got {}",
            EXPECTED_METRICS_COUNT,
            result.metric_definitions.len()
        );
        // how to get a first element in result.metrics
        if let Some(first_metric) = result.metric_definitions.get(0) {
            if let Some(user_key) = first_metric.metadata.get("user_key") {
                assert!(user_key == "user_value", "Unexpected");
            }
        }

        assert!(
            result.slo_definitions.len() == EXPECTED_SLOS_COUNT,
            "Unexpected SLOs count. Expected {}, but got {}",
            EXPECTED_SLOS_COUNT,
            result.slo_definitions.len()
        );

        // Print the result for debugging purposes
        println!("{:?}", result);

        Ok(())
    }
}
