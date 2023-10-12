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
            Some(serde_json::Value::Object(manifest)) 
            ) = (
            params.get("namespace"),
            params.get("name"),
            params.get("manifest"),
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

        let Some(api_version) = manifest.get("apiVersion") else {
            return Err(anyhow::anyhow!("apiVersion not found"));
        };
        let Some(api_version) = api_version.as_str() else {
            return Err(anyhow::anyhow!("apiVersion string not found"));
        };
        let Some(api_group) = api_version.split('/').next() else {
            return Err(anyhow::anyhow!("api Group not found"));
        };
        let Some(kind) = manifest.get("kind") else {
            return Err(anyhow::anyhow!("kind not found"));
        };
        let Some(kind) = kind.as_str() else {
            return Err(anyhow::anyhow!("kind string not found"));
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
        let patch_params = PatchParams::force(patch_params);

        let result = api
            .patch(name, &patch_params, &Patch::Apply(manifest))
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

        let patch_yaml = r#"
        apiVersion: networking.istio.io/v1beta1
        kind: VirtualService
        spec:
          hosts:
            - "*"
          gateways:
            - istio-gateway
          http:
            - match:
                - uri:
                    prefix: /product/
                  port: 443
              rewrite:
                uri: "/"
              route:
                - destination:
                    host: product-server-sv.wave-autoscale.svc.cluster.local
                    port:
                      number: 5001
                  weight: 20
            - match:
                - uri:
                    prefix: /order/
                  port: 443
              rewrite:
                uri: "/"
              route:
                - destination:
                    host: order-server-sv.wave-autoscale.svc.cluster.local
                    port:
                      number: 5002
                  weight: 20
        "#;

        let patch_json = serde_yaml::from_str::<serde_json::Value>(patch_yaml).unwrap();
        println!("patch_json:\n {:#?}", patch_json);
        let api_version = patch_json.get("apiVersion").unwrap();
        let api_group = api_version.as_str().unwrap().split('/').next().unwrap();
        let kind = patch_json.get("kind").unwrap().as_str().unwrap();

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
        // println!(" >> get_object :\n{:?}", get_object);

        let patch_params = PatchParams::apply("wave-autoscale");
        let patch_params = PatchParams::force(patch_params);

        let result = api
            .patch(name, &patch_params, &Patch::Apply(patch_json))
            .await;

        println!("result: {:#?}", result);
        assert!(result.is_ok());
    }

    #[ignore]
    #[tokio::test]
    async fn test_kubernetes_deployment_patch() {
        let client = Client::try_default().await;
        let client = client.unwrap();

        let patch_yaml = r#"
        apiVersion: apps/v1
        kind: Deployment
        spec:
          replicas: 1
          selector:
            matchLabels:
              app: product-server
          template:
            metadata:
              labels:
                app: product-server
            spec:
              containers:
                - name: product-server
                  image: xxxxxxx.dkr.ecr.ap-northeast-1.amazonaws.com/wa-demo-commerce-product-server:latest
                  imagePullPolicy: Always
                  resources:
                    requests:
                      cpu: "1000m"
                      memory: "2Gi"
                    limits:
                      cpu: "1500m"
                      memory: "4Gi"
                  ports:
                    - containerPort: 5001
        "#;
        let patch_json = serde_yaml::from_str::<serde_json::Value>(patch_yaml).unwrap();
        println!("patch_json:\n {:#?}", patch_json);
        let api_version = patch_json.get("apiVersion").unwrap();
        let api_group = api_version.as_str().unwrap().split('/').next().unwrap();
        let kind = patch_json.get("kind").unwrap().as_str().unwrap();

        // https://github.com/kube-rs/kube/blob/main/examples/crd_derive_schema.rs
        let apigroup = discovery::group(&client, api_group).await.unwrap();
        let (ar, _caps) = apigroup.recommended_kind(kind).unwrap();
        let namespace = "wave-autoscale";
        let name = "product-server-dp";
        // Use the discovered kind in an Api, and Controller with the ApiResource as its DynamicType
        let api: Api<DynamicObject> = Api::namespaced_with(client, namespace, &ar);
        // let get_object = api.get(name).await.unwrap();
        // println!(" >> get_object :\n{:?}", get_object);
        
        let patch_json = serde_yaml::from_str::<serde_json::Value>(patch_yaml).unwrap();
        println!("patch_json:\n {:#?}", patch_json);

        let patch_params = PatchParams::apply("wave-autoscale");
        let patch_params = PatchParams::force(patch_params);

        let result = api
            .patch(name, &patch_params, &Patch::Apply(patch_json))
            .await;

        println!("result: {:#?}", result);
        assert!(result.is_ok());
    }
}