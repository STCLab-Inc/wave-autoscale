use super::ScalingComponent;
use crate::util::cloudflare::CloudflareClient;
use anyhow::{Ok, Result};
use async_trait::async_trait;
use data_layer::ScalingComponentDefinition;
use serde_json::Value;
use std::collections::HashMap;

pub struct CloudflareRuleScalingComponent {
    definition: ScalingComponentDefinition,
}

impl CloudflareRuleScalingComponent {
    // Static variables
    pub const SCALING_KIND: &'static str = "cloudflare-rule";

    // Functions
    pub fn new(definition: ScalingComponentDefinition) -> Self {
        CloudflareRuleScalingComponent { definition }
    }
}

#[async_trait]
impl ScalingComponent for CloudflareRuleScalingComponent {
    fn get_scaling_component_kind(&self) -> &str {
        &self.definition.component_kind
    }
    fn get_id(&self) -> &str {
        &self.definition.id
    }
    async fn apply(&self, params: HashMap<String, Value>, _context: rquickjs::AsyncContext) -> Result<HashMap<String, Value>> {
        let metadata = &self.definition.metadata;
        let (
            Some(Value::String(api_token)), 
            Some(Value::String(level)), 
            Some(Value::String(ruleset_id)),
            Some(Value::String(rule_id)),
            Some(rule)) = (
            metadata.get("api_token"),
            params.get("level"),
            params.get("ruleset_id"),
            params.get("rule_id"),
            params.get("rule"),
        ) else {
            return Err(anyhow::anyhow!("Invalid metadata"));
        };

        if level == "zone" {
            let Some(Value::String(zone_id)) = params.get("zone_id") else {
                return Err(anyhow::anyhow!("Invalid zone_id"));
            };
            let client = CloudflareClient::new(api_token.clone());
            let result = client
                .update_zone_rule(
                    zone_id,
                    ruleset_id,
                    rule_id,
                    rule.clone()
                )
                .await;
            if result.is_err() {
                let result_err = result.err().unwrap().to_string();
                return Err(anyhow::anyhow!(result_err));
            }
        } else if level == "account" {
            let Some(Value::String(account_id)) = params.get("account_id") else {
                return Err(anyhow::anyhow!("Invalid account_id"));
            };
            let client = CloudflareClient::new(api_token.clone());
            let result = client
                .update_account_rule(
                    account_id,
                    ruleset_id,
                    rule_id,
                    rule.clone()
                )
                .await;
            if result.is_err() {
                let result_err = result.err().unwrap().to_string();
                return Err(anyhow::anyhow!(result_err));
            }

        } else {
            return Err(anyhow::anyhow!("Invalid level"));
        }
        Ok(params)
    }
}

#[cfg(test)]
mod test {
    use super::CloudflareRuleScalingComponent;
    use crate::scaling_component::ScalingComponent;
    use data_layer::ScalingComponentDefinition;
    use std::collections::HashMap;
    use crate::scaling_component::test::get_rquickjs_context;

    // Purpose of the test is call apply function and fail test. just consists of test forms only.
    #[tokio::test]
    #[ignore]
    async fn test_apply_zone() {
        let scaling_definition = ScalingComponentDefinition {
            kind: data_layer::types::object_kind::ObjectKind::ScalingComponent,
            db_id: String::from("db_id"),
            id: String::from("scaling-id"),
            component_kind: String::from("cloudflare-rule"),
            metadata: HashMap::from([
                (
                    "api_token".to_string(),
                    serde_json::Value::String("".to_string())
                )
            ]),
            ..Default::default()
        };

        let params = HashMap::from([
            (
                "level".to_string(),
                serde_json::Value::String("zone".to_string()),
            ),
            (
                "zone_id".to_string(),
                serde_json::Value::String("".to_string()),
            ),
            (
                "ruleset_id".to_string(),
                serde_json::Value::String("".to_string()),
            ),
            (
                "rule_id".to_string(),
                serde_json::Value::String("".to_string()),
            ),
            (
                "rule".to_string(),
                serde_json::json!({
                    "enabled": true,
                    "action": "block",
                    "expression": "(ip.geoip.country eq \"GB\" or ip.geoip.country eq \"FR\") and cf.threat_score > 10",
                    "description": "Block requests from GB and FR with a threat score greater than 10"
                }),
            ),
        ]);
        let lambda_function_scaling_component = CloudflareRuleScalingComponent::new(scaling_definition)
            .apply(params, get_rquickjs_context().await)
            .await;
        assert!(lambda_function_scaling_component.is_ok());
    }
    #[tokio::test]
    #[ignore]
    async fn test_apply_account() {
        let scaling_definition = ScalingComponentDefinition {
            kind: data_layer::types::object_kind::ObjectKind::ScalingComponent,
            db_id: String::from("db_id"),
            id: String::from("scaling-id"),
            component_kind: String::from("cloudflare-rule"),
            metadata: HashMap::from([
                (
                    "api_token".to_string(),
                    serde_json::Value::String("".to_string())
                )
            ]),
            ..Default::default()
        };

        let params = HashMap::from([
            (
                "level".to_string(),
                serde_json::Value::String("account".to_string()),
            ),
            (
                "account_id".to_string(),
                serde_json::Value::String("".to_string()),
            ),
            (
                "ruleset_id".to_string(),
                serde_json::Value::String("".to_string()),
            ),
            (
                "rule_id".to_string(),
                serde_json::Value::String("".to_string()),
            ),
            (
                "rule".to_string(),
                serde_json::json!({
                    "enabled": true,
                    "action": "block",
                    "expression": "(ip.geoip.country eq \"GB\" or ip.geoip.country eq \"FR\") and cf.threat_score > 10",
                    "description": "Block requests from GB and FR with a threat score greater than 10"
                }),
            ),
        ]);
        let lambda_function_scaling_component = CloudflareRuleScalingComponent::new(scaling_definition)
            .apply(params, get_rquickjs_context().await)
            .await;
        assert!(lambda_function_scaling_component.is_ok());
    }
    #[tokio::test]
    #[ignore]
    async fn test_apply_invalid_params() {
        let scaling_definition = ScalingComponentDefinition {
            kind: data_layer::types::object_kind::ObjectKind::ScalingComponent,
            db_id: String::from("db_id"),
            id: String::from("scaling-id"),
            component_kind: String::from("cloudflare-rule"),
            metadata: HashMap::from([
                (
                    "api_token".to_string(),
                    serde_json::Value::String("".to_string())
                )
            ]),
            ..Default::default()
        };

        let params = HashMap::from([
            (
                "level".to_string(),
                serde_json::Value::String("invalid_level".to_string()),
            ),
            (
                "ruleset_id".to_string(),
                serde_json::Value::String("".to_string()),
            ),
            (
                "rule_id".to_string(),
                serde_json::Value::String("".to_string()),
            ),
            (
                "rule".to_string(),
                serde_json::json!({
                    "enabled": true,
                    "action": "block",
                    "expression": "(ip.geoip.country eq \"GB\" or ip.geoip.country eq \"FR\") and cf.threat_score > 10",
                    "description": "Block requests from GB and FR with a threat score greater than 10"
                }),
            ),
        ]);
        let lambda_function_scaling_component = CloudflareRuleScalingComponent::new(scaling_definition)
            .apply(params, get_rquickjs_context().await)
            .await;
        assert!(lambda_function_scaling_component.is_err());
    }
}
