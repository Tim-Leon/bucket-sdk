use crate::client::query_client::{
    backend_api::{
        get_account_details_request::User, DeleteAccountRequest, DeleteAccountResponse,
        GetAccountDetailsRequest, GetAccountDetailsResponse, UpdateAccountRequest,
        UpdateAccountResponse,
    },
    QueryClient,
};

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

pub async fn get_account_details(
    client: &mut QueryClient,
    user: User,
) -> Result<GetAccountDetailsResponse, tonic::Status> {
    let req = GetAccountDetailsRequest { user: Some(user) };
    let resp = client.get_account_details(req).await?.into_inner();
    Ok(resp)
}

pub async fn update_account(
    client: &mut QueryClient,
    req: UpdateAccountRequest,
) -> Result<UpdateAccountResponse, tonic::Status> {
    let resp = client.update_account(req).await?.into_inner();
    Ok(resp)
}

pub async fn delete_account(
    client: &mut QueryClient,
    req: DeleteAccountRequest,
) -> Result<DeleteAccountResponse, tonic::Status> {
    let resp = client.delete_account(req).await?.into_inner();
    Ok(resp)
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
