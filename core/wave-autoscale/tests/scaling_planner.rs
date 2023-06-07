#[cfg(test)]
mod scaling_planner_test {
    use std::{sync::Arc, time::Duration};

    use anyhow::Result;
    use data_layer::{
        data_layer::{DataLayer, DataLayerNewParam},
        reader::yaml_reader::read_yaml_file,
    };

    use tokio::time::sleep;
    use wave_autoscale::{
        metric_adapter::MetricAdapterManager,
        metric_store::{new_metric_store, MetricStore},
        scaling_component::new_scaling_component_manager,
        scaling_planner::ScalingPlanner,
    };

    const PLAN_PROMETHEUS_EC2: &str = "./tests/yaml/plan_prometheus_ec2.yaml";

    // multithreaded test
    #[tokio::test]
    async fn planner_prometheus_ec2() -> Result<()> {
        // read yaml file
        let result = read_yaml_file(PLAN_PROMETHEUS_EC2)?;

        // create metric adapter manager
        let metric_store: MetricStore = new_metric_store();
        let mut metric_adapter_manager = MetricAdapterManager::new(metric_store.clone());
        metric_adapter_manager.add_definitions(result.metric_definitions);
        metric_adapter_manager.run();

        // Give some time for the metric adapters to collect metrics
        sleep(Duration::from_millis(2000)).await;

        let scaling_component_manager = new_scaling_component_manager();
        // use {} to avoid the error: cannot move out of `result.scaling_component_definitions` which is behind a shared reference
        {
            let cloned = scaling_component_manager.clone();
            let mut cloned_scaling_component_manager = cloned.write().await;
            cloned_scaling_component_manager.add_definitions(result.scaling_component_definitions);
        }

        // Delete the test db if it exists
        let TEST_DB = "sqlite://tests/temp/test.db";
        let path = std::path::Path::new(TEST_DB.trim_start_matches("sqlite://"));
        let _ = std::fs::remove_file(path);
        // create data layer
        let data_layer = DataLayer::new(DataLayerNewParam {
            sql_url: TEST_DB.to_string(),
        })
        .await;
        let data_layer = Arc::new(data_layer);

        // create scaling planner
        let scaling_planners: Vec<ScalingPlanner> = result
            .scaling_plan_definitions
            .iter()
            .map(|definition| {
                ScalingPlanner::new(
                    definition.clone(),
                    metric_store.clone(),
                    scaling_component_manager.clone(),
                    Arc::clone(&data_layer),
                )
            })
            .collect();

        // run scaling planner
        if let Some(scaling_planner) = scaling_planners.get(0) {
            scaling_planner.run().await;
        }
        // Give some time for the scaling planner to run plans
        sleep(Duration::from_millis(5000)).await;

        // check data layer
        let history = data_layer
            .get_autoscaling_history_by_plan_id(result.scaling_plan_definitions[0].id.clone())
            .await;
        assert!(history.is_ok());
        let history = history.unwrap();
        assert_eq!(history.len(), 1);
        let history = history[0].clone();
        assert_eq!(history.plan_id, result.scaling_plan_definitions[0].id);

        Ok(())
    }
}
