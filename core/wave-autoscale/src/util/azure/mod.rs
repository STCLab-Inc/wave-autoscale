pub mod azure_virtual_machine_scale_sets;
use azure_core;
use azure_identity::client_credentials_flow;
use log::error;

#[derive(Clone)]
pub struct AzureCredential {
    pub client_id: String,
    pub client_secret: String,
    pub tenant_id: String,
}

async fn get_azure_credential_token(
    azure_credential: AzureCredential,
) -> Result<String, anyhow::Error> {
    if azure_credential.client_id.is_empty()
        || azure_credential.client_secret.is_empty()
        || azure_credential.tenant_id.is_empty()
    {
        error!("Failed to get azure authentication - parameters required");
        return Err(anyhow::anyhow!(
            "Failed to get azure authentication - parameters required"
        ));
    }

    let http_client = azure_core::new_http_client();
    let client_credential = client_credentials_flow::perform(
        http_client.clone(),
        azure_credential.client_id.as_str(),
        azure_credential.client_secret.as_str(),
        &["https://management.azure.com/.default"],
        azure_credential.tenant_id.as_str(),
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
}

#[cfg(test)]
mod test {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn test_get_azure_credential_token() {
        let azure_credential = AzureCredential {
            client_id: std::env::var("AZURE_CLIENT_ID").unwrap(),
            client_secret: std::env::var("AZURE_CLIENT_SECRET").unwrap(),
            tenant_id: std::env::var("AZURE_TENANT_ID").unwrap(),
        };
        let token = get_azure_credential_token(azure_credential);
        assert!(token.await.is_ok());
    }
}
