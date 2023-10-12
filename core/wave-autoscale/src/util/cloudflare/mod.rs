use reqwest::Client;
use std::error::Error;

/**
Cloudflare API client
This module provides a client for the Cloudflare API

Example:
```rust
let client = CloudflareClient::new("YOUR_API_TOKEN_HERE".to_string());
let payload = json!({
    "id": "RULE_ID",
    "action": "allow",
    "expression": "true",
});
let result = client
    .update_zone_rule("TEST_ZONE_ID", "TEST_RULESET_ID", "TEST_RULE_ID", payload)
    .await;
assert!(result.is_ok(), "Failed to update zone rule");
```
*/
pub struct CloudflareClient {
    api_token: String,
    client: Client,
}

impl CloudflareClient {
    // Constructor to create a new Cloudflare instance with a given api_token
    pub fn new(api_token: String) -> Self {
        Self {
            api_token,
            client: Client::new(),
        }
    }

    // Validate the payload has the required fields for a zone rule (action, expression, description)
    fn validate_rule_payload(payload: &serde_json::Value) -> Result<(), Box<dyn Error>> {
        if !payload["action"].is_string() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Missing action field",
            )));
        }
        if !payload["expression"].is_string() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Missing expression field",
            )));
        }
        if !payload["description"].is_string() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Missing description field",
            )));
        }
        Ok(())
    }

    // Function to update an existing rule in a zone ruleset
    pub async fn update_zone_rule(
        &self,
        zone_id: &str,
        ruleset_id: &str,
        rule_id: &str,
        payload: serde_json::Value,
    ) -> Result<(), Box<dyn Error>> {
        Self::validate_rule_payload(&payload)?;

        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/rulesets/{}/rules/{}",
            zone_id, ruleset_id, rule_id
        );

        let response = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to update zone rule",
            )));
        }

        Ok(())
    }

    // Function to update an existing rule in an account ruleset
    pub async fn update_account_rule(
        &self,
        account_id: &str,
        ruleset_id: &str,
        rule_id: &str,
        payload: serde_json::Value,
    ) -> Result<(), Box<dyn Error>> {
        Self::validate_rule_payload(&payload)?;

        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/rulesets/{}/rules/{}",
            account_id, ruleset_id, rule_id
        );

        let response = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to update account rule",
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::CloudflareClient;
    use serde_json::json;

    #[tokio::test]
    #[ignore]
    async fn test_update_zone_rule() {
        let api_token = "";
        let zone_id = "";
        let ruleset_id = "";
        let rule_id = "";

        let cloudflare = CloudflareClient::new(api_token.to_string());
        let payload = json!({
            "enabled": true,
            "action": "block",
            "expression": "(ip.geoip.country eq \"GB\" or ip.geoip.country eq \"FR\") and cf.threat_score > 10",
            "description": "Block requests from GB and FR with a threat score greater than 10"
        });
        let result = cloudflare
            .update_zone_rule(zone_id, ruleset_id, rule_id, payload)
            .await;
        assert!(result.is_ok(), "Failed to update zone rule");
    }

    #[tokio::test]
    #[ignore]
    async fn test_update_account_rule() {
        let api_token = "";
        let account_id = "";
        let ruleset_id = "";
        let rule_id = "";

        let cloudflare = CloudflareClient::new(api_token.to_string());
        let payload = json!({
            "enabled": true,
            "action": "block",
            "expression": "(ip.geoip.country eq \"GB\" or ip.geoip.country eq \"FR\") and cf.threat_score > 10",
            "description": "Block requests from GB and FR with a threat score greater than 10"
        });
        let result = cloudflare
            .update_account_rule(account_id, ruleset_id, rule_id, payload)
            .await;
        assert!(result.is_ok(), "Failed to update account rule");
    }
}
