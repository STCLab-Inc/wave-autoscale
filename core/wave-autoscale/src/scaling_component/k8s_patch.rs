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
    pub const SCALING_KIND: &'static str = "kubernetes-patch";

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

    async fn apply(&self, params: HashMap<String, serde_json::Value>) -> anyhow::Result<()> {
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

        let Ok(json_patch) = serde_json::from_value(serde_json::json!(json_patch)) else {
            return Err(anyhow::anyhow!("json patch not found"));
        };

        let result = api
            .patch(name, &patch_params, &Patch::Json::<()>(json_patch))
            .await;

        if let Err(e) = result {
            return Err(anyhow::anyhow!(e));
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn test_istio_vs_patch() {
        let client = Client::try_default().await;
        let client = client.unwrap();

        let patch_json = serde_json::json!([
            {
                "op": "add",
                "path": "/spec/http/0/fault",
                "value": {"delay" : { "fixedDelay": "10s" } }
            },
            {
                "op": "add",
                "path": "/spec/http/0/fault/delay/percentage",
                "value": { "value": 100 }
            }
        ]);
        println!(" >> patch_json: {:?}", patch_json);
        let api_version = "networking.istio.io/v1beta1";
        let api_group = api_version.split('/').next().unwrap();
        let kind = "VirtualService";

        // https://github.com/kube-rs/kube/blob/main/examples/crd_derive_schema.rs
        let apigroup = discovery::group(&client, api_group).await.unwrap();
        let recommended_kind = apigroup.recommended_kind(kind);
        let Some(api_resource) = recommended_kind else {
            assert!(false);
            return;
        };
        let namespace = "istio-system";
        let name = "istio-vs";
        let api: Api<DynamicObject> = Api::namespaced_with(client, namespace, &api_resource.0);
        // let get_object = api.get(name).await.unwrap();

        let patch_params = PatchParams::apply("wave-autoscale");
        // let patch_params = PatchParams::force(patch_params);

        let json_patch = serde_json::from_value(patch_json).unwrap();
        let result = api
            .patch(name, &patch_params, &Patch::Json::<()>(json_patch))
            .await;
        println!(" result: {:?}", result);
        assert!(result.is_ok());
    }

    #[ignore]
    #[tokio::test]
    async fn test_kubernetes_deployment_patch() {
        let client = Client::try_default().await;
        let client = client.unwrap();

        let patch_json = serde_json::json!([
            {
                "op": "add",
                "path": "/spec/replicas",
                "value": 2
            },
        ]);

        let api_version = "apps/v1";
        let api_group = api_version.split('/').next().unwrap();
        let kind = "Deployment";

        // https://github.com/kube-rs/kube/blob/main/examples/crd_derive_schema.rs
        let apigroup = discovery::group(&client, api_group).await.unwrap();
        let (ar, _caps) = apigroup.recommended_kind(kind).unwrap();
        let namespace = "wave-autoscale";
        let name = "product-server-dp";
        // Use the discovered kind in an Api, and Controller with the ApiResource as its DynamicType
        let api: Api<DynamicObject> = Api::namespaced_with(client, namespace, &ar);
        // let get_object = api.get(name).await.unwrap();
        
        let patch_params = PatchParams::apply("wave-autoscale");
        // let patch_params = PatchParams::force(patch_params);

        let json_patch = serde_json::from_value(patch_json).unwrap();
        let result = api
            .patch(name, &patch_params, &Patch::Json::<()>(json_patch))
            .await;
        println!(" result: {:?}", result);
        assert!(result.is_ok());
    }
}