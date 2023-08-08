use crate::util::google_cloud::google_cloud_run_service_helper::call_patch_cloud_run_service;

use super::super::util::google_cloud::google_cloud_run_service_helper::{
    call_get_cloud_run_service, CloudRunGetServiceSetting, CloudRunPatchServiceSetting,
};
use super::ScalingComponent;
use anyhow::{Ok, Result};
use async_trait::async_trait;
use data_layer::ScalingComponentDefinition;

use std::collections::HashMap;

pub struct CloudRunServiceScalingComponent {
    definition: ScalingComponentDefinition,
}

impl CloudRunServiceScalingComponent {
    // Static variables
    pub const SCALING_KIND: &'static str = "google-cloud-run";

    // Functions
    pub fn new(definition: ScalingComponentDefinition) -> Self {
        CloudRunServiceScalingComponent { definition }
    }
}

#[async_trait]
impl ScalingComponent for CloudRunServiceScalingComponent {
    fn get_scaling_component_kind(&self) -> &str {
        &self.definition.component_kind
    }

    fn get_id(&self) -> &str {
        &self.definition.id
    }

    async fn apply(&self, params: HashMap<String, serde_json::Value>) -> Result<()> {
        let metadata: HashMap<String, serde_json::Value> = self.definition.metadata.clone();

        if let (
            Some(serde_json::Value::String(api_version)),
            Some(serde_json::Value::String(project_name)),
            Some(serde_json::Value::String(location_name)),
            Some(serde_json::Value::String(service_name)),
            min_instance_count,
            max_instance_count,
            max_request_per_instance,
            execution_environment,
        ) = (
            metadata.get("api_version"),
            metadata.get("project_name"),
            metadata.get("location_name"),
            metadata.get("service_name"),
            params
                .get("min_instance_count")
                .and_then(serde_json::Value::as_i64)
                .map(|v| v as i32),
            params
                .get("max_instance_count")
                .and_then(serde_json::Value::as_i64)
                .map(|v| v as i32),
            params
                .get("max_request_per_instance")
                .and_then(serde_json::Value::as_i64)
                .map(|v| v as i32),
            params
                .get("execution_environment")
                .and_then(serde_json::Value::as_str)
                .map(|s| s as &str),
        ) {
            fn extract_container_image_from_first_version(
                json_str: &str,
                service_name: &str,
            ) -> Option<String> {
                let parsed: serde_json::Value =
                    serde_json::from_str(json_str).expect("Failed to parse JSON");

                if parsed["metadata"]["name"].as_str() == Some(service_name) {
                    return parsed["spec"]["template"]["spec"]["containers"][0]["image"]
                        .as_str()
                        .map(|s| s.to_string());
                }

                None
            }
            fn extract_container_image_from_second_version(
                json_str: &str,
                service_name: &str,
            ) -> Option<String> {
                let parsed: serde_json::Value =
                    serde_json::from_str(json_str).expect("Failed to parse JSON");

                if parsed["name"].as_str() == Some(service_name) {
                    return parsed["template"]["containers"][0]["image"]
                        .as_str()
                        .map(|s| s.to_string());
                }

                None
            }
            fn extract_container_image(
                metadata: &HashMap<String, serde_json::Value>,
                json_str: &str,
            ) -> Option<String> {
                match metadata.get("api_version").unwrap().as_str().unwrap_or("") {
                    "v1" => extract_container_image_from_first_version(
                        json_str,
                        metadata.get("service_name").unwrap().as_str().unwrap_or(""),
                    ),
                    _ => {
                        let formatted_service_name = format!(
                            "projects/{}/locations/{}/services/{}",
                            metadata.get("project_name").unwrap().as_str().unwrap_or(""),
                            metadata
                                .get("location_name")
                                .unwrap()
                                .as_str()
                                .unwrap_or(""),
                            metadata.get("service_name").unwrap().as_str().unwrap_or("")
                        );
                        extract_container_image_from_second_version(
                            json_str,
                            &formatted_service_name,
                        )
                    }
                }
            }

            let cloud_run_get_service_setting = CloudRunGetServiceSetting {
                api_version: api_version.to_string(),
                project_name: project_name.to_string(),
                location_name: location_name.to_string(),
                service_name: service_name.to_string(),
            };
            let result = call_get_cloud_run_service(cloud_run_get_service_setting).await;
            if result.is_err() {
                return Err(anyhow::anyhow!(serde_json::json!({
                    "message": "API call error",
                    "code": "500",
                    "extras": result.unwrap_err().is_body().to_string()
                })));
            }
            let result = result.unwrap();
            let result_status_code = result.status();
            let core::result::Result::Ok(result_body) = result.text().await else {
                        return Err(anyhow::anyhow!(serde_json::json!({
                            "message": "API call error",
                            "code": "500",
                            "extras": "Not found response text",
                        })));
                    };
            if !result_status_code.is_success() {
                log::error!("API call error: {:?}", result_body);
                let json = serde_json::json!({
                    "message": "API call error",
                    "code": result_status_code.as_str(),
                    "extras": result_body
                });
                return Err(anyhow::anyhow!(json));
            }
            let container_image = extract_container_image(&metadata, &result_body);

            fn create_payload_from_first_version(
                service_name: &str,
                project_name: &str,
                min_instance_count: Option<&str>,
                max_instance_count: Option<&str>,
                max_request_per_instance: Option<&str>,
                execution_environment: Option<&str>,
                container_image: &str,
            ) -> serde_json::Value {
                // Annotations
                let mut annotations = serde_json::Map::new();

                if let Some(min_count) = min_instance_count {
                    annotations.insert(
                        "autoscaling.knative.dev/minScale".to_string(),
                        serde_json::json!(min_count),
                    );
                }

                if let Some(max_count) = max_instance_count {
                    annotations.insert(
                        "autoscaling.knative.dev/maxScale".to_string(),
                        serde_json::json!(max_count),
                    );
                }

                if let Some(environment) = execution_environment {
                    // Allowed values are gen1 and gen2
                    let environment = match environment {
                        "EXECUTION_ENVIRONMENT_UNSPECIFIED" => "gen1",
                        "EXECUTION_ENVIRONMENT_GEN1" => "gen1",
                        "EXECUTION_ENVIRONMENT_GEN2" => "gen2",
                        _ => "gen1",
                    };
                    annotations.insert(
                        "run.googleapis.com/execution-environment".to_string(),
                        serde_json::json!(environment),
                    );
                }

                // Spec
                let mut spec = serde_json::Map::new();

                if let Some(concurrency) = max_request_per_instance {
                    spec.insert(
                        "containerConcurrency".to_string(),
                        serde_json::json!(concurrency),
                    );
                }

                spec.insert(
                    "containers".to_string(),
                    serde_json::json!([{ "image": container_image }]),
                );

                // Only include the metadata section if there are annotations
                let template = if annotations.is_empty() {
                    serde_json::json!({ "spec": spec })
                } else {
                    serde_json::json!({
                        "metadata": {
                            "annotations": annotations
                        },
                        "spec": spec
                    })
                };

                serde_json::json!({
                    "apiVersion": "serving.knative.dev/v1",
                    "kind": "Service",
                    "metadata": {
                        "name": service_name,
                        "namespace": project_name,
                    },
                    "spec": {
                        "template": template
                    }
                })
            }
            fn create_payload_from_second_version(
                min_instance_count: Option<&str>,
                max_instance_count: Option<&str>,
                max_request_per_instance: Option<&str>,
                execution_environment: Option<&str>,
                container_image: &str,
            ) -> serde_json::Value {
                let mut template = serde_json::Map::new();

                // Add maxInstanceRequestConcurrency to template if it's available
                if let Some(max_request) = max_request_per_instance {
                    template.insert(
                        "maxInstanceRequestConcurrency".to_string(),
                        serde_json::Value::String(max_request.to_string()),
                    );
                }

                // Scaling block
                let mut scaling = serde_json::Map::new();
                if let Some(min_count) = min_instance_count {
                    scaling.insert(
                        "minInstanceCount".to_string(),
                        serde_json::Value::String(min_count.to_string()),
                    );
                }
                if let Some(max_count) = max_instance_count {
                    scaling.insert(
                        "maxInstanceCount".to_string(),
                        serde_json::Value::String(max_count.to_string()),
                    );
                }
                if !scaling.is_empty() {
                    template.insert("scaling".to_string(), serde_json::Value::Object(scaling));
                }

                // Containers block
                template.insert(
                    "containers".to_string(),
                    serde_json::json!([{ "image": container_image }]),
                );

                // Add executionEnvironment to template if it's available
                if let Some(env) = execution_environment {
                    template.insert(
                        "executionEnvironment".to_string(),
                        serde_json::Value::String(env.to_string()),
                    );
                }

                // Return the overall structure
                serde_json::Value::Object({
                    let mut obj = serde_json::Map::new();
                    obj.insert("template".to_string(), serde_json::Value::Object(template));
                    obj
                })
            }
            fn create_payload(
                metadata: &HashMap<String, serde_json::Value>,
                min_instance_count: Option<&str>,
                max_instance_count: Option<&str>,
                max_request_per_instance: Option<&str>,
                execution_environment: Option<&str>,
                container_image: &str,
            ) -> serde_json::Value {
                match metadata.get("api_version").unwrap().as_str().unwrap_or("") {
                    "v1" => create_payload_from_first_version(
                        metadata.get("service_name").unwrap().as_str().unwrap_or(""),
                        metadata.get("project_name").unwrap().as_str().unwrap_or(""),
                        min_instance_count,
                        max_instance_count,
                        max_request_per_instance,
                        execution_environment,
                        container_image,
                    ),
                    _ => create_payload_from_second_version(
                        min_instance_count,
                        max_instance_count,
                        max_request_per_instance,
                        execution_environment,
                        container_image,
                    ),
                }
            }

            let cloud_run_patch_service_setting = CloudRunPatchServiceSetting {
                api_version: api_version.to_string(),
                project_name: project_name.to_string(),
                location_name: location_name.to_string(),
                service_name: service_name.to_string(),
                payload: Some(create_payload(
                    &metadata,
                    min_instance_count
                        .map(|value| value.to_string())
                        .as_deref()
                        .or(None),
                    max_instance_count
                        .map(|value| value.to_string())
                        .as_deref()
                        .or(None),
                    max_request_per_instance
                        .map(|value| value.to_string())
                        .as_deref()
                        .or(None),
                    execution_environment,
                    &container_image.unwrap_or("".to_string()),
                )),
            };
            let result = call_patch_cloud_run_service(cloud_run_patch_service_setting).await;
            if result.is_err() {
                return Err(anyhow::anyhow!(serde_json::json!({
                    "message": "API call error",
                    "code": "500",
                    "extras": result.unwrap_err().is_body().to_string()
                })));
            }
            let result = result.unwrap();
            let result_status_code = result.status();
            let core::result::Result::Ok(result_body) = result.text().await else {
                    return Err(anyhow::anyhow!(serde_json::json!({
                        "message": "API call error",
                        "code": "500",
                        "extras": "Not found response text",
                    })));
                };
            if !result_status_code.is_success() {
                log::error!("API call error: {:?}", result_body);
                let json = serde_json::json!({
                    "message": "API call error",
                    "code": result_status_code.as_str(),
                    "extras": result_body
                });
                return Err(anyhow::anyhow!(json));
            }

            Ok(())
        } else {
            Err(anyhow::anyhow!("Invalid metadata"))
        }
    }
}

