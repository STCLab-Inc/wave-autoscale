#[cfg(test)]
mod metric_adapter_test {
    use anyhow::Result;
    use data_layer::reader::yaml_reader::read_yaml_file;
    use std::{collections::HashMap, sync::Arc, time::Duration};
    use tokio::{sync::RwLock, time::sleep};
    use wave_autoscale::metric_adapter::{
        prometheus::PrometheusMetricAdapter, MetricAdapterManager, MetricStore,
    };

    const EXAMPLE_FILE_PATH: &str = "./tests/yaml/metric_prometheus.yaml";

    // multithreaded test
    #[tokio::test]
    async fn prometheus() -> Result<()> {
        // read yaml file
        let result = read_yaml_file(EXAMPLE_FILE_PATH)?;

        // create metric adapter manager
        let metric_store: MetricStore = Arc::new(RwLock::new(HashMap::new()));
        let mut metric_adapter_manager = MetricAdapterManager::new(metric_store.clone());
        metric_adapter_manager.add_definitions(result.metric_definitions);
        metric_adapter_manager.run().await;

        // run metric adapter
        if let Some(metric_adapter) =
            metric_adapter_manager.get_metric_adapter("prometheus_api_server_cpu_average_each")
        {
            // // check metric kind
            // let prometheus_metric_adapter = metric_adapter.as_ref();
            // let name = prometheus_metric_adapter.get_metric_kind();
            // assert!(name == PrometheusMetricAdapter::METRIC_KIND, "Unexpected");

            // // run metric adapter
            // prometheus_metric_adapter.run().await;

            // wait for 5 seconds to get a value from prometheus
            sleep(Duration::from_millis(2000)).await;

            // get value from metric adapter if it gets a value from prometheus
            let value = metric_adapter.get_value().await;
            assert_ne!(value, 0.0);

            // get timestamp from metric adapter if it gets a value from prometheus
            let timestamp = metric_adapter.get_timestamp().await;
            assert_ne!(timestamp, 0.0);

            // Compare the value and timestamp in metric_adapter and timestamp in the metric store
            let cloned = metric_store.clone();
            let cloned = cloned.read().await;
            println!("metric_store: {:?}", cloned.keys());
            let metric_from_store = cloned
                .get("prometheus_api_server_cpu_average_each")
                .unwrap();
            let value_from_store = metric_from_store.as_str().unwrap().parse::<f64>().unwrap();
            assert_eq!(value, value_from_store);
        } else {
            assert!(false, "No metric adapter found")
        }

        Ok(())
    }
}
