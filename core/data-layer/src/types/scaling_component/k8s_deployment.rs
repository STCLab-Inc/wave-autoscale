use ts_rs::TS;

#[derive(TS)]
#[ts(export, export_to = "../web-app/src/types/bindings/k8s-deployment.ts")]
pub struct K8sDeploymentMetadata {
    pub api_server_endpoint: String,
    pub namespace: String,
    pub name: String,
}

#[derive(TS)]
#[ts(
    export,
    export_to = "../web-app/src/types/bindings/k8s-deployment-plan.ts"
)]
pub struct K8sDeploymentPlanMetadata {
    pub replicas: u16,
}
