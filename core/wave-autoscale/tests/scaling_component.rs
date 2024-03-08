pub mod scaling_component_common {
    pub async fn get_rquickjs_context() -> rquickjs::AsyncContext {
        rquickjs::AsyncContext::full(&rquickjs::AsyncRuntime::new().unwrap())
            .await
            .unwrap()
    }
}

mod scaling_component_test {
    use std::collections::HashMap;

    use super::scaling_component_common::get_rquickjs_context;
    use anyhow::Result;
    use data_layer::reader::wave_definition_reader::read_definition_yaml_file;
    use serde_json::{json, Value};
    use wave_autoscale::scaling_component::{
        aws_ec2_autoscaling::EC2AutoScalingComponent,
        k8s_deployment::K8sDeploymentScalingComponent, ScalingComponentManager,
    };

    const EC2_AUTOSCALING_FILE_PATH: &str = "./tests/yaml/component_ec2_autoscaling.yaml";

    // multithreaded test
    #[tokio::test]
    #[ignore]
    async fn ec2_autoscaling() -> Result<()> {
        // read yaml file
        let result = read_definition_yaml_file(EC2_AUTOSCALING_FILE_PATH)?;

        // create metric adapter
        let mut scaling_component_manager = ScalingComponentManager::new();
        scaling_component_manager.add_definitions(result.scaling_component_definitions);

        if let Some(scaling_component) =
            scaling_component_manager.get_scaling_component("ec2_autoscaling_api_server")
        {
            let name = scaling_component.get_scaling_component_kind();
            assert!(name == EC2AutoScalingComponent::SCALING_KIND, "Unexpected");
        } else {
            assert!(false, "No scaling component found")
        }

        // run scaling trigger
        let mut options: HashMap<String, Value> = HashMap::new();
        options.insert("min".to_string(), json!(1));
        options.insert("max".to_string(), json!(5));
        options.insert("desired".to_string(), json!(1));
        let result = scaling_component_manager
            .apply_to(
                "ec2_autoscaling_api_server",
                options,
                get_rquickjs_context().await,
            )
            .await;
        assert!(result.is_ok());
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn k8s_deployment_autoscaling() -> Result<()> {
        // read yaml file
        let result = read_definition_yaml_file("./tests/yaml/component_k8s_deployment.yaml")?;
        // create metric adapter
        let mut scaling_component_manager = ScalingComponentManager::new();
        scaling_component_manager.add_definitions(result.scaling_component_definitions);

        if let Some(scaling_component) =
            scaling_component_manager.get_scaling_component("k8s_deployment")
        {
            let name = scaling_component.get_scaling_component_kind();
            assert!(
                name == K8sDeploymentScalingComponent::SCALING_KIND,
                "Unexpected"
            );
        } else {
            assert!(false, "No scaling component found")
        }

        // run scaling trigger
        let mut options: HashMap<String, Value> = HashMap::new();
        options.insert("replicas".to_string(), json!(5));

        let result = scaling_component_manager
            .apply_to("k8s_deployment", options, get_rquickjs_context().await)
            .await;
        assert!(result.is_ok());
        Ok(())
    }
}
