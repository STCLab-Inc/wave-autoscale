/**
 * [Scaling Component] Kubernetes Deployment Scaling Component
 *
 * This component is used to scale a Kubernetes Deployment
 * It requires the following metadata:
 * - api_server_endpoint: The API server endpoint
 * - namespace: The namespace of the deployment
 * - name: The name of the deployment
 * - ca_cert: The CA certificate of the API server
 * It requires the following parameters:
 * - replicas: The number of replicas to scale to
 * It requires the following environment variables:
 * - KUBECONFIG: The path to the kubeconfig file
 *
 */
use super::ScalingComponent;
use anyhow::{Ok, Result};
use async_trait::async_trait;
use data_layer::ScalingComponentDefinition;
use k8s_openapi::api::apps::v1::Deployment;
use kube::{
    api::{Api, Patch, PatchParams},
    Client,
};
use serde_json::{json, Value};
use std::collections::HashMap;

pub struct K8sDeploymentScalingComponent {
    definition: ScalingComponentDefinition,
}

impl K8sDeploymentScalingComponent {
    pub const SCALING_KIND: &'static str = "kubernetes-deployment";

    pub fn new(definition: ScalingComponentDefinition) -> Self {
        K8sDeploymentScalingComponent { definition }
    }

    async fn get_client(
        &self,
        _api_server_endpoint: &String,
        _ca_cert: &String,
        _namespace: &String,
    ) -> Result<kube::Client> {
        // TODO: Use the metadata to create a Kubernetes Client
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
            // TODO: Use the metadata to create a Kubernetes Client
            let client = self
                .get_client(api_server_endpoint, ca_cert, namespace)
                .await;
            if let Err(e) = client {
                return Err(anyhow::anyhow!(e));
            }
            let client = client.unwrap();

            let deployment_api: Api<Deployment> = Api::namespaced(client, namespace);
            // https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.27/#deployment-v1-apps
            let patch = json!({
                "apiVersion": "apps/v1",
                "kind": "Deployment",
                "spec": {
                    "replicas": replicas
                }
            });

            let patch_params = PatchParams::apply("wave-autoscale");

            let result = deployment_api
                .patch(name, &patch_params, &Patch::Apply(patch))
                .await;

            println!("{:#?}", result);
            if let Err(e) = result {
                return Err(anyhow::anyhow!(e));
            }
            let result = result.unwrap();

            if let Some(spec) = result.spec {
                if let Some(replicas_result) = spec.replicas {
                    println!("{}: {} -> {}", name, replicas_result, replicas);
                    if replicas_result != replicas as i32 {
                        return Err(anyhow::anyhow!("Failed to scale deployment"));
                    }
                } else {
                    return Err(anyhow::anyhow!("Failed to scale deployment"));
                }
            } else {
                return Err(anyhow::anyhow!("Failed to scale deployment"));
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("Invalid metadata"))
        }
    }
}
