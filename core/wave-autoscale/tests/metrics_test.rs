mod metrics_test {
    use data_layer::data_layer::DataLayer;
    use log::debug;
    use serde_json::json;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    // A utility function for metrics.
    // This function automatically creates SourceMetrics in DataLayer.
    // As a result, the developer can have a test with ScalingPlan easily.
    #[tokio::test]
    #[ignore]
    async fn test_simulation() {
        init();
        let data_layer = DataLayer::new("sqlite://../../tests/db/wave.db", "").await;
        let collector = "vector";
        let metric_id = "cloudwatch_dynamodb_id";

        let mut json_template = handlebars::Handlebars::new();

        let base_json_value = r#"[{
            "name": "dynamodb_capacity_usage",
            "tags": {
                "tag1": "value1"
            },
            "value": {{value}}
        }]
        "#;

        json_template
            .register_template_string("metric", base_json_value)
            .unwrap();

        // Call
        let _ = data_layer
            .add_source_metric(
                collector,
                metric_id,
                json_template
                    .render("metric", &json!({"value": 1}))
                    .unwrap()
                    .as_str(),
            )
            .await;
        debug!("Added a metric");
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Call
        let _ = data_layer
            .add_source_metric(
                collector,
                metric_id,
                json_template
                    .render("metric", &json!({"value": 12}))
                    .unwrap()
                    .as_str(),
            )
            .await;
        debug!("Added a metric");
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Call
        let _ = data_layer
            .add_source_metric(
                collector,
                metric_id,
                json_template
                    .render("metric", &json!({"value": 10}))
                    .unwrap()
                    .as_str(),
            )
            .await;
        debug!("Added a metric");
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