#[cfg(test)]
mod test {
    use super::CloudRunServiceScalingComponent;
    use crate::scaling_component::ScalingComponent;
    use data_layer::ScalingComponentDefinition;
    use std::collections::HashMap;

    #[ignore]
    #[tokio::test]
    async fn apply_call_get_first_version_cloud_run_service() {
        let metadata: HashMap<String, serde_json::Value> = vec![
            (String::from("api_version"), serde_json::json!("v1")),
            (
                String::from("location_name"),
                serde_json::json!("asia-northeast2"),
            ),
            (
                String::from("project_name"),
                serde_json::json!("wave-autoscale-test"),
            ),
            (String::from("service_name"), serde_json::json!("service-1")),
        ]
        .into_iter()
        .collect();
        let params: HashMap<String, serde_json::Value> = vec![].into_iter().collect();
        let scaling_definition = ScalingComponentDefinition {
            kind: data_layer::types::object_kind::ObjectKind::ScalingComponent,
            db_id: String::from("db-id"),
            id: String::from("scaling-id"),
            component_kind: String::from("google-cloud-run"),
            metadata,
        };
        let cloud_run_service_scaling_component: Result<(), anyhow::Error> =
            CloudRunServiceScalingComponent::new(scaling_definition)
                .apply(params)
                .await;

        println!(
            "cloud_run_service_scaling_component: {:?}",
            cloud_run_service_scaling_component
        );
        assert!(cloud_run_service_scaling_component.is_ok());
    }

