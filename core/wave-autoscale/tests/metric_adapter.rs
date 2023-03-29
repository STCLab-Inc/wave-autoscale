#[cfg(test)]
mod metric_adapter_test {
    use std::{collections::HashMap, time::Duration};

    use anyhow::Result;
    use data_layer::reader::yaml_reader::read_yaml_file;
    use tokio::time::sleep;
    use wave_autoscale::metric_adapter::{
        create_metric_adapter, prometheus::PrometheusMetricAdapter, MetricAdapter,
    };

    const EXAMPLE_FILE_PATH: &str = "./tests/yaml/metric_prometheus.yaml";

    // multithreaded test
    #[tokio::test]
    async fn prometheus() -> Result<()> {
        // read yaml file
        let result = read_yaml_file(EXAMPLE_FILE_PATH)?;

        // create metric adapter
        let metrics: HashMap<String, Box<dyn MetricAdapter>> = result
            .metrics
            .iter()
            .map(|metric| {
                let metric_adapter = create_metric_adapter(metric).unwrap();
                (metric.id.clone(), metric_adapter)
            })
            .collect();

        // run metric adapter
        if let Some(first_metric) = metrics.get("prometheus_api_server_cpu_average_each") {
            // check metric kind
            let prometheus_metric_adapter = first_metric.as_ref();
            let name = prometheus_metric_adapter.get_metric_kind();
            assert!(name == PrometheusMetricAdapter::METRIC_KIND, "Unexpected");

            // run metric adapter
            prometheus_metric_adapter.run().await;

            // wait for 5 seconds to get a value from prometheus
            sleep(Duration::from_millis(5000)).await;

            // get value from metric adapter if it gets a value from prometheus
            let value = prometheus_metric_adapter.get_value().await;
            assert_ne!(value, 0.0);

            // get timestamp from metric adapter if it gets a value from prometheus
            let timestamp = prometheus_metric_adapter.get_timestamp().await;
            assert_ne!(timestamp, 0.0);
        } else {
            assert!(false, "No metric adapter found")
        }
        Ok(())
    }
}
