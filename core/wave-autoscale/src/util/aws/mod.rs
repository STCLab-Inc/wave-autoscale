/**
 * AWS Module
 *
 * This module is responsible for all AWS related functionality.
 *
 * References
 * https://github.com/awslabs/aws-sdk-rust
 */
use super::aws_region::get_aws_region_static_str;
use aws_config::{meta::region::RegionProviderChain, SdkConfig};
use aws_credential_types::Credentials;
use log::debug;
use std::{collections::HashMap, time::SystemTime};
use thiserror::Error;

const PROVIDER_NAME: &str = "wave-autoscale";

#[derive(Error, Debug)]
pub enum GetAwsConfigError {
    #[error("Failed to get AWS region: {0}")]
    GetAwsRegionError(String),
}

pub async fn get_aws_config_with_metadata(
    metadata: &HashMap<String, serde_json::Value>,
) -> Result<SdkConfig, GetAwsConfigError> {
    let access_key = metadata
        .get("access_key")
        .map(|access_key| access_key.to_string());
    let secret_key = metadata
        .get("secret_key")
        .map(|secret_key| secret_key.to_string());
    let region: Option<String> = metadata
        .get("region")
        .and_then(|region| region.as_str().map(|region| region.to_string()));

    get_aws_config(region, access_key, secret_key, None, None).await
}

pub async fn get_aws_config(
    region: Option<String>,
    access_key: Option<String>,
    secret_key: Option<String>,
    session_token: Option<String>,
    expires_after: Option<SystemTime>,
) -> Result<SdkConfig, GetAwsConfigError> {
    // Provide your AWS credentials with the default credential provider chain, which currently looks in:

    // 1. Environment variables: AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY, and AWS_REGION
    // 2. The default credentials files located in ~/.aws/config and ~/.aws/credentials (location can vary per platform)
    // 3. Web Identity Token credentials from the environment or container (including EKS)
    // 4. ECS Container Credentials (IAM roles for tasks)
    // 5. EC2 Instance Metadata Service (IAM Roles attached to instance)

    // Sep 5 2023: SSO Login is not working (https://github.com/awslabs/smithy-rs/pull/2917)

    // Region
    let region_provider = if let Some(region) = region {
        let region_static: &'static str = get_aws_region_static_str(region.as_str());
        if region_static.is_empty() {
            return Err(GetAwsConfigError::GetAwsRegionError(format!(
                "Invalid AWS region: {}",
                region
            )));
        }
        RegionProviderChain::first_try(region_static).or_default_provider()
    } else {
        RegionProviderChain::default_provider()
    };
    let config = aws_config::from_env().region(region_provider);

    // Credentials
    let config = if let (Some(access_key), Some(secret_key)) = (access_key, secret_key) {
        let credentials = Credentials::new(
            access_key,
            secret_key,
            session_token,
            expires_after,
            PROVIDER_NAME,
        );
        debug!("Using provided AWS credentials");
        config.credentials_provider(credentials)
    } else {
        debug!("Using default AWS credentials");
        config
    };
    Ok(config.load().await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_aws_config() {
        let config_with_invalid_region = get_aws_config(
            Some("us-east-1222".to_string()),
            Some("test".to_string()),
            Some("test".to_string()),
            None,
            None,
        )
        .await;
        assert!(config_with_invalid_region.is_err());

        let config = get_aws_config(Some("us-east-1".to_string()), None, None, None, None).await;
        assert!(config.is_ok());
    }
}
