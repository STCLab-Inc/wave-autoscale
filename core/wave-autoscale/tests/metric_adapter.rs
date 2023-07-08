#[cfg(test)]
mod metric_adapter_test {
    use anyhow::Result;
    use data_layer::reader::wave_definition_reader::read_definition_yaml_file;
    use serde_json::Value;
    use std::time::Duration;
    use tokio::time::sleep;
    use wave_autoscale::{
        metric_adapter::MetricAdapterManager,
        metric_store::{new_shared, SharedMetricStore},
    };

    const EXAMPLE_FILE_PATH: &str = "./tests/yaml/metric_prometheus.yaml";

    // multithreaded test
    #[tokio::test]
    async fn prometheus() -> Result<()> {
        // read yaml file
        let result = read_definition_yaml_file(EXAMPLE_FILE_PATH)?;

        // create metric adapter manager
        let metric_store: SharedMetricStore = new_shared();
        let mut metric_adapter_manager = MetricAdapterManager::new(metric_store.clone());
        metric_adapter_manager.add_definitions(result.metric_definitions);

        // run metric adapters and wait for them to start
        metric_adapter_manager.run();

        sleep(Duration::from_millis(2000)).await;

        // Compare the value and timestamp in metric_adapter and timestamp in the metric store
        let cloned_metric_store = metric_store.clone();
        let cloned_metric_store = cloned_metric_store.read().await;
        println!("metric_store: {:?}", cloned_metric_store);
        let metric_from_store = cloned_metric_store
            .get("prometheus_api_server_cpu_average_total")
            .unwrap();
        let value_from_store = metric_from_store.as_str().unwrap().parse::<f64>().unwrap();
        println!("value_from_store: {}", value_from_store);
        assert!(value_from_store > 0.0);

        Ok(())
    }
    #[tokio::test]
    async fn cloudwatch_statistics() -> Result<()> {
        // read yaml file
        let result = read_definition_yaml_file("./tests/yaml/metric_cloudwatch_statistics.yaml")?;

        // create metric adapter manager
        let metric_store: SharedMetricStore = new_shared();
        let mut metric_adapter_manager = MetricAdapterManager::new(metric_store.clone());
        metric_adapter_manager.add_definitions(result.metric_definitions);

        // run metric adapters and wait for them to start
        metric_adapter_manager.run();

        sleep(Duration::from_millis(2000)).await;

        // Compare the value and timestamp in metric_adapter and timestamp in the metric store
        let cloned_metric_store = metric_store.clone();
        let cloned_metric_store = cloned_metric_store.read().await;
        println!("metric_store: {:?}", cloned_metric_store);
        let metric_from_store = cloned_metric_store
            .get("cloudwatch_cpu_average")
            .and_then(Value::as_f64)
            .unwrap();
        println!("value_from_store: {}", metric_from_store);
        assert!(metric_from_store > 0.0);

        Ok(())
    }
    #[tokio::test]
    async fn cloudwatch_data() -> Result<()> {
        // read yaml file
        let result = read_definition_yaml_file("./tests/yaml/metric_cloudwatch_data.yaml")?;

        // create metric adapter manager
        let metric_store: SharedMetricStore = new_shared();
        let mut metric_adapter_manager = MetricAdapterManager::new(metric_store.clone());
        metric_adapter_manager.add_definitions(result.metric_definitions);

        // run metric adapters and wait for them to start
        metric_adapter_manager.run();

        sleep(Duration::from_millis(2000)).await;

        // Compare the value and timestamp in metric_adapter and timestamp in the metric store
        let cloned_metric_store = metric_store.clone();
        let cloned_metric_store = cloned_metric_store.read().await;
        println!("metric_store: {:?}", cloned_metric_store);
        let metric_from_store = cloned_metric_store
            .get("cloudwatch_cpu_average")
            .and_then(Value::as_f64)
            .unwrap();
        println!("value_from_store: {}", metric_from_store);
        assert!(metric_from_store > 0.0);

        Ok(())
    }
}
