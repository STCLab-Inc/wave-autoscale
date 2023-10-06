use super::ScalingComponent;
use crate::util::aws::get_aws_config_with_metadata;
use anyhow::{Ok, Result};
use async_trait::async_trait;
use aws_sdk_wafv2::Client as WAFClient;
use data_layer::ScalingComponentDefinition;
use serde_json::Value;
use std::collections::HashMap;

pub struct AWSWAFv2ScalingComponent {
    definition: ScalingComponentDefinition,
}

impl AWSWAFv2ScalingComponent {
    // Static variables
    pub const SCALING_KIND: &'static str = "aws-wafv2";

    // Functions
    pub fn new(definition: ScalingComponentDefinition) -> Self {
        AWSWAFv2ScalingComponent { definition }
    }
}

#[async_trait]
impl ScalingComponent for AWSWAFv2ScalingComponent {
    fn get_scaling_component_kind(&self) -> &str {
        &self.definition.component_kind
    }
    fn get_id(&self) -> &str {
        &self.definition.id
    }
    async fn apply(&self, params: HashMap<String, Value>) -> Result<()> {
        let metadata = &self.definition.metadata;
        let (
            Some(Value::String(web_acl_id)), 
            Some(Value::String(web_acl_name)), 
            Some(Value::String(scope)),
            Some(Value::String(rule_name)),
            Some(rate_limit)) = (
            metadata.get("web_acl_id"),
            metadata.get("web_acl_name"),
            metadata.get("scope"),
            params.get("rule_name"),
            params.get("rate_limit").and_then(Value::as_i64),
        ) else {
            return Err(anyhow::anyhow!("Invalid metadata"));
        };
        let config = get_aws_config_with_metadata(metadata).await;
        if config.is_err() {
            let config_err = config.err().unwrap();
            return Err(anyhow::anyhow!(config_err));
        }
        let config = config.unwrap();
        let client = WAFClient::new(&config);
        let scope = match scope.as_str().to_lowercase().as_str() {
            "cloudfront" => aws_sdk_wafv2::types::Scope::Cloudfront,
            "regional" => aws_sdk_wafv2::types::Scope::Regional,
            _ => return Err(anyhow::anyhow!("Invalid scope")),
        };
        let web_acl = client
            .get_web_acl()
            .id(web_acl_id)
            .name(web_acl_name)
            .scope(scope.clone())
            .send()
            .await;
        if web_acl.is_err() {
            let web_acl_err = web_acl.err().unwrap();
            return Err(anyhow::anyhow!(web_acl_err));
        }
        let web_acl = web_acl.unwrap();
        let lock_token = web_acl.lock_token.clone().unwrap();
        let web_acl = web_acl.web_acl();
        if web_acl.is_none() {
            return Err(anyhow::anyhow!("Web ACL is none"));
        }
        let web_acl = web_acl.unwrap();
        let default_action = web_acl.default_action.clone().unwrap();
        let visiblility_config = web_acl.visibility_config.clone().unwrap();

        let rules = web_acl
            .rules()
            .unwrap()
            .iter()
            .map(|rule| {
                let mut rule = rule.clone();
                if let Some(this_rule_name) = &rule.name {
                    if this_rule_name == rule_name {
                        if let Some(statement) = &mut rule.statement {
                            if let Some(rate_based_statement) = &mut statement.rate_based_statement {
                                rate_based_statement.limit = rate_limit;
                            }
                        }
                    }
                }
                rule
            })
            .collect::<Vec<_>>();

        let result = client
            .update_web_acl()
            .id(web_acl_id)
            .name(web_acl_name)
            .scope(scope)
            .set_rules(Some(rules))
            .visibility_config(visiblility_config)
            .default_action(default_action)
            .lock_token(lock_token)
            .send()
            .await;
        
        if result.is_err() {
            let result_err = result.err().unwrap();
            return Err(anyhow::anyhow!(result_err));
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::AWSWAFv2ScalingComponent;
    use crate::scaling_component::ScalingComponent;
    use data_layer::ScalingComponentDefinition;
    use std::collections::HashMap;

    // Purpose of the test is call apply function and fail test. just consists of test forms only.
    #[tokio::test]
    async fn apply_test() {
        let scaling_definition = ScalingComponentDefinition {
            kind: data_layer::types::object_kind::ObjectKind::ScalingComponent,
            db_id: String::from("db_id"),
            id: String::from("scaling-id"),
            component_kind: String::from("aws-wafv2"),
            metadata: HashMap::from([
                (
                    "region".to_string(),
                    serde_json::Value::String("ap-northeast-1".to_string()),
                ),
                (
                    "web_acl_id".to_string(),
                    serde_json::Value::String("4eb74b24-2c47-426c-96bf-608a103f710f".to_string()),
                ),
                (
                    "web_acl_name".to_string(),
                    serde_json::Value::String("wa-acl-test".to_string()),
                ),
                (
                    "scope".to_string(),
                    serde_json::Value::String("regional".to_string()),
                ),
            ]),
        };

        let params = HashMap::from([
            (
                "rule_name".to_string(),
                serde_json::Value::String("rate-limit-count-all".to_string()),
            ),
            (
                "rate_limit".to_string(),
                serde_json::Value::Number(serde_json::Number::from(100)),
            ),
        ]);
        let lambda_function_scaling_component = AWSWAFv2ScalingComponent::new(scaling_definition)
            .apply(params)
            .await;
        println!("{:?}", lambda_function_scaling_component);
        assert!(lambda_function_scaling_component.is_ok());
    }
}
