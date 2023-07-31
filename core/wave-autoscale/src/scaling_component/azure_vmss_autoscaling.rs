use super::super::util::azure::{
    azure_virtual_machine_scale_sets::call_azure_patch_virtual_machine_scale_sets_capacity,
    AzureCredential,
};
use super::ScalingComponent;
use async_trait::async_trait;
use data_layer::ScalingComponentDefinition;
use log::error;
use std::collections::HashMap;

pub struct VMSSAutoScalingComponent {
    definition: ScalingComponentDefinition,
}

impl VMSSAutoScalingComponent {
    pub const SCALING_KIND: &'static str = "azure-virtual-machine-scale-sets";

    pub fn new(definition: ScalingComponentDefinition) -> Self {
        VMSSAutoScalingComponent { definition }
    }
}

#[async_trait]
impl ScalingComponent for VMSSAutoScalingComponent {
    fn get_scaling_component_kind(&self) -> &str {
        &self.definition.component_kind
    }
    fn get_id(&self) -> &str {
        &self.definition.id
    }

    async fn apply(&self, params: HashMap<String, serde_json::Value>) -> anyhow::Result<()> {
        let metadata: HashMap<String, serde_json::Value> = self.definition.metadata.clone();
        if let (
            Some(serde_json::Value::String(client_id)),
            Some(serde_json::Value::String(client_secret)),
            Some(serde_json::Value::String(tenant_id)),
            Some(serde_json::Value::String(subscription_id)),
            Some(serde_json::Value::String(resource_group_name)),
            Some(serde_json::Value::String(vm_scale_set_name)),
            Some(capacity),
        ) = (
            metadata.get("client_id"),
            metadata.get("client_secret"),
            metadata.get("tenant_id"),
            metadata.get("subscription_id"),
            metadata.get("resource_group_name"),
            metadata.get("vm_scale_set_name"),
            params.get("capacity").and_then(serde_json::Value::as_u64),
        ) {
            let azure_credential = AzureCredential {
                client_id: client_id.to_string(),
                client_secret: client_secret.to_string(),
                tenant_id: tenant_id.to_string(),
            };

            let response = call_azure_patch_virtual_machine_scale_sets_capacity(
                azure_credential,
                subscription_id.to_string(),
                resource_group_name.to_string(),
                vm_scale_set_name.to_string(),
                capacity as u32,
            )
            .await;

            if response.is_err() {
                error!("ERROR: patch azure vmss - capacity");
                return Err(anyhow::anyhow!("ERROR: patch azure vmss - capacity"));
            }
            if !response.unwrap().status().is_success() {
                error!("ERROR: patch azure vmss - capacity update fail");
                return Err(anyhow::anyhow!(
                    "ERROR: patch azure vmss - capacity update fail"
                ));
            }
        } else {
            error!("Invalid metadata");
            return Err(anyhow::anyhow!("Invalid metadata"));
        }
        Ok(())
    }
}
