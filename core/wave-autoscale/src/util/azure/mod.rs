pub mod azure_autoscale_settings_helper;
pub mod azure_virtual_machine_scale_sets;
use azure_core;
use azure_core::auth::TokenCredential;
use azure_identity::client_credentials_flow;
use log::error;

#[derive(Clone)]
pub struct AzureCredential {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub tenant_id: Option<String>,
}

async fn get_azure_credential_token(
    azure_credential: AzureCredential,
) -> Result<String, anyhow::Error> {
    if let (Some(client_id), Some(client_secret), Some(tenant_id)) = (
        azure_credential.client_id,
        azure_credential.client_secret,
        azure_credential.tenant_id,
    ) {
        let http_client = azure_core::new_http_client();
        let client_credential = client_credentials_flow::perform(
            http_client.clone(),
            client_id.as_str(),
            client_secret.as_str(),
            &["https://management.azure.com/.default"],
            tenant_id.as_str(),
        )
        .await;

        if client_credential.is_err() {
            error!("Failed to get azure authentication - client_credential");
            return Err(anyhow::anyhow!(
                "Failed to get azure authentication - client_credential"
            ));
        }
        let token = client_credential
            .unwrap()
            .access_token()
            .secret()
            .to_string();
        Ok(token)
    } else {
        let credential = azure_identity::DefaultAzureCredentialBuilder::default().build();
        let client_credential = credential.get_token("https://management.azure.com").await;
        if client_credential.is_err() {
            error!("Failed to get azure authentication - DefaultAzureCredentialBuilder");
            return Err(anyhow::anyhow!(
                "Failed to get azure authentication - DefaultAzureCredentialBuilder"
            ));
        }
        let token = client_credential.unwrap().token.secret().to_string();

        Ok(token)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn test_get_azure_credential_token_err() {
        let azure_credential = AzureCredential {
            client_id: Some("AZURE_CLIENT_ID".to_string()),
            client_secret: Some("AZURE_CLIENT_SECRET".to_string()),
            tenant_id: Some("rLA8Q~jDhLZ8wWByiTHw7mSpdbc2W3egESs2XaeS".to_string()),
        };
        let token = get_azure_credential_token(azure_credential);
        assert!(token.await.is_err());
    }

    #[ignore]
    #[tokio::test]
    async fn test_get_azure_credential_token_ok() {
        let azure_credential = AzureCredential {
            client_id: None,
            client_secret: None,
            tenant_id: None,
        };
        let token = get_azure_credential_token(azure_credential);
        assert!(token.await.is_ok());
    }
}
