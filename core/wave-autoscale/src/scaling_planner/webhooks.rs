use reqwest::header::HeaderMap;
use reqwest::Client;
use std::collections::HashMap;
use tracing::{debug, error};
use utils::wave_config::WebhookType;
use utils::wave_config::Webhooks;

#[derive(Clone)]
pub struct WebhookResponse {
    pub plan_id: String,
    pub plan_item_id: String,
    pub scaling_component_json_str: String,
    pub fail_message: Option<String>,
}
impl WebhookResponse {
    pub fn response_for_http(&self) -> Option<serde_json::Value> {
        let mut scaling_component = serde_json::json!("");
        if !self.scaling_component_json_str.is_empty() {
            let Ok(scaling_component_json) = serde_json::from_str::<serde_json::Value>(self.scaling_component_json_str.as_str()) else {
                error!(
                    "[Webhook] Failed to send webhook: Failed to parse scaling_component_json_str (json)"
                );
                return None;
            };
            scaling_component = scaling_component_json;
        }
        Some(serde_json::json!({
            "timestamp": chrono::Utc::now().timestamp(),
            "plan_id": self.plan_id,
            "plan_item_id": self.plan_item_id,
            "scaling_component": scaling_component,
            "status": if self.fail_message.is_some() { WebhookResponseStatus::Fail.to_string() } else { WebhookResponseStatus::Success.to_string() },
            "fail_message": if self.fail_message.is_some() { self.fail_message.clone().unwrap() } else { "".to_string() },
        }))
    }

    pub fn response_for_slack(&self) -> Option<serde_json::Value> {
        let slack_date_time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let mut scaling_component = "-\n".to_string();
        if !self.scaling_component_json_str.is_empty() {
            let Ok(scaling_component_json) = serde_json::from_str::<serde_json::Value>(self.scaling_component_json_str.as_str()) else {
                error!(
                    "[Webhook] Failed to send webhook: Failed to parse scaling_component_json_str (json)"
                );
                return None;
            };
            let Ok(scaling_component_yaml) = serde_yaml::to_string(&scaling_component_json) else {
                error!(
                    "[Webhook] Failed to send webhook: Failed to parse scaling_component_json_str (yaml)"
                );
                return None;
            };
            scaling_component = scaling_component_yaml;
        }

        Some(serde_json::json!({
            "blocks": [
                {
                    "type": "section",
                    "text": {
                        "type": "mrkdwn",
                        "text": if self.fail_message.is_some() { format!(":X: *FAIL*\nFail Message: *{}*", self.fail_message.clone().unwrap()) } else { ":white_check_mark: *SUCCESS*".to_string() }
                    }
                },
                {
                    "type": "context",
                    "elements": [
                        {
                            "text": format!("*{}*  |  Wave Autoscale - Scaling Plan History", slack_date_time),
                            "type": "mrkdwn"
                        }
                    ]
                },
                {
                    "type": "divider"
                },
                {
                    "type": "section",
                    "text": {
                        "type": "mrkdwn",
                        "text": format!("Plan ID: *{}*\nPlan Item ID: *{}*\nScaling Component:\n ```{}```", self.plan_id, self.plan_item_id, scaling_component)
                    }
                }
            ]
        }))
    }
}

pub enum WebhookResponseStatus {
    Success,
    Fail,
}
impl std::fmt::Display for WebhookResponseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            WebhookResponseStatus::Success => write!(f, "SUCCESS"),
            WebhookResponseStatus::Fail => write!(f, "FAIL"),
        }
    }
}

pub fn send_webhooks(
    webhooks: Option<Vec<Webhooks>>,
    plan_webhooks: Option<Vec<String>>,
    webhook_response: WebhookResponse,
) {
    tokio::spawn(async move {
        if let (Some(plan_webhooks), Some(webhooks)) = (plan_webhooks, webhooks) {
            let plan_webhooks_map: HashMap<String, String> = plan_webhooks
                .iter()
                .map(|webhook_id| (webhook_id.clone(), "".to_string()))
                .collect();
            for webhook in webhooks {
                if !(plan_webhooks_map.contains_key(webhook.id.as_str())) {
                    return;
                }
                match webhook.webhook_type {
                    WebhookType::Http => {
                        let _ = send_webhook_http(webhook, webhook_response.clone()).await;
                    }
                    WebhookType::SlackIncomingWebhook => {
                        let _ =
                            send_webhook_slack_incoming_webhook(webhook, webhook_response.clone())
                                .await;
                    }
                }
            }
        }
    });
}