    #[ignore]
    #[tokio::test]
    async fn apply_call_get_second_version_cloud_run_service() {
        let metadata: HashMap<String, serde_json::Value> = vec![
            (String::from("api_version"), serde_json::json!("v2")),
            (
                String::from("project_name"),
                serde_json::json!("wave-autoscale-test"),
            ),
            (
                String::from("location_name"),
                serde_json::json!("asia-northeast2"),
            ),
            (String::from("service_name"), serde_json::json!("service-1")),
        ]
        .into_iter()
        .collect();
        let params: HashMap<String, serde_json::Value> = vec![].into_iter().collect();
        let scaling_definition = ScalingComponentDefinition {
            kind: data_layer::types::object_kind::ObjectKind::ScalingComponent,
            db_id: String::from("db-id"),
            id: String::from("scaling-id"),
            component_kind: String::from("google-cloud-run"),
            metadata,
        };
        let cloud_run_service_scaling_component: Result<(), anyhow::Error> =
            CloudRunServiceScalingComponent::new(scaling_definition)
                .apply(params)
                .await;

        println!(
            "cloud_run_service_scaling_component: {:?}",
            cloud_run_service_scaling_component
        );
        assert!(cloud_run_service_scaling_component.is_ok());
    }

    #[ignore]
    #[tokio::test]
    async fn apply_call_patch_first_version_cloud_run_service() {
        let metadata: HashMap<String, serde_json::Value> = vec![
            (String::from("api_version"), serde_json::json!("v1")),
            (
                String::from("location_name"),
                serde_json::json!("asia-northeast2"),
            ),
            (
                String::from("project_name"),
                serde_json::json!("wave-autoscale-test"),
            ),
            (String::from("service_name"), serde_json::json!("service-1")),
        ]
        .into_iter()
        .collect();
        let params: HashMap<String, serde_json::Value> = vec![
            (String::from("min_instance_count"), serde_json::json!(5)),
            (String::from("max_instance_count"), serde_json::json!(8)),
            (
                String::from("max_request_per_instance"),
                serde_json::json!(3),
            ),
            (
                String::from("execution_environment"),
                serde_json::json!("EXECUTION_ENVIRONMENT_UNSPECIFIED"),
            ),
        ]
        .into_iter()
        .collect();
        let scaling_definition = ScalingComponentDefinition {
            kind: data_layer::types::object_kind::ObjectKind::ScalingComponent,
            db_id: String::from("db-id"),
            id: String::from("scaling-id"),
            component_kind: String::from("google-cloud-run"),
            metadata,
        };
        let cloud_run_service_scaling_component: Result<(), anyhow::Error> =
            CloudRunServiceScalingComponent::new(scaling_definition)
                .apply(params)
                .await;

        println!(
            "cloud_run_service_scaling_component: {:?}",
            cloud_run_service_scaling_component
        );
        assert!(cloud_run_service_scaling_component.is_ok());
    }

