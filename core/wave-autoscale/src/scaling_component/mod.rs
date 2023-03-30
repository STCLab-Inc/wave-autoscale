use async_trait::async_trait;
use data_layer::ScalingComponentDefinition;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
pub mod aws_ec2_autoscaling;
use anyhow::Result;

use self::aws_ec2_autoscaling::EC2AutoScalingComponent;

pub fn create_scaling_component(
    definition: &ScalingComponentDefinition,
) -> Result<Box<dyn ScalingComponent + Send + Sync>> {
    // Get a value of metric and clone it.
    let cloned_defintion = definition.clone();
    match cloned_defintion.component_kind.as_str() {
        EC2AutoScalingComponent::TRIGGER_KIND => {
            Ok(Box::new(EC2AutoScalingComponent::new(cloned_defintion)))
        }
        _ => Err(anyhow::anyhow!("Unknown trigger kind")),
    }
}

#[async_trait]
pub trait ScalingComponent {
    async fn apply(&self, params: HashMap<String, Value>) -> Result<()>;
    fn get_scaling_component_kind(&self) -> &str;
    fn get_id(&self) -> &str;
}

pub struct ScalingComponentManager {
    scaling_components: HashMap<String, Box<dyn ScalingComponent + Send + Sync>>,
}

impl ScalingComponentManager {
    pub fn new() -> Self {
        ScalingComponentManager {
            scaling_components: HashMap::new(),
        }
    }

    pub fn add_definition(
        &mut self,
        scaling_component_definition: ScalingComponentDefinition,
    ) -> Result<()> {
        let scaling_component = create_scaling_component(&scaling_component_definition)?;
        self.add_scaling_component(scaling_component);
        Ok(())
    }

    pub fn add_definitions(
        &mut self,
        scaling_component_definitions: Vec<ScalingComponentDefinition>,
    ) -> Result<()> {
        for scaling_component_definition in scaling_component_definitions {
            self.add_definition(scaling_component_definition)?;
        }
        Ok(())
    }

    pub fn add_scaling_component(
        &mut self,
        scaling_component: Box<dyn ScalingComponent + Send + Sync>,
    ) {
        self.scaling_components
            .insert(scaling_component.get_id().to_string(), scaling_component);
    }

    pub fn get_scaling_component(
        &self,
        id: &str,
    ) -> Option<&Box<dyn ScalingComponent + Send + Sync>> {
        self.scaling_components.get(id)
    }

    pub async fn apply_to_scaling_component(
        &self,
        id: &str,
        params: HashMap<String, Value>,
    ) -> Result<()> {
        match self.scaling_components.get(id) {
            Some(scaling_component) => scaling_component.apply(params).await,
            None => Err(anyhow::anyhow!("Unknown scaling component kind")),
        }
    }
}
