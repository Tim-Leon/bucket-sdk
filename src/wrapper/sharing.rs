use crate::api::{BucketApiError, BucketClient};
use crate::client::grpc::request_ext::RequestAuthorizationMetadataExt;
use bucket_api::backend_api::{CreateBucketShareLinkRequest, CreateBucketShareLinkResponse};
use tonic::Request;
use crate::dto::sharing::CreateBucketShareLinkParams;

pub trait ClientBucketSharingExt {
    async fn create_bucket_share_link(
        &mut self,
        param: CreateBucketShareLinkParams,
    ) -> Result<CreateBucketShareLinkResponse, BucketApiError>;
}

impl ClientBucketSharingExt for BucketClient {
    async fn create_bucket_share_link(
        &mut self,
        param: CreateBucketShareLinkParams,
    ) -> Result<CreateBucketShareLinkResponse, BucketApiError> {
        let cbslr: CreateBucketShareLinkRequest = param.try_into()?;
        let mut req = Request::new(cbslr);
        req.set_authorization_metadata(&self.api_token);
        Ok(self
            .client
            .create_bucket_share_link(req)
            .await
            .unwrap()
            .into_inner())
    }
}
