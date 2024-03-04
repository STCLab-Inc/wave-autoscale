use super::ScalingComponent;
use async_trait::async_trait;
use data_layer::ScalingComponentDefinition;
use kube::{
    api::{Api, DynamicObject, Patch, PatchParams},
    discovery, Client,
};
use std::collections::HashMap;


pub struct K8sPatchScalingComponent {
    definition: ScalingComponentDefinition,
}

impl K8sPatchScalingComponent {
    pub const SCALING_KIND: &'static str = "kubernetes-json-patch";

    pub fn new(definition: ScalingComponentDefinition) -> Self {
        K8sPatchScalingComponent { definition }
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

#[async_trait]
impl ScalingComponent for K8sPatchScalingComponent {
    fn get_scaling_component_kind(&self) -> &str {
        &self.definition.component_kind
    }
    fn get_id(&self) -> &str {
        &self.definition.id
    }

    async fn apply(&self, params: HashMap<String, serde_json::Value>, context: rquickjs::AsyncContext) -> anyhow::Result<HashMap<String, serde_json::Value>> {
        let metadata = self.definition.metadata.clone();

        let (
            Some(serde_json::Value::String(namespace)),
            Some(serde_json::Value::String(name)),
            Some(serde_json::Value::String(api_version)),
            Some(serde_json::Value::String(kind)),
            Some(serde_json::Value::Array(json_patch)) 
            ) = (
            params.get("namespace"),
            params.get("name"),
            params.get("api_version"),
            params.get("kind"),
            params.get("json_patch"),
        ) else {
            return Err(anyhow::anyhow!("Invalid metadata"));
        };

        // TODO: Use the metadata to create a Kubernetes Client
        let api_server_endpoint = metadata
                .get("api_server_endpoint")
                .map(|api_server_endpoint| api_server_endpoint.to_string());
        let ca_cert = metadata
                .get("ca_cert")
                .map(|ca_cert| ca_cert.to_string());
        let Ok(client) = self.get_client(api_server_endpoint, ca_cert, Some(namespace.to_string())).await else {
            return Err(anyhow::anyhow!("cannot create kubernetes client"));
        };

        let Some(api_group) = api_version.split('/').next() else {
            return Err(anyhow::anyhow!("api Group not found"));
        };

        let Ok(apigroup) = discovery::group(&client, api_group).await else {
            return Err(anyhow::anyhow!("api group not found"));
        };
        let recommended_kind = apigroup.recommended_kind(kind);
        let Some(api_resource) = recommended_kind else {
            return Err(anyhow::anyhow!("api group resource not found"));
        };
        let api: Api<DynamicObject> = Api::namespaced_with(client, namespace, &api_resource.0);

        let patch_params = PatchParams::apply("wave-autoscale");
        // let patch_params = PatchParams::force(patch_params);

        let Ok(json_patch) = serde_json::from_value(serde_json::json!(json_patch)) else {
            return Err(anyhow::anyhow!("json patch not found"));
        };

        let result = api
            .patch(name, &patch_params, &Patch::Json::<()>(json_patch))
            .await;

        if let Err(e) = result {
            return Err(anyhow::anyhow!(e));
        }

        Ok(params)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn get_value_of_istio_delay_add() -> serde_json::Value {
        serde_json::json!([
            {
                "op": "add",
                "path": "/spec/http/0/fault",
                "value": {"delay" : { "fixedDelay": "10s" } }
            },
            {
                "op": "add",
                "path": "/spec/http/1/fault",
                "value": {"delay" : { "fixedDelay": "10s" } }
            },
            {
                "op": "add",
                "path": "/spec/http/0/fault/delay/percentage",
                "value": { "value": 100 }
            },
            {
                "op": "add",
                "path": "/spec/http/1/fault/delay/percentage",
                "value": { "value": 100 }
            }
        ])
    }

    fn get_value_of_istio_delay_remove() -> serde_json::Value {
        serde_json::json!([
            {
                "op": "add",
                "path": "/spec/http/0/fault",
                "value": null
            },
            {
                "op": "add",
                "path": "/spec/http/1/fault",
                "value": null
            }
        ])
    }

    fn get_value_of_istio_delay_remove_fail() -> serde_json::Value {
        serde_json::json!([
            {
                "op": "remove",
                "path": "/spec/http/0/fault",
            },
            {
                "op": "remove",
                "path": "/spec/http/1/fault",
            }
        ])
    }

    fn get_value_of_istio_retry_add() -> serde_json::Value {
        serde_json::json!([
            {
                "op": "add",
                "path": "/spec/http/1/retries",
                "value": { "attempts": 3, "perTryTimeout": "2s" }
            },
        ])
    }

    fn get_value_of_istio_weight_add() -> serde_json::Value {
        serde_json::json!([
            {
                "op": "add",
                "path": "/spec/http/0/route/0/weight",
                "value": 75
            },
            {
                "op": "add",
                "path": "/spec/http/1/route/0/weight",
                "value": 25
            },
        ])
    }
    

    fn get_value_of_deployment_replicas_add(replicas: u16) -> serde_json::Value {
        serde_json::json!([
            {
                "op": "add",
                "path": "/spec/replicas",
                "value": replicas
            },
        ])
    }

    fn get_value_of_deployment_resource_cpu_mem_add(cpu: String, mem: String) -> serde_json::Value {
        serde_json::json!([
            {
                "op": "add",
                "path": "/spec/template/spec/containers/0/resources/requests",
                "value": {"cpu": cpu, "memory": mem}
            },
        ])
    }

    fn get_scaling_component() -> K8sPatchScalingComponent {
        K8sPatchScalingComponent::new(ScalingComponentDefinition {
            kind: data_layer::types::object_kind::ObjectKind::ScalingComponent,
            db_id: "".to_string(),
            id: "id".to_string(),
            component_kind: "kubernetes-json-patch".to_string(),
            metadata: HashMap::new(),
            ..Default::default()
        })
    }

    async fn get_rquickjs_context() -> rquickjs::AsyncContext {
        rquickjs::AsyncContext::full(&rquickjs::AsyncRuntime::new().unwrap())
            .await
            .unwrap()
    }


    #[ignore]
    #[tokio::test]
    async fn test_istio_vs_delay_patch() {
        let scaling_component = get_scaling_component();
        
        let mut apply_params = HashMap::new();
        apply_params.insert("namespace".to_string(), serde_json::Value::String("istio-system".to_string()));
        apply_params.insert("name".to_string(), serde_json::Value::String("istio-vs".to_string()));
        apply_params.insert("api_version".to_string(), serde_json::Value::String("networking.istio.io/v1beta1".to_string()));
        apply_params.insert("kind".to_string(), serde_json::Value::String("VirtualService".to_string()));
        apply_params.insert("json_patch".to_string(), get_value_of_istio_delay_add());

        let istio_delay_add_result = scaling_component.apply(apply_params.clone(), get_rquickjs_context().await);
        assert!(istio_delay_add_result.await.is_ok());

        apply_params.insert("json_patch".to_string(), get_value_of_istio_delay_remove());
        let istio_delay_remove_result = scaling_component.apply(apply_params, get_rquickjs_context().await);
        assert!(istio_delay_remove_result.await.is_ok());
    }

    #[ignore]
    #[tokio::test]
    async fn test_istio_vs_delay_patch_fail() {
        // An error occurs if the remove patch is duplicated according to the plan schedule.
        // Therefore, create a json patch in the form {"op":"add", .. "value": null}.
        let scaling_component = get_scaling_component();
        let mut apply_params = HashMap::new();
        apply_params.insert("namespace".to_string(), serde_json::Value::String("istio-system".to_string()));
        apply_params.insert("name".to_string(), serde_json::Value::String("istio-vs".to_string()));
        apply_params.insert("api_version".to_string(), serde_json::Value::String("networking.istio.io/v1beta1".to_string()));
        apply_params.insert("kind".to_string(), serde_json::Value::String("VirtualService".to_string()));
        apply_params.insert("json_patch".to_string(), get_value_of_istio_delay_add());

        let istio_delay_add_result = scaling_component.apply(apply_params.clone(), get_rquickjs_context().await);
        assert!(istio_delay_add_result.await.is_ok());

        apply_params.insert("json_patch".to_string(), get_value_of_istio_delay_remove_fail());
        let istio_delay_remove_result = scaling_component.apply(apply_params.clone(), get_rquickjs_context().await);
        assert!(istio_delay_remove_result.await.is_ok());

        apply_params.insert("json_patch".to_string(), get_value_of_istio_delay_remove_fail());
        let istio_delay_remove_result = scaling_component.apply(apply_params, get_rquickjs_context().await);
        assert!(istio_delay_remove_result.await.is_err());
    }

    #[ignore]
    #[tokio::test]
    async fn test_istio_vs_retry_patch() {
        let scaling_component = get_scaling_component();
        
        let mut apply_params = HashMap::new();
        apply_params.insert("namespace".to_string(), serde_json::Value::String("istio-system".to_string()));
        apply_params.insert("name".to_string(), serde_json::Value::String("istio-vs".to_string()));
        apply_params.insert("api_version".to_string(), serde_json::Value::String("networking.istio.io/v1beta1".to_string()));
        apply_params.insert("kind".to_string(), serde_json::Value::String("VirtualService".to_string()));
        apply_params.insert("json_patch".to_string(), get_value_of_istio_retry_add());

        let istio_delay_add_result = scaling_component.apply(apply_params.clone(), get_rquickjs_context().await);
        assert!(istio_delay_add_result.await.is_ok());
    }

    #[ignore]
    #[tokio::test]
    async fn test_istio_vs_weight_patch() {
        let scaling_component = get_scaling_component();
        
        let mut apply_params = HashMap::new();
        apply_params.insert("namespace".to_string(), serde_json::Value::String("istio-system".to_string()));
        apply_params.insert("name".to_string(), serde_json::Value::String("istio-vs".to_string()));
        apply_params.insert("api_version".to_string(), serde_json::Value::String("networking.istio.io/v1beta1".to_string()));
        apply_params.insert("kind".to_string(), serde_json::Value::String("VirtualService".to_string()));
        apply_params.insert("json_patch".to_string(), get_value_of_istio_weight_add());

        let istio_delay_add_result = scaling_component.apply(apply_params.clone(), get_rquickjs_context().await);
        assert!(istio_delay_add_result.await.is_ok());
    }

    #[ignore]
    #[tokio::test]
    async fn test_kubernetes_deployment_replicas_patch() {
        let scaling_component = get_scaling_component();
        let mut apply_params = HashMap::new();
        apply_params.insert("namespace".to_string(), serde_json::Value::String("wave-autoscale".to_string()));
        apply_params.insert("name".to_string(), serde_json::Value::String("product-server-dp".to_string()));
        apply_params.insert("api_version".to_string(), serde_json::Value::String("apps/v1".to_string()));
        apply_params.insert("kind".to_string(), serde_json::Value::String("Deployment".to_string()));
        apply_params.insert("json_patch".to_string(), get_value_of_deployment_replicas_add(2));

        let deployment_replicas_add_result = scaling_component.apply(apply_params.clone(), get_rquickjs_context().await);
        assert!(deployment_replicas_add_result.await.is_ok());

        apply_params.insert("json_patch".to_string(), get_value_of_deployment_replicas_add(1));

        let deployment_replicas_add_result = scaling_component.apply(apply_params.clone(), get_rquickjs_context().await);
        assert!(deployment_replicas_add_result.await.is_ok());
    }

    #[ignore]
    #[tokio::test]
    async fn test_kubernetes_deployment_cpu_mem_patch() {
        let scaling_component = get_scaling_component();
        let mut apply_params = HashMap::new();
        apply_params.insert("namespace".to_string(), serde_json::Value::String("wave-autoscale".to_string()));
        apply_params.insert("name".to_string(), serde_json::Value::String("product-server-dp".to_string()));
        apply_params.insert("api_version".to_string(), serde_json::Value::String("apps/v1".to_string()));
        apply_params.insert("kind".to_string(), serde_json::Value::String("Deployment".to_string()));
        apply_params.insert("json_patch".to_string(), get_value_of_deployment_resource_cpu_mem_add("250m".to_string(), "64Mi".to_string()));

        let deployment_replicas_add_result = scaling_component.apply(apply_params.clone(), get_rquickjs_context().await);
        assert!(deployment_replicas_add_result.await.is_ok());
    }
}