use std::str::FromStr;
use email_address::EmailAddress;
use tonic::transport::Uri;
use crate::api::{ApiToken, AuthenticationClientExt, BucketClient, BucketClientBuilder};
use crate::api::authentication::{register, LoginError, RegistrationError};
use crate::client::grpc::native::client::query_client::QueryClient;
use crate::client::grpc::QueryClientBuilder;
use crate::dto::authentication::LoginParams;

impl BucketClientBuilder for BucketClient {
    async fn from_token(api_url: Uri, api_token: ApiToken) -> Self {
        let client = QueryClient::build(api_url).await;
        BucketClient { client, api_token }
    }

    /// Uses environment variables:
    /// API_URL
    /// API_TOKEN
    async fn from_env() -> Self {
        let api_url = std::env::var("API_URL").unwrap();
        let api_token = std::env::var("API_TOKEN").unwrap();
        let client = QueryClient::build(Uri::from_str(api_url.as_str()).unwrap()).await;

        Self {
            client,
            api_token: ApiToken::try_from(api_token.as_str()).unwrap(),
        }
    }

    async fn plaintext_credentials_registration(
        api_url: Uri,
        email: &EmailAddress,
        username: &str,
        password: &str,
        captcha: &str,
    ) -> Result<Self, RegistrationError> {
        let mut client = QueryClient::build(api_url).await;
        let api_token = register(&mut client, email, username, password, captcha).await?;
        Ok(Self { client, api_token })
    }

    async fn plaintext_credentials_login(
        api_url: Uri,
        login_params: &LoginParams,
    ) -> Result<Self, LoginError> {
        let mut client = QueryClient::build(api_url).await;
        let api_token = client.login(login_params).await?;
        Ok(BucketClient { client, api_token })
    }
}