#[cfg(test)]
mod app_test {
    use anyhow::Result;
    use std::time::Duration;
    use tokio::time::sleep;
    use uuid::Uuid;
    use wave_autoscale::{app, args::Args};

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn remove_db(path: String) {
        let path = std::path::Path::new(path.trim_start_matches("sqlite://"));
        let remove_result = std::fs::remove_file(path);
        if remove_result.is_err() {
            println!("Error removing file: {:?}", remove_result);
        }
    }

    // multithreaded test
    #[tokio::test]
    async fn test_run() -> Result<()> {
        init();
        remove_db("wave.db".to_string());
        let plan_path = "tests/yaml/plan_prometheus_ec2.yaml".to_string();
        let mut app = app::App::new(Args {
            plan: Some(plan_path),
            config: None,
            watch_duration: 1,
            autoscaling_history_retention: None,
        })
        .await;
        app.run().await;
        sleep(Duration::from_secs(2)).await;

        // Try to change metric definitions in DataLayer and see if the changes are reflected in the app
        let data_layer = app.get_data_layer();
        let metric_definitions = data_layer.get_all_metrics().await?;
        let mut new_metric = metric_definitions[0].clone();
        let new_id = Uuid::new_v4().to_string();
        new_metric.db_id = "".to_string();
        new_metric.id = new_id.clone();

        println!("Adding new metric definitions: {:?}", new_metric);
        let new_metric_definitions = vec![new_metric.clone()];
        data_layer.add_metrics(new_metric_definitions).await?;
        app.run().await;

        sleep(Duration::from_secs(2)).await;

        // Check if the new metric is added to the app
        let metric_adapter_manager = app.get_metric_adapter_manager();
        let metric_adapters = metric_adapter_manager.get_metric_adapters();
        // Find new_id in metric_adapters
        let mut found = false;
        for metric_adapter_id in metric_adapters.keys() {
            if metric_adapter_id == &new_id {
                found = true;
                break;
            }
        }
        assert!(found);
        // Try to change scaling component definitions in DataLayer and see if the changes are reflected in the app
        let scaling_component_definitions = data_layer.get_all_scaling_components().await?;
        let mut new_scaling_component = scaling_component_definitions[0].clone();
        let new_id = Uuid::new_v4().to_string();
        new_scaling_component.db_id = "".to_string();
        new_scaling_component.id = new_id.clone();

        println!(
            "Adding new scaling component definitions: {:?}",
            new_scaling_component
        );
        let new_scaling_component_definitions = vec![new_scaling_component.clone()];
        data_layer
            .add_scaling_components(new_scaling_component_definitions)
            .await?;
        app.run().await;

        sleep(Duration::from_secs(2)).await;

        // Check if the new scaling component is added to the app
        {
            let scaling_component_manager = app.get_scaling_component_manager();
            let scaling_component_manager = scaling_component_manager.read().await;
            let scaling_components = scaling_component_manager.get_scaling_components();
            // Find new_id in scaling_components
            let mut found = false;
            for scaling_component_id in scaling_components.keys() {
                if scaling_component_id == &new_id {
                    found = true;
                    break;
                }
            }
            assert!(found);
        }

        // Try to change scaling plan in DataLayer and see if the changes are reflected in the app
        let scaling_plan = data_layer.get_all_plans().await.unwrap();
        let mut new_scaling_plan = scaling_plan[0].clone();
        let new_id = Uuid::new_v4().to_string();
        new_scaling_plan.db_id = "".to_string();
        new_scaling_plan.id = new_id.clone();

        println!("Adding new scaling plan: {:?}", new_scaling_plan);
        data_layer.add_plans(vec![new_scaling_plan.clone()]).await?;
        app.run().await;

        sleep(Duration::from_secs(2)).await;

        // Check if the new scaling plan is added to the app
        {
            let scaling_planner_manager = app.get_scaling_planner_manager();
            let scaling_planner_manager = scaling_planner_manager.read().await;
            let scaling_planners = scaling_planner_manager.get_scaling_planners();
            // Find new_id in scaling_plans
            let mut found = false;
            for scaling_plan_id in scaling_planners.keys() {
                if scaling_plan_id == &new_id {
                    found = true;
                    break;
                }
            }
            assert!(found);
        }

        // sleep(Duration::from_secs(10)).await;
        Ok(())
    }
}
