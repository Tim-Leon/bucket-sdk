use bucket_api::backend_api::CreateBucketShareLinkRequest;
use bucket_common_types::BucketGuid;
use bucket_common_types::unix_timestamp::UnixTimestamp;

pub struct CreateBucketShareLinkParams {
    pub target_bucket_guid: BucketGuid,
    pub expires: Option<UnixTimestamp>,
    pub usages: Option<u32>,
    pub registered_users_only: bool,
    pub view_permission: bool,
    pub read_permission: bool,
    pub write_permission: bool,
    pub delete_file_permission: bool,
    pub delete_bucket_permission: bool,
    pub share_bucket_permission: bool,
    pub clone_permission: bool,
    pub search_permission: bool,
    pub is_secret_share_link: bool,
}
#[derive(thiserror::Error, Debug)]
pub enum CreateBucketShareLinkParamsParsingError {
    #[error(transparent)]
    FailedToParseExpiresTimestamp(#[from] time::error::ComponentRange),
}
impl TryInto<CreateBucketShareLinkRequest> for CreateBucketShareLinkParams {
    type Error = CreateBucketShareLinkParamsParsingError;

    fn try_into(self) -> Result<CreateBucketShareLinkRequest, Self::Error> {
        Ok(CreateBucketShareLinkRequest {
            user_id: self.target_bucket_guid.user_id.to_string(),
            bucket_id: self.target_bucket_guid.bucket_id.to_string(),
            expires: self.expires.map(|x| x.try_into().unwrap()),
            usages: self.usages,
            registered_users_only: self.registered_users_only,
            view_permission: self.view_permission,
            read_permission: self.read_permission,
            write_permission: self.write_permission,
            delete_file_permission: self.delete_file_permission,
            delete_bucket_permission: self.delete_bucket_permission,
            share_bucket_permission: self.share_bucket_permission,
            clone_permission: self.clone_permission,
            search_permission: self.search_permission,
            is_secret_share_link: self.is_secret_share_link,
        })
    }
}
