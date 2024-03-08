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
use super::{evaluate_expression_with_current_state, filter_current_state_in_expression};
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
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

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
        _api_server_endpoint: Option<String>,
        _ca_cert: Option<String>,
        _namespace: Option<String>,
    ) -> anyhow::Result<kube::Client> {
        // TODO: Use the metadata to create a Kubernetes Client
        // Infer the runtime environment and try to create a Kubernetes Client
        let client = Client::try_default().await?;
        Ok(client)
    }
}

/*
 * Replicas - This indicates the desired number of pods that should be associated with the Deployment. This value is copied from the Deployment's specification (spec). Since this operation occurs asynchronously, there may be a brief interval during which spec.replicas does not match status.replicas.
 * availableReplicas - This indicates the total number of pods that the deployment aims to have available, with each of them being ready for at least minReadySeconds.
 * unavailableReplicas - This represents the total number of pods that must be unavailable for this deployment to achieve 100% available capacity. It includes both running but not yet available pods and pods that have not been created yet.
 * readyReplicas - readyReplicas represents the number of pods targeted by this Deployment that have achieved a 'Ready Condition'.
 * updatedReplicas - This represents the total number of non-terminated pods targeted by this deployment that have the desired template spec.
*/
#[derive(Debug, EnumIter)]
enum K8sComponentTargetValue {
    Replicas,
    UnavailableReplicas,
    AvailableReplicas,
    ReadyReplicas,
    UpdatedReplicas,
}
impl std::fmt::Display for K8sComponentTargetValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            K8sComponentTargetValue::Replicas => write!(f, "replicas"),
            K8sComponentTargetValue::UnavailableReplicas => write!(f, "unavailable_replicas"),
            K8sComponentTargetValue::AvailableReplicas => write!(f, "available_replicas"),
            K8sComponentTargetValue::ReadyReplicas => write!(f, "ready_replicas"),
            K8sComponentTargetValue::UpdatedReplicas => write!(f, "updated_replicas"),
        }
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

    async fn apply(
        &self,
        params: HashMap<String, Value>,
        context: rquickjs::AsyncContext,
    ) -> Result<HashMap<String, Value>> {
        let metadata = self.definition.metadata.clone();

        let (Some(Value::String(namespace)), Some(Value::String(name)), Some(replicas)) = (
            metadata.get("namespace"),
            metadata.get("name"),
            params.get("replicas"),
        ) else {
            return Err(anyhow::anyhow!("Invalid metadata"));
        };
        // TODO: Use the metadata to create a Kubernetes Client
        let api_server_endpoint = metadata
            .get("api_server_endpoint")
            .map(|api_server_endpoint| api_server_endpoint.to_string());
        let ca_cert = metadata.get("ca_cert").map(|ca_cert| ca_cert.to_string());
        let client = self
            .get_client(api_server_endpoint, ca_cert, Some(namespace.to_string()))
            .await;
        if let Err(e) = client {
            return Err(anyhow::anyhow!(e));
        }
        let client = client.unwrap();

        let replicas_value = match replicas {
            Value::String(replicas) => {
                // check target value contains enum variables
                let current_state_key_array = K8sComponentTargetValue::iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<String>>();
                let current_state_array =
                    filter_current_state_in_expression(replicas, current_state_key_array);
                // save target value to map
                let current_state_map = get_current_state_map(
                    current_state_array,
                    client.clone(),
                    namespace.to_string(),
                    name.to_string(),
                )
                .await;
                let core::result::Result::Ok(current_state_map) = current_state_map else {
                    return Err(current_state_map.unwrap_err());
                };

                // evaluate target value
                let replicas = evaluate_expression_with_current_state(
                    replicas,
                    current_state_map.clone(),
                    context,
                )
                .await;
                let core::result::Result::Ok(replicas) = replicas else {
                    return Err(replicas.unwrap_err());
                };
                core::result::Result::Ok(replicas as i64)
            }
            Value::Number(replicas) => {
                let Some(replicas) = replicas.as_f64() else {
                    return Err(anyhow::anyhow!("Invalid replicas"));
                };
                core::result::Result::Ok(replicas as i64)
            }
            _ => Err(anyhow::anyhow!("Invalid replicas")),
        };
        let core::result::Result::Ok(replicas_value) = replicas_value else {
            return Err(replicas_value.unwrap_err());
        };

        let deployment_api: Api<Deployment> = Api::namespaced(client, namespace);
        // https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.27/#deployment-v1-apps
        let patch = json!({
            "apiVersion": "apps/v1",
            "kind": "Deployment",
            "spec": {
                "replicas": replicas_value
            }
        });

        let patch_params = PatchParams::apply("wave-autoscale");
        let patch_params = PatchParams::force(patch_params);

        let result = deployment_api
            .patch(name, &patch_params, &Patch::Apply(patch))
            .await;

        if let Err(e) = result {
            return Err(anyhow::anyhow!(e));
        }
        let result = result.unwrap();

        if let Some(spec) = result.spec {
            if let Some(replicas_result) = spec.replicas {
                if replicas_result != replicas_value as i32 {
                    return Err(anyhow::anyhow!("Failed to scale deployment"));
                }
            } else {
                return Err(anyhow::anyhow!("Failed to scale deployment"));
            }
        } else {
            return Err(anyhow::anyhow!("Failed to scale deployment"));
        }

        // Reflect the result value.
        let mut return_params = params.clone();
        return_params.insert("replicas".to_string(), Value::from(replicas_value));
        Ok(return_params)
    }
}

async fn get_current_state_map(
    current_state_array: Vec<String>,
    client: Client,
    namespace: String,
    deployment_name: String,
) -> Result<HashMap<String, i64>, anyhow::Error> {
    let mut current_state_map: HashMap<String, i64> = HashMap::new();
    for current_state in current_state_array {
        let mut current_state_kind = K8sComponentTargetValue::Replicas;
        if current_state.eq(&format!("${}", K8sComponentTargetValue::Replicas)) {
            current_state_kind = K8sComponentTargetValue::Replicas;
        } else if current_state.eq(&format!("${}", K8sComponentTargetValue::AvailableReplicas)) {
            current_state_kind = K8sComponentTargetValue::AvailableReplicas;
        } else if current_state.eq(&format!(
            "${}",
            K8sComponentTargetValue::UnavailableReplicas
        )) {
            current_state_kind = K8sComponentTargetValue::UnavailableReplicas;
        } else if current_state.eq(&format!("${}", K8sComponentTargetValue::ReadyReplicas)) {
            current_state_kind = K8sComponentTargetValue::ReadyReplicas;
        } else if current_state.eq(&format!("${}", K8sComponentTargetValue::UpdatedReplicas)) {
            current_state_kind = K8sComponentTargetValue::UpdatedReplicas;
        }

        let replicas = get_deployment_replicas(
            client.clone(),
            &namespace,
            &deployment_name,
            current_state_kind,
        )
        .await;
        if replicas.is_err() {
            return Err(replicas.unwrap_err());
        };
        current_state_map.insert(current_state.clone(), replicas.unwrap() as i64);
    }
    Ok(current_state_map)
}

async fn get_deployment_replicas(
    client: Client,
    namespace: &str,
    deployment_name: &str,
    kind: K8sComponentTargetValue,
) -> Result<i32, anyhow::Error> {
    let deployment_api: Api<Deployment> = Api::namespaced(client, namespace);

    let deployment_get = deployment_api.get(deployment_name).await;
    if deployment_get.is_err() {
        return Err(anyhow::anyhow!(
            "Failed to get deployment - deployment get err"
        ));
    }
    let deployment_status = deployment_get.unwrap().status;
    if deployment_status.is_none() {
        return Err(anyhow::anyhow!("Failed to get deployment - status none"));
    }
    let status = match kind {
        K8sComponentTargetValue::Replicas => deployment_status.unwrap().replicas,
        K8sComponentTargetValue::UnavailableReplicas => {
            deployment_status.unwrap().unavailable_replicas
        }
        K8sComponentTargetValue::AvailableReplicas => deployment_status.unwrap().available_replicas,
        K8sComponentTargetValue::ReadyReplicas => deployment_status.unwrap().ready_replicas,
        K8sComponentTargetValue::UpdatedReplicas => deployment_status.unwrap().updated_replicas,
    };
    if status.is_none() {
        // return Err(anyhow::anyhow!("Failed to get deployment - status none"));
        return Ok(0);
    }
    Ok(status.unwrap())
}

#[cfg(test)]
mod test {
    use super::super::ScalingComponentManager;
    use super::*;
    use crate::scaling_component::test::get_rquickjs_context;
    use data_layer::types::object_kind::ObjectKind;

    fn get_data() -> (String, String, String, String) {
        (
            "api_server_endpoint".to_string(), // api_server_endpoint
            "default".to_string(),             // namespace
            "echo".to_string(),                // name
            "ca_cert".to_string(),             // ca_cert
        )
    }

    #[ignore]
    #[tokio::test]
    async fn test_get_deployment_replicas() {
        let client = Client::try_default().await;
        let replicas = get_deployment_replicas(
            client.unwrap(),
            get_data().1.as_str(),
            get_data().2.as_str(),
            K8sComponentTargetValue::Replicas,
        )
        .await;
        assert!(replicas.unwrap() > 0);
        let client = Client::try_default().await;
        let unavailable_replicas = get_deployment_replicas(
            client.unwrap(),
            get_data().1.as_str(),
            get_data().2.as_str(),
            K8sComponentTargetValue::UnavailableReplicas,
        )
        .await;
        assert!(unavailable_replicas.unwrap() >= 0);
    }

    #[ignore]
    #[tokio::test]
    async fn test_k8s_deployment() {
        let mut scaling_component_metadata = HashMap::new();
        scaling_component_metadata.insert(
            "api_server_endpoint".to_string(),
            serde_json::Value::String(get_data().0),
        );
        scaling_component_metadata.insert(
            "namespace".to_string(),
            serde_json::Value::String(get_data().1),
        );
        scaling_component_metadata
            .insert("name".to_string(), serde_json::Value::String(get_data().2));
        scaling_component_metadata.insert(
            "ca_cert".to_string(),
            serde_json::Value::String(get_data().3),
        );

        let scaling_component_definitions = vec![ScalingComponentDefinition {
            kind: ObjectKind::ScalingComponent,
            db_id: "".to_string(),
            id: "api_server".to_string(),
            component_kind: "kubernetes-deployment".to_string(),
            metadata: scaling_component_metadata,
            ..Default::default()
        }];

        // create metric adapter
        let mut scaling_component_manager = ScalingComponentManager::new();
        scaling_component_manager.add_definitions(scaling_component_definitions);

        // run scaling trigger
        let mut options: HashMap<String, serde_json::Value> = HashMap::new();
        options.insert(
            "replicas".to_string(),
            json!(
                "$replicas * $unavailable_replicas + $ready_replicas + $available_replicas + $updated_replicas"
                    .to_string()
            ),
        );

        let result = scaling_component_manager
            .apply_to("api_server", options, get_rquickjs_context().await)
            .await;
        assert!(result.is_ok());
    }
}
