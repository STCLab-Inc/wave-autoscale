mod test_gcp_mig_scaling {
    use anyhow::Result;
    use data_layer::reader::wave_definition_reader::read_definition_yaml_file;
    use serde_json::{json, Value};
    use std::collections::HashMap;
    use wave_autoscale::scaling_component::{
        gcp_mig_autoscaling::MIGAutoScalingComponent, ScalingComponentManager,
    };

    const GCP_MIG_AUTOSCALING_FILE_PATH: &str = "./tests/yaml/plan_gcp_mig.yaml";

    #[tokio::test]
    //#[ignore]
    async fn test_gcp_mig_autoscaling() -> Result<()> {
        //let file = std::fs::File::open(GCP_MIG_AUTOSCALING_FILE_PATH)?;
        // read yaml file
        let result = read_definition_yaml_file(GCP_MIG_AUTOSCALING_FILE_PATH)?;

        // create metric adapter
        let mut scaling_component_manager = ScalingComponentManager::new();
        scaling_component_manager.add_definitions(result.scaling_component_definitions);

        if let Some(scaling_component) =
            scaling_component_manager.get_scaling_component("gcp_mig_region_autoscaling_api_server")
        {
            let name = scaling_component.get_scaling_component_kind();
            assert!(name == MIGAutoScalingComponent::SCALING_KIND, "Unexpected");
        } else {
            assert!(false, "No scaling component found")
        }

        // run scaling trigger
        let mut options: HashMap<String, Value> = HashMap::new();
        options.insert("resize".to_string(), json!(2));

        scaling_component_manager
            .apply_to("gcp_mig_region_autoscaling_api_server", options)
            .await
    }
}
