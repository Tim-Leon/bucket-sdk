use bucket_api::backend_api::{DeleteAccountRequest, DeleteAccountResponse, GetAccountDetailsRequest, GetAccountDetailsResponse, UpdateAccountRequest, UpdateAccountResponse};
use tonic::{IntoRequest, Request};
use tonic::Status;
use crate::api::{BucketApiError, BucketClient};

use crate::dto::dto::{DeleteAccountParams, GetAccountDetailsParams, UpdateAccountParams};
use crate::request_ext::RequestAuthorizationMetadataExt;
// pub async fn get_user_signing_key(
//     client: &mut QueryClient,
//     user_id: &uuid::Uuid,
// ) -> Result<PublicKey, tonic::Status> {
//     let req = GetUserSigningKeyRequest {
//         user_id: user_id.to_string(),
//     };
//     let resp = client.get_user_signing_key(req).await?.into_inner();
//     let signing_key = PublicKey::from_slice(&resp.signing_key).unwrap();
//     Ok(signing_key)
// }





pub trait AccountClientExt {
    async fn get_account_details(&mut self, param: GetAccountDetailsParams) -> Result<GetAccountDetailsResponse, BucketApiError>;

    async fn update_account(&mut self, param: UpdateAccountParams) -> Result<UpdateAccountResponse, BucketApiError>;

    async fn delete_account(&mut self, param: DeleteAccountParams) -> Result<DeleteAccountResponse, BucketApiError>;
}


impl AccountClientExt for BucketClient {
    async fn get_account_details(&mut self, param: GetAccountDetailsParams) -> Result<GetAccountDetailsResponse, BucketApiError> {
        let mut gadr: GetAccountDetailsRequest = param.try_into().unwrap();
        let mut req = gadr.into_request();
        req.set_authorization_metadata(&self.api_token);
        let resp = self.client.get_account_details(req).await?;
        Ok(resp.into_inner())
    }

    async fn update_account(&mut self, param: UpdateAccountParams) -> Result<UpdateAccountResponse, BucketApiError> {
        let mut uar: UpdateAccountRequest = param.try_into().unwrap();
        let mut req = uar.into_request();
        req.set_authorization_metadata(&self.api_token);
        let resp = self.client.update_account(req).await?;
        Ok(resp.into_inner())
    }

    async fn delete_account(&mut self, param: DeleteAccountParams) -> Result<DeleteAccountResponse, BucketApiError> {
        let mut dar: DeleteAccountRequest = param.try_into().unwrap();
        let mut req = dar.into_request();
        req.set_authorization_metadata(&self.api_token);
        let resp = self.client.delete_account(req).await?;
        Ok(resp.into_inner())
    }
}

// pub async fn set_account_signature_pk(
//     client: &mut QueryClient,
//     pk: &ed25519_compact::PublicKey,
// ) -> Result<SetUserSigningKeyResponse, tonic::Status> {
//     let req = SetUserSigningKeyRequest {
//         version: todo!(),
//         public_signing_key: pk,
//     };
//     Ok(client.set_user_signing_key(req).await?)
// }
