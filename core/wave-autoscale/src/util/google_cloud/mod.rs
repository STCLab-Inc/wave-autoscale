use gcp_auth::AuthenticationManager;
use log::error;
pub mod gcp_managed_instance_group;
pub mod google_cloud_functions_helper;

async fn get_gcp_credential_token() -> Result<String, anyhow::Error> {
    // Set ENV
    //std::env::set_var("GOOGLE_APPLICATION_CREDENTIALS", "/Users/ari/work/develop/keys/gcp/wave-autoscale-test-510ffb543810.json");
    /* std::env::set_var(
        "GOOGLE_APPLICATION_CREDENTIALS",
        "/Users/bryan/Desktop/stclab/workspace/wave-autoscale/wave-autoscale-test-b5a2ece3a508.json",
    ); */
    let authentication_manager = AuthenticationManager::new().await;
    if authentication_manager.is_err() {
        let authentication_manager_err = authentication_manager.err().unwrap();
        error!(
            "Failed to get gcp authentication manager: {:?}",
            authentication_manager_err.to_string()
        );
        return Err(anyhow::anyhow!(
            "Failed to get gcp authentication manager - {:?}",
            authentication_manager_err.to_string()
        ));
    }
    let scopes = &[
        "https://www.googleapis.com/auth/compute",
        "https://www.googleapis.com/auth/cloud-platform",
    ];

    let Ok(token) = authentication_manager.unwrap().get_token(scopes).await else {
        error!("Failed to get gcp authentication manager - to str");
        return Err(anyhow::anyhow!("Failed to get gcp authentication manager - to str"));
    };
    Ok(token.as_str().to_string())
}

#[cfg(test)]
mod test {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn test_get_gcp_credential_token() {
        // fail case - 401 error (UNAUTHENTICATED)
        std::env::set_var("GOOGLE_APPLICATION_CREDENTIALS", "/test.json");
        let err_token = get_gcp_credential_token();
        assert!(err_token.await.is_err());

        std::env::set_var(
            "GOOGLE_APPLICATION_CREDENTIALS",
            "./tests/credentials/wave-autoscale-test-gcp.json",
        );
        let token = get_gcp_credential_token();
        assert!(token.await.is_ok());
    }
}
