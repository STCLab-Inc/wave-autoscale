/**
 * Now, each test function is responsible for a single assertion,
 * making it easier to identify and fix issues.
 * You can add more test functions in a similar manner to test other aspects of the ParserResult.
 */

mod reader {
    use anyhow::Result;
    use data_layer::reader::wave_definition_reader::{read_definition_yaml_file, ParserResult};

    const EXAMPLE_FILE_PATH: &str = "./tests/yaml/example.yaml";
    const EXPECTED_METRICS_COUNT: usize = 2;
    const EXPECTED_SLOS_COUNT: usize = 1;

    fn read_example_yaml_file() -> Result<ParserResult> {
        let yaml_file_path = EXAMPLE_FILE_PATH;
        read_definition_yaml_file(yaml_file_path)
    }

    #[test]
    fn test_metric_definitions_count() -> Result<()> {
        let result = read_example_yaml_file()?;
        assert_eq!(
            result.metric_definitions.len(),
            EXPECTED_METRICS_COUNT,
            "Unexpected metrics count"
        );
        Ok(())
    }

    #[test]
    fn test_metric_definitions_metadata() -> Result<()> {
        let result = read_example_yaml_file()?;
        if let Some(first_metric) = result.metric_definitions.get(0) {
            if let Some(user_key) = first_metric.metadata.get("user_key") {
                assert_eq!(user_key, "user_value", "Unexpected metadata value");
            }
        }
        Ok(())
    }

    #[test]
    fn test_slo_definitions_count() -> Result<()> {
        let result = read_example_yaml_file()?;
        assert_eq!(
            result.slo_definitions.len(),
            EXPECTED_SLOS_COUNT,
            "Unexpected SLOs count"
        );
        Ok(())
    }

    #[test]
    fn test_scaling_plan_definitions_count() -> Result<()> {
        let result = read_example_yaml_file()?;
        assert!(
            !result.scaling_plan_definitions.is_empty(),
            "Unexpected Scaling Plans count. Expected 1, but got 0"
        );
        Ok(())
    }

    #[test]
    fn test_scaling_component_definitions_count() -> Result<()> {
        let result = read_example_yaml_file()?;
        assert!(
            !result.scaling_component_definitions.is_empty(),
            "Unexpected Scaling Plans count. Expected 1, but got 0"
        );
        Ok(())
    }

    const EXAMPLE_ORIGINIAL_FILE_PATH: &str = "./tests/variables-examples/example.yaml";
    fn read_example_original_yaml_file() -> Result<ParserResult> {
        let yaml_file_path = EXAMPLE_ORIGINIAL_FILE_PATH;
        read_definition_yaml_file(yaml_file_path)
    }

    #[test]
    fn test_parsed_file_yaml_file() -> Result<()> {
        let result = read_example_original_yaml_file()?;
        if let Some(scaling_component_definition) = result.scaling_component_definitions.get(0) {
            if let Some(region) = scaling_component_definition.metadata.get("region") {
                assert_eq!(
                    region, "ap-northeast-3",
                    "Unexpected metadata value in region from yaml file"
                );
            }
            if let Some(access_key) = scaling_component_definition.metadata.get("access_key") {
                assert_eq!(
                    access_key, "access_key",
                    "Unexpected metadata value in access_key from environment file"
                );
            }
            if let Some(secret_key) = scaling_component_definition.metadata.get("secret_key") {
                assert_eq!(
                    secret_key, "secret_key",
                    "Unexpected metadata value in secret_key from json file"
                );
            }
        }
        Ok(())
    }
}