    #[ignore]
    #[tokio::test]
    async fn apply_call_patch_second_version_cloud_run_service() {
        let metadata: HashMap<String, serde_json::Value> = vec![
            (String::from("api_version"), serde_json::json!("v2")),
            (
                String::from("project_name"),
                serde_json::json!("wave-autoscale-test"),
            ),
            (
                String::from("location_name"),
                serde_json::json!("asia-northeast2"),
            ),
            (String::from("service_name"), serde_json::json!("service-1")),
        ]
        .into_iter()
        .collect();
        let params: HashMap<String, serde_json::Value> = vec![
            (String::from("min_instance_count"), serde_json::json!(2)),
            (String::from("max_instance_count"), serde_json::json!(4)),
            (
                String::from("max_request_per_instance"),
                serde_json::json!(6),
            ),
            (
                String::from("execution_environment"),
                serde_json::json!("EXECUTION_ENVIRONMENT_UNSPECIFIED"),
            ),
        ]
        .into_iter()
        .collect();
        let scaling_definition = ScalingComponentDefinition {
            kind: data_layer::types::object_kind::ObjectKind::ScalingComponent,
            db_id: String::from("db-id"),
            id: String::from("scaling-id"),
            component_kind: String::from("google-cloud-run"),
            metadata,
        };
        let cloud_run_service_scaling_component: Result<(), anyhow::Error> =
            CloudRunServiceScalingComponent::new(scaling_definition)
                .apply(params)
                .await;

        println!(
            "cloud_run_service_scaling_component: {:?}",
            cloud_run_service_scaling_component
        );
        assert!(cloud_run_service_scaling_component.is_ok());
    }
}
