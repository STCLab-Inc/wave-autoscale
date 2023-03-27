#[cfg(test)]
mod scaling_trigger_test {
    use anyhow::Result;
    use data_layer::reader::yaml_reader::read_yaml_file;
    use std::collections::HashMap;
    use wave_autoscale::scaling_trigger::{
        aws_ec2_autoscaling::EC2AutoScalingTrigger, create_scaling_trigger, ScalingTrigger,
        ScalingTriggerValueType,
    };

    const EC2_AUTOSCALING_FILE_PATH: &str = "./tests/yaml/trigger_ec2_autoscaling.yaml";

    // multithreaded test
    #[tokio::test]
    async fn ec2_autoscaling() -> Result<()> {
        // read yaml file
        let result = read_yaml_file(EC2_AUTOSCALING_FILE_PATH)?;

        // create metric adapter
        let scaling_triggers: Vec<Box<dyn ScalingTrigger>> = result
            .scaling_triggers
            .iter()
            .map(|scaling_trigger_data| create_scaling_trigger(scaling_trigger_data).unwrap())
            .collect();

        // run metric adapter
        if let Some(scaling_trigger) = scaling_triggers.get(0) {
            let name = scaling_trigger.get_trigger_kind();
            assert!(name == EC2AutoScalingTrigger::TRIGGER_KIND, "Unexpected");

            // run scaling trigger
            let mut options: HashMap<String, ScalingTriggerValueType> = HashMap::new();
            options.insert(
                "name".to_string(),
                ScalingTriggerValueType::String(String::from("tf-wa-20230322020900305100000006")),
            );
            options.insert("min".to_string(), ScalingTriggerValueType::Int(1));
            options.insert("max".to_string(), ScalingTriggerValueType::Int(10));
            options.insert("desired".to_string(), ScalingTriggerValueType::Int(1));
            scaling_trigger.apply(options).await;
        } else {
            assert!(false, "No metric adapter found")
        }
        Ok(())
    }
}
