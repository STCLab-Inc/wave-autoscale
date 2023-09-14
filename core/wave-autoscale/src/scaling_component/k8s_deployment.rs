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
use super::{get_target_value_result, target_value_expression_regex_filter};
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

/*
 * Replicas - This indicates the desired number of pods that should be associated with the Deployment. This value is copied from the Deployment's specification (spec). Since this operation occurs asynchronously, there may be a brief interval during which spec.replicas does not match status.replicas.
 * availableReplicas - This indicates the total number of pods that the deployment aims to have available, with each of them being ready for at least minReadySeconds.
 * unavailableReplicas - This represents the total number of pods that must be unavailable for this deployment to achieve 100% available capacity. It includes both running but not yet available pods and pods that have not been created yet.
 * readyReplicas - readyReplicas represents the number of pods targeted by this Deployment that have achieved a 'Ready Condition'.
 * updatedReplicas - This represents the total number of non-terminated pods targeted by this deployment that have the desired template spec.
*/
#[derive(Debug, EnumIter)]
enum EKSComponentTargetValue {
    Replicas,
    UnavailableReplicas,
    AvailableReplicas,
    ReadyReplicas,
    UpdatedReplicas,
}
impl std::fmt::Display for EKSComponentTargetValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EKSComponentTargetValue::Replicas => write!(f, "replicas"),
            EKSComponentTargetValue::UnavailableReplicas => write!(f, "unavailable_replicas"),
            EKSComponentTargetValue::AvailableReplicas => write!(f, "available_replicas"),
            EKSComponentTargetValue::ReadyReplicas => write!(f, "ready_replicas"),
            EKSComponentTargetValue::UpdatedReplicas => write!(f, "updated_replicas"),
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

    async fn apply(&self, params: HashMap<String, Value>) -> Result<()> {
        let metadata = self.definition.metadata.clone();

        if let (
            Some(Value::String(api_server_endpoint)),
            Some(Value::String(namespace)),
            Some(Value::String(name)),
            Some(Value::String(ca_cert)),
            Some(Value::String(replicas)),
        ) = (
            metadata.get("api_server_endpoint"),
            metadata.get("namespace"),
            metadata.get("name"),
            metadata.get("ca_cert"),
            params.get("replicas"),
        ) {
            // TODO: Use the metadata to create a Kubernetes Client
            let client = self
                .get_client(api_server_endpoint, ca_cert, namespace)
                .await;
            if let Err(e) = client {
                return Err(anyhow::anyhow!(e));
            }
            let client = client.unwrap();

            // check target value contains enum variables
            let target_value_key_array = EKSComponentTargetValue::iter()
                .map(|value| value.to_string())
                .collect::<Vec<String>>();
            let target_value_array =
                target_value_expression_regex_filter(replicas, target_value_key_array);
            // save target value to map
            let target_value_map = get_target_value_map(
                target_value_array,
                client.clone(),
                namespace.to_string(),
                name.to_string(),
            )
            .await;
            if target_value_map.is_err() {
                return Err(target_value_map.unwrap_err());
            };

            // evaluate target value
            let replicas =
                get_target_value_result(replicas, target_value_map.unwrap().clone()).await;
            if replicas.is_err() {
                return Err(replicas.unwrap_err());
            };
            let replicas = replicas.unwrap();

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
            let patch_params = PatchParams::force(patch_params);

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

async fn get_target_value_map(
    target_value_array: Vec<String>,
    client: Client,
    namespace: String,
    deployment_name: String,
) -> Result<HashMap<String, i64>, anyhow::Error> {
    let mut target_value_map: HashMap<String, i64> = HashMap::new();
    for target_value in target_value_array {
        let mut target_value_kind = EKSComponentTargetValue::Replicas;
        if target_value.eq(&format!("${}", EKSComponentTargetValue::Replicas)) {
            target_value_kind = EKSComponentTargetValue::Replicas;
        } else if target_value.eq(&format!("${}", EKSComponentTargetValue::AvailableReplicas)) {
            target_value_kind = EKSComponentTargetValue::AvailableReplicas;
        } else if target_value.eq(&format!(
            "${}",
            EKSComponentTargetValue::UnavailableReplicas
        )) {
            target_value_kind = EKSComponentTargetValue::UnavailableReplicas;
        } else if target_value.eq(&format!("${}", EKSComponentTargetValue::ReadyReplicas)) {
            target_value_kind = EKSComponentTargetValue::ReadyReplicas;
        } else if target_value.eq(&format!("${}", EKSComponentTargetValue::UpdatedReplicas)) {
            target_value_kind = EKSComponentTargetValue::UpdatedReplicas;
        }

        let replicas = get_deployment_replicas(
            client.clone(),
            &namespace,
            &deployment_name,
            target_value_kind,
        )
        .await;
        if replicas.is_err() {
            return Err(replicas.unwrap_err());
        };
        target_value_map.insert(target_value.clone(), replicas.unwrap() as i64);
    }
    Ok(target_value_map)
}

async fn get_deployment_replicas(
    client: Client,
    namespace: &str,
    deployment_name: &str,
    kind: EKSComponentTargetValue,
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
        EKSComponentTargetValue::Replicas => deployment_status.unwrap().replicas,
        EKSComponentTargetValue::UnavailableReplicas => {
            deployment_status.unwrap().unavailable_replicas
        }
        EKSComponentTargetValue::AvailableReplicas => deployment_status.unwrap().available_replicas,
        EKSComponentTargetValue::ReadyReplicas => deployment_status.unwrap().ready_replicas,
        EKSComponentTargetValue::UpdatedReplicas => deployment_status.unwrap().updated_replicas,
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
    use data_layer::types::object_kind::ObjectKind;

    fn get_data() -> (String, String, String, String) {
        (
            "https://3AAD8E2817F0CB2492BCD34FA32CAC09.gr7.ap-northeast-2.eks.amazonaws.com".to_string(),        // api_server_endpoint
            "default".to_string(), // namespace
            "echo".to_string(),    // name
            "LS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0tCk1JSURCVENDQWUyZ0F3SUJBZ0lJU0JEVXNQc3ZkY013RFFZSktvWklodmNOQVFFTEJRQXdGVEVUTUJFR0ExVUUKQXhNS2EzVmlaWEp1WlhSbGN6QWVGdzB5TXpBNU1UUXdOREkxTlROYUZ3MHpNekE1TVRFd05ESTFOVE5hTUJVeApFekFSQmdOVkJBTVRDbXQxWW1WeWJtVjBaWE13Z2dFaU1BMEdDU3FHU0liM0RRRUJBUVVBQTRJQkR3QXdnZ0VLCkFvSUJBUURpQjBiNHZzT29Nd0x0OXVpOTU4L2xrWDhUMTJ6REF3SUVXNE92WTJwb2lOazZOSnpUcEgwOU1HaGsKdzYvWXhabndobitoNVFEUitDdG1vSGx0T2xpUVdleXI3Tm5nLzl3elBtMTl6NE9qS3M3T1hKT2l1dThJRGQzeQpzTU44NVF3RC9BTWJqb1hkSlFNZmVGOWZ0QjVxdXFPTGVVeVBmdVVyZGZTYW1Lak5oaHFYWHVRR0pabXordWthCng1NjBpUTJhVklzUlFNWmp4QUlzU2lZcmU2SitsL3pOck5PZUhSeEhxa2ZDNWRaZXRTK3paaG5HaXQvMEVjK3MKWnQ4T085R2RtL2k3THdJMWRHVFdkVmhEWTAvbzJBNmZLSXBjMjUvaFR0QUJiYW5ZQ2NpWEJDVkVwaVJ3SkFZdQpuT1VnZjNSa1JqUE1DUkhXaFVwb0NIUHJlb2EzQWdNQkFBR2pXVEJYTUE0R0ExVWREd0VCL3dRRUF3SUNwREFQCkJnTlZIUk1CQWY4RUJUQURBUUgvTUIwR0ExVWREZ1FXQkJRai9ScnBIRG4zQlNJWEZPYXRpbzV1WHQwRzREQVYKQmdOVkhSRUVEakFNZ2dwcmRXSmxjbTVsZEdWek1BMEdDU3FHU0liM0RRRUJDd1VBQTRJQkFRQnE0VmNqS0xsawpFckZjM0VTMFh1VjM0aTZZSWJtdXhaV2FodmZuUk1pTUR4QklNRE5oWDZPWDFVTDNncmhuejZRQVpwV0VMa3JsCml5U0Z0T1MzWk9rTmZRWWZET3haUGVZbkVNMHRNY3BXbER1OW1MY21WWWNWWTZVR3o4OWoyRlg4Qm9UREQ5b0EKZlU1cGhzdmpBREVJaktEM1V3SHhYYWpGd0FZL2Y4SXZvWXRzVVcvQityRlR6MlUydnduVjFaZjlHaFZGNSs4RwovbEgxV3AxSW5rWjltbno0U1NQdWRvWDZiZXBNNGxOeDRpYnRTNzcvSXJpZGFaT1MyTTZpNDdtMVRLL1lFbk9PCit3ME5nOHkwa3o4Yk04NmV5V3QveHJJSWN0ZndmNzB3UUlBQUxnU1RvWjB3NEdtbmpvV1ZHT3dVK0ViRDI5eXUKcXJRSnVmUXhvdHVPCi0tLS0tRU5EIENFUlRJRklDQVRFLS0tLS0K".to_string(),        // ca_cert
        )
    }

    #[tokio::test]
    async fn test_get_deployment_replicas() {
        let client = Client::try_default().await;
        let replicas = get_deployment_replicas(
            client.unwrap(),
            get_data().1.as_str(),
            get_data().2.as_str(),
            EKSComponentTargetValue::Replicas,
        )
        .await;
        assert!(replicas.unwrap() > 0);
        let client = Client::try_default().await;
        let unavailable_replicas = get_deployment_replicas(
            client.unwrap(),
            get_data().1.as_str(),
            get_data().2.as_str(),
            EKSComponentTargetValue::UnavailableReplicas,
        )
        .await;
        assert!(unavailable_replicas.unwrap() >= 0);
    }

    // #[ignore]
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
            .apply_to("api_server", options)
            .await;
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}
