use bucket_api::backend_api::{CreateCheckoutRequest, CreateCheckoutResponse};
use tonic::Request;
use crate::api::{BucketApiError, BucketClient};
use crate::dto::dto::CreateCheckoutParams;
use crate::request_ext::RequestAuthorizationMetadataExt;

pub trait ClientPaymentExt {
    async fn create_checkout(
        &mut self,
        param: CreateCheckoutParams,
    ) -> Result<CreateCheckoutResponse, BucketApiError>;
}

impl ClientPaymentExt for BucketClient {
    async fn create_checkout(&mut self, param: CreateCheckoutParams) -> Result<CreateCheckoutResponse, BucketApiError> {
        let ccr: CreateCheckoutRequest = param.try_into()?;
        let mut req = Request::new(ccr);
        req.set_authorization_metadata(&self.api_token);
        Ok(self.client.create_checkout(req).await.unwrap().into_inner())
    }
}
