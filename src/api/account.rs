use bucket_api::backend_api::{DeleteAccountRequest, DeleteAccountResponse, GetAccountDetailsRequest, GetAccountDetailsResponse, UpdateAccountRequest, UpdateAccountResponse};
use tonic::IntoRequest;
use crate::api::AccountClientExt;
use crate::dto::account::{DeleteAccountParams, GetAccountDetailsParams, UpdateAccountParams};
use crate::client::grpc::request_ext::RequestAuthorizationMetadataExt;

impl AccountClientExt for crate::api::BucketClient {
    async fn get_account_details(
        &mut self,
        param: GetAccountDetailsParams,
    ) -> Result<GetAccountDetailsResponse, crate::api::BucketApiError> {
        let mut gadr: GetAccountDetailsRequest = param.try_into().unwrap();
        let mut req = gadr.into_request();
        req.set_authorization_metadata(&self.api_token);
        let resp = self.client.get_account_details(req).await?;
        Ok(resp.into_inner())
    }

    async fn update_account(
        &mut self,
        param: UpdateAccountParams,
    ) -> Result<UpdateAccountResponse, crate::api::BucketApiError> {
        let mut uar: UpdateAccountRequest = param.try_into().unwrap();
        let mut req = uar.into_request();
        req.set_authorization_metadata(&self.api_token);
        let resp = self.client.update_account(req).await?;
        Ok(resp.into_inner())
    }

    async fn delete_account(
        &mut self,
        param: DeleteAccountParams,
    ) -> Result<DeleteAccountResponse, crate::api::BucketApiError> {
        let mut dar: DeleteAccountRequest = param.try_into().unwrap();
        let mut req = dar.into_request();
        req.set_authorization_metadata(&self.api_token);
        let resp = self.client.delete_account(req).await?;
        Ok(resp.into_inner())
    }
}