async fn send_webhook_http(
    webhook: Webhooks,
    webhook_response: WebhookResponse,
) -> Result<(), anyhow::Error> {
    let Some(url) = webhook.url else {
        error!("[Webhook] Failed to send webhook: url is not set");
        return Err(anyhow::anyhow!("[Webhook] Failed to send webhook: url is not set"));
    };
    let client = Client::new();
    let mut headers = HeaderMap::new();
    if let Some(headers_map) = webhook.headers {
        for (key, value) in headers_map {
            headers.insert(
                reqwest::header::HeaderName::from_bytes(key.as_bytes()).unwrap(),
                reqwest::header::HeaderValue::from_str(value.as_str()).unwrap(),
            );
        }
    }
    let Some(webhook_response_to_json) = webhook_response.response_for_http() else {
        error!("[Webhook] Failed to send webhook: Failed to parse response_for_http");
        return Err(anyhow::anyhow!("[Webhook] Failed to send webhook: Failed to parse response_for_http"));
    };
    let response = client
        .post(&url)
        .headers(headers)
        .json(&webhook_response_to_json)
        .send()
        .await;
    if let Err(e) = response {
        error!("[Webhook] Failed to send webhook HTTP: {}", e);
        return Err(anyhow::anyhow!("[Webhook] Failed to send webhook HTTP"));
    }
    Ok(())
}

async fn send_webhook_slack_incoming_webhook(
    webhook: Webhooks,
    webhook_response: WebhookResponse,
) -> Result<(), anyhow::Error> {
    let Some(url) = webhook.webhook_url else {
        error!("[Webhook] Failed to send webhook: webhook_url is not set");
        return Err(anyhow::anyhow!("[Webhook] Failed to send webhook: webhook_url is not set"));
    };
    let client = Client::new();
    let Some(webhook_response_for_slack) = webhook_response.response_for_slack() else {
        error!("[Webhook] Failed to send webhook: Failed to parse response_for_slack");
        return Err(anyhow::anyhow!("[Webhook] Failed to send webhook: Failed to parse response_for_slack"));
    };
    let response = client
        .post(&url)
        .json(&webhook_response_for_slack)
        .send()
        .await;
    if let Err(e) = response {
        error!(
            "[Webhook] Failed to send webhook Slack incoming webhook: {}",
            e
        );
        return Err(anyhow::anyhow!(
            "[Webhook] Failed to send webhook Slack incoming webhook"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_json_str_to_yaml() {
        let json_str = r#"{"component_id":"k8s_node_dp","replicas":"1"}"#;
        let scaling_component_json = serde_json::from_str::<serde_json::Value>(json_str).unwrap();
        let scaling_component_yaml = serde_yaml::to_string(&scaling_component_json).unwrap();
        assert_eq!(
            scaling_component_yaml,
            "component_id: k8s_node_dp\nreplicas: '1'\n"
        );
    }

    #[ignore]
    #[tokio::test]
    async fn test_send_webhook_http() {
        let webhooks = Webhooks {
            id: "test".to_string(),
            webhook_type: WebhookType::Http,
            url: Some("http://localhost:3024/api/test".to_string()),
            headers: None,
            webhook_url: None,
        };
        let webhook_response = WebhookResponse {
            plan_id: "test-plan-1".to_string(),
            plan_item_id: "test-plan-item-1".to_string(),
            scaling_component_json_str: r#"{"component_id":"k8s_node_dp","replicas":"1"}"#
                .to_string(),
            fail_message: None,
        };
        let send_webhook_http = send_webhook_http(webhooks, webhook_response).await;
        assert!(send_webhook_http.is_ok());
    }

    #[ignore]
    #[tokio::test]
    async fn test_send_webhook_slack_incoming_webhook() {
        let webhooks = Webhooks {
            id: "test".to_string(),
            webhook_type: WebhookType::SlackIncomingWebhook,
            url: None,
            headers: None,
            webhook_url: Some(
                "https://hooks.slack.com/services/T00000000/B00000000/XXXXXXXXXXXXXXXXXXXXXXXX"
                    .to_string(),
            ),
        };
        let webhook_response = WebhookResponse {
            plan_id: "test-plan-1".to_string(),
            plan_item_id: "test-plan-item-1".to_string(),
            scaling_component_json_str: r#""#.to_string(),
            fail_message: None,
        };
        let send_webhook_slack_incoming_webhook =
            send_webhook_slack_incoming_webhook(webhooks, webhook_response).await;
        assert!(send_webhook_slack_incoming_webhook.is_ok());
    }
}
