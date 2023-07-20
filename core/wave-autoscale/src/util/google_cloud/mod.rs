use gcp_auth::{AuthenticationManager, Token, Error};
use log::{error};
pub mod gcp_managed_instance_gorup;


async fn get_gcp_credential_token() -> Result<Token,Error> {
    // Set ENV
    //std::env::set_var("GOOGLE_APPLICATION_CREDENTIALS", "/Users/ari/work/develop/keys/gcp/wave-autoscale-test-510ffb543810.json");
    let authentication_manager = AuthenticationManager::new().await;
    if authentication_manager.is_err() {
        let authentication_manager_err = authentication_manager.err().unwrap();
        error!("Failed to get gcp authentication manager: {:?}", authentication_manager_err.to_string());
        return Err(authentication_manager_err);
    }
    let scopes = &["https://www.googleapis.com/auth/compute","https://www.googleapis.com/auth/cloud-platform"];
    
    authentication_manager.unwrap().get_token(scopes).await
}


#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_get_gcp_credential_token() {

        // fail case - 401 error (UNAUTHENTICATED)
        std::env::set_var("GOOGLE_APPLICATION_CREDENTIALS", "/test.json");
        let err_token = get_gcp_credential_token();
        assert!(err_token.await.is_err());

        std::env::set_var("GOOGLE_APPLICATION_CREDENTIALS", "/Users/ari/work/develop/keys/gcp/wave-autoscale-test-510ffb543810.json");
        let token = get_gcp_credential_token();
        assert!(token.await.is_ok());

    }

}