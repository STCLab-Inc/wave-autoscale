use super::ScalingComponent;
use anyhow::{Ok, Result};
use async_trait::async_trait;
use data_layer::ScalingComponentDefinition;
use k8s_openapi::api::apps::v1::Deployment;
use kube::{
    api::{Api, Patch, PatchParams},
    client::ConfigExt,
    Client, Config,
};
use serde_json::{json, Value};
use std::collections::HashMap;

pub struct K8sDeploymentScalingComponent {
    definition: ScalingComponentDefinition,
}

impl K8sDeploymentScalingComponent {
    pub const TRIGGER_KIND: &'static str = "kubernetes-deployment";

    pub fn new(definition: ScalingComponentDefinition) -> Self {
        K8sDeploymentScalingComponent { definition }
    }

    async fn get_client(
        &self,
        api_server_endpoint: &String,
        ca_cert: &String,
        namespace: &String,
    ) -> Result<kube::Client> {
        // Infer the runtime environment and try to create a Kubernetes Client
        let client = Client::try_default().await?;
        Ok(client)
    }
}

#[async_trait]
impl ScalingComponent for K8sDeploymentScalingComponent {
    fn get_scaling_component_kind(&self) -> &str {
        &self.definition.component_kind
    }
    fn get_id(&self) -> &str {
        &self.definition.id
    }

    async fn apply(&self, params: HashMap<String, Value>) -> Result<()> {
        let metadata = self.definition.metadata.clone();

        if let (
            Some(Value::String(api_server_endpoint)),
            Some(Value::String(namespace)),
            Some(Value::String(name)),
            Some(Value::String(ca_cert)),
            Some(replicas),
        ) = (
            metadata.get("api_server_endpoint"),
            metadata.get("namespace"),
            metadata.get("name"),
            metadata.get("ca_cert"),
            params.get("replicas").and_then(Value::as_i64),
        ) {
            let client = self
                .get_client(api_server_endpoint, ca_cert, namespace)
                .await;
            if let Err(e) = client {
                return Err(anyhow::anyhow!(e));
            }
            let client = client.unwrap();

            let deployment_api: Api<Deployment> = Api::namespaced(client, &namespace);

            let patch = json!({
                "apiVersion": "apps/v1",
                "kind": "Deployment",
                "spec": {
                    "replicas": replicas
                }
            });

            let patch_params = PatchParams::apply("wave-autoscale");

            let result = deployment_api
                .patch(&name, &patch_params, &Patch::Apply(patch))
                .await;

            println!("{:?}", result);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Invalid metadata"))
        }
    }
}
