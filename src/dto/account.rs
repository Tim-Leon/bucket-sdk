use bucket_api::backend_api;
use bucket_api::backend_api::{DeleteAccountRequest, GetAccountDetailsRequest, UpdateAccountRequest};

pub struct UpdateAccountParams {}

#[derive(thiserror::Error, Debug)]
pub enum UpdateAccountParamsParsingError {}
impl TryInto<UpdateAccountRequest> for UpdateAccountParams {
    type Error = UpdateAccountParamsParsingError;

    fn try_into(self) -> Result<UpdateAccountRequest, Self::Error> {
        todo!()
    }
}
pub struct DeleteAccountParams {
    pub target_user_id: uuid::Uuid,
}
#[derive(thiserror::Error, Debug)]
pub enum DeleteAccountParamsParsingError {}

impl TryInto<DeleteAccountRequest> for DeleteAccountParams {
    type Error = DeleteAccountParamsParsingError;

    fn try_into(self) -> Result<DeleteAccountRequest, Self::Error> {
        Ok(DeleteAccountRequest {
            user_id: self.target_user_id.to_string(),
        })
    }
}

pub enum User {
    UserId(uuid::Uuid),
    Username(String),
}

pub struct GetAccountDetailsParams {
    pub target_user_id: Option<User>,
}
#[derive(thiserror::Error, Debug)]
pub enum GetAccountDetailsParamsParsingError {}

impl TryInto<GetAccountDetailsRequest> for GetAccountDetailsParams {
    type Error = GetAccountDetailsParamsParsingError;

    fn try_into(self) -> Result<GetAccountDetailsRequest, Self::Error> {
        Ok(GetAccountDetailsRequest {
            user: self.target_user_id.map(|x| match x {
                User::UserId(user_id) => {
                    backend_api::get_account_details_request::User::UserId(user_id.to_string())
                }
                User::Username(username) => {
                    backend_api::get_account_details_request::User::Username(username)
                }
            }),
        })
    }
}
