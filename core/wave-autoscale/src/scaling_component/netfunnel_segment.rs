use super::ScalingComponent;
use anyhow::{Ok, Result};
use async_trait::async_trait;

use data_layer::ScalingComponentDefinition;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

use reqwest::Client;

pub struct NetfunnelSegmentScalingComponent {
    definition: ScalingComponentDefinition,
}

impl NetfunnelSegmentScalingComponent {
    // Static variables
    pub const SCALING_KIND: &'static str = "netfunnel";

    // Functions
    pub fn new(definition: ScalingComponentDefinition) -> Self {
        NetfunnelSegmentScalingComponent { definition }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct NetfunnelResponse {
    #[serde(rename = "statusCode")]
    status_code: i32,

    #[serde(rename = "status")]
    status: String,

    #[serde(rename = "errorCode")]
    error_code: String,

    #[serde(rename = "message")]
    message: String,
}

#[async_trait]
impl ScalingComponent for NetfunnelSegmentScalingComponent {
    fn get_scaling_component_kind(&self) -> &str {
        &self.definition.component_kind
    }
    fn get_id(&self) -> &str {
        &self.definition.id
    }
    async fn apply(&self, params: HashMap<String, serde_json::Value>) -> Result<()> {
        let metadata: HashMap<String, serde_json::Value> = self.definition.metadata.clone();

        if let (
            Some(serde_json::Value::String(base_url)),
            Some(serde_json::Value::String(authorization)),
            Some(serde_json::Value::String(organization_id)),
            Some(serde_json::Value::String(tenant_id)),
            Some(serde_json::Value::String(user_key)),
            Some(serde_json::Value::String(project_id)),
            Some(serde_json::Value::String(segment_id)),
            max_inflow,
        ) = (
            metadata.get("base_url"),
            metadata.get("authorization"),
            metadata.get("organization_id"),
            metadata.get("tenant_id"),
            metadata.get("user_key"),
            metadata.get("project_id"),
            metadata.get("segment_id"),
            params
                .get("max_inflow")
                .and_then(serde_json::Value::as_i64)
                .map(|v| v as i32),
        ) {
            if let Some(max_inflow) = max_inflow {
                let url = format!(
                    "{}/v2/wave/project/{}/segment/{}",
                    base_url, project_id, segment_id
                );

                let client = Client::new();
                let result = client
                    .put(url)
                    .header("Authorization", authorization)
                    .header("organizationId", organization_id)
                    .header("tenantId", tenant_id)
                    .header("userKey", user_key)
                    .json(&json!({ "maxInflow": max_inflow }))
                    .send()
                    .await?;
                if result.status().is_success() {
                    return Ok(());
                } else {
                    let result = result.text().await?;
                    let response: NetfunnelResponse = serde_json::from_str(&result)?;
                    let json = json!({
                      "message": response.message,
                      "code": response.error_code,
                      "extras":  response.status
                    });
                    return Err(anyhow::anyhow!(json));
                }
            }
        } else {
            return Err(anyhow::anyhow!("Invalid metadata"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::NetfunnelSegmentScalingComponent;
    use crate::scaling_component::ScalingComponent;
    use data_layer::ScalingComponentDefinition;
    use std::collections::HashMap;

    #[tokio::test]
    #[ignore]
    async fn apply_max_inflow() {
        let metadata: HashMap<String, serde_json::Value> = vec![
            (
                String::from("base_url"),
                serde_json::json!("https://dev-api.surffy-dev.io"),
            ),
            (
                String::from("authorization"),
                serde_json::json!("authorization"),
            ),
            (
                String::from("organization_id"),
                serde_json::json!("organization_id"),
            ),
            (String::from("tenant_id"), serde_json::json!("tenant_id")),
            (String::from("user_key"), serde_json::json!("user_key")),
            (String::from("project_id"), serde_json::json!("project_id")),
            (String::from("segment_id"), serde_json::json!("segment_id")),
        ]
        .into_iter()
        .collect();
        let params: HashMap<String, serde_json::Value> =
            vec![(String::from("max_inflow"), serde_json::json!(20))]
                .into_iter()
                .collect();

        let scaling_definition = ScalingComponentDefinition {
            kind: data_layer::types::object_kind::ObjectKind::ScalingComponent,
            db_id: String::from("db_id"),
            id: String::from("scaling-id"),
            component_kind: String::from("netfunnel"),
            metadata,
            ..Default::default()
        };
        let netfunnel_segment_scaling_component: Result<(), anyhow::Error> =
            NetfunnelSegmentScalingComponent::new(scaling_definition)
                .apply(params)
                .await;
        assert!(netfunnel_segment_scaling_component.is_ok());
    }
}
