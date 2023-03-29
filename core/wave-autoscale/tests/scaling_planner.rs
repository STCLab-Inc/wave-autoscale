#[cfg(test)]
mod scaling_planner_test {
    use anyhow::Result;
    use data_layer::reader::yaml_reader::read_yaml_file;
    use serde_json::{json, Value};
    use std::collections::HashMap;
    use wave_autoscale::scaling_component::{
        aws_ec2_autoscaling::EC2AutoScalingComponent, create_scaling_component, ScalingComponent,
    };

    const PLAN_PROMETHEUS_EC2: &str = "./tests/yaml/plan_prometheus_ec2.yaml";

    // multithreaded test
    #[tokio::test]
    async fn prometheus_ec2() -> Result<()> {
        // read yaml file
        let result = read_yaml_file(PLAN_PROMETHEUS_EC2)?;

        // create metric adapter
        let scaling_triggers: Vec<Box<dyn ScalingComponent>> = result
            .scaling_component_definitions
            .iter()
            .map(|definition| create_scaling_component(definition).unwrap())
            .collect();

        // run metric adapter
        if let Some(scaling_trigger) = scaling_triggers.get(0) {
            let name = scaling_trigger.get_scaling_component_kind();
            assert!(name == EC2AutoScalingComponent::TRIGGER_KIND, "Unexpected");

            // run scaling trigger
            let mut options: HashMap<String, Value> = HashMap::new();
            options.insert(
                "name".to_string(),
                json!("tf-wa-20230322020900305100000006"),
            );
            options.insert("min".to_string(), json!(1));
            options.insert("max".to_string(), json!(4));
            options.insert("desired".to_string(), json!(1));
            let result = scaling_trigger.apply(options).await;
            return result;
        } else {
            assert!(false, "No metric adapter found")
        }
        Ok(())
    }
}
