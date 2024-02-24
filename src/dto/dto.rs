use bucket_common_types::{
    unix_timestamp::UnixTimestamp, BucketCompression, BucketEncryption, BucketGuid,
    BucketRedundancy, BucketStorageClass, BucketVisibility, PaymentModel, RegionCluster,
};

use crate::{
    controller::bucket::{
        bucket::DownloadFilesFromBucketError,
        errors::{DownloadError, UploadError},
        io::file::VirtualFileDetails,
    },
    query_client::backend_api::{
        download_files_request::File, get_account_details_request::User, CreateBucketRequest,
        CreateBucketShareLinkRequest, CreateCheckoutRequest, DeleteAccountRequest,
        DeleteBucketRequest, DeleteFilesInBucketRequest, DownloadBucketRequest,
        DownloadFilesRequest, GetAccountDetailsRequest,
        GetBucketDetailsRequest, GetBucketFilestructureRequest, MoveFilesInBucketRequest,
        UpdateAccountRequest, UpdateBucketRequest, UploadFilesToBucketRequest,
    },
};

pub struct CreateBucketParams {
    pub target_user_id: uuid::Uuid,
//  pub target_bucket_id: uuid::Uuid,
    pub name: String,
    pub visibility: Option<BucketVisibility>,
    pub encryption: Option<BucketEncryption>,
    pub password: Option<String>,
    pub target_directory: String,
    pub source_files: Vec<VirtualFileDetails>,
    pub description: Option<String>,
    pub storage_class: BucketStorageClass,
    pub expire_at: Option<bucket_common_types::unix_timestamp::UnixTimestamp>,
    pub expected_capacity: Option<u64>,
    pub is_nsfw: bool,
    pub is_searchable: bool,
    pub is_sharable: bool,
    pub is_bucket_cloneable: bool,
    pub is_prepaid: bool,
    pub bucket_compression: Option<BucketCompression>,
    pub tags: Vec<String>,
    //pub redundancy: Option<BucketRedundancy>,
    pub total_size_in_bytes: usize, // Can not go over this value. Going over will result in overwriting previous writes. pretty much unexpected behavior.
}
#[derive(thiserror::Error, Debug)]
pub enum CreateBucketParamsParsingError {}

impl TryInto<CreateBucketRequest> for CreateBucketParams {
    type Error = CreateBucketParamsParsingError;

    fn try_into(self) -> Result<CreateBucketRequest, Self::Error> {
        Ok(CreateBucketRequest {
            name: self.name,
            visibility: self.visibility.map(|x| x.to_string()),
            encryption: self.encryption.map(|x| x.to_string()),
            password: self.password,
            description: self.description,
            storage_class: self.storage_class.to_string(),
            tags: self.tags,
            expires_timestamp: self.expire_at.map(|x| x.try_into().unwrap()),
            expected_capacity_in_bytes: self.expected_capacity,
            is_nsfw: self.is_nsfw,
            is_searchable: self.is_searchable,
            is_bucket_cloneable: self.is_bucket_cloneable,
            is_sharable: self.is_sharable,
            is_prepaid: self.is_prepaid,
            bucket_compression: self.bucket_compression.map(|x| x.to_string()),
        })
    }
}

pub struct DeleteBucketParams {
    pub bucket_guid: BucketGuid,
}
#[derive(thiserror::Error, Debug)]
pub enum ParseDeleteBucketRequestError {}

impl TryInto<DeleteBucketRequest> for DeleteBucketParams {
    type Error = ParseDeleteBucketRequestError;

    fn try_into(self) -> Result<DeleteBucketRequest, Self::Error> {
        Ok(DeleteBucketRequest {
            bucket_id: self.bucket_guid.bucket_id.to_string(),
            bucket_owner_id: self.bucket_guid.user_id.to_string(),
        })
    }
}

pub struct UpdateBucketParams {
    pub bucket_id: uuid::Uuid,
    pub bucket_user_id: uuid::Uuid,
    pub name: Option<String>,
    pub visibility: Option<BucketVisibility>,
    pub encryption: Option<BucketEncryption>,
    pub password: Option<String>,
    //pre_alllocated_capacity_in_bytes: u64,
    pub redundancy: Option<BucketRedundancy>,
    pub region_cluster: Option<RegionCluster>,
    pub description: Option<String>,
    pub storage_class: Option<BucketStorageClass>,
    pub opt_tags: Vec<String>,
    pub expires_timestamp: Option<bucket_common_types::unix_timestamp::UnixTimestamp>,
    pub expected_size_in_bytes: Option<u64>,
    pub bucket_compression: Option<BucketCompression>,
    pub is_nsfw: Option<bool>,
    pub is_searchable: Option<bool>,
    pub is_bucket_cloneable: Option<bool>,
    pub is_sharable: Option<bool>,
}

#[derive(thiserror::Error, Debug)]
pub enum UpdateBucketParamsParsingError {}
impl TryInto<UpdateBucketRequest> for UpdateBucketParams {
    type Error = UpdateBucketParamsParsingError;

    fn try_into(self) -> Result<UpdateBucketRequest, Self::Error> {
        Ok(UpdateBucketRequest {
            bucket_id: self.bucket_id.to_string(),
            bucket_user_id: self.bucket_user_id.to_string(),
            name: self.name,
            visibility: self.visibility.map(|x| x.to_string()),
            encryption: self.encryption.map(|x| x.to_string()),
            password: self.password,
            pre_allocated_capacity_in_bytes: None,
            redundancy: None,
            region_cluster: self.region_cluster.map(|x| x.to_string()),
            description: self.description,
            storage_class: self.storage_class.map(|x| x.to_string()),
            opt_tags: self.opt_tags,
            expires_timestamp: self.expires_timestamp.map(|x| x.try_into().unwrap()),
            expected_size_in_bytes: self.expected_size_in_bytes,
            is_nsfw: self.is_nsfw,
            is_searchable: self.is_searchable,
            is_bucket_cloneable: self.is_bucket_cloneable,
            is_sharable: self.is_sharable,
            bucket_compression: self.bucket_compression.map(|x| x.to_string()),
        })
    }
}

pub struct GetBucketDetailsParams {
    //pub bucket_guid: BucketGuid,
    pub opt_bucket_ids: Vec<uuid::Uuid>,
    pub bucket_owner_id: uuid::Uuid,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(thiserror::Error, Debug)]
pub enum GetBucketDetailsRequestParsingError {}

impl TryInto<GetBucketDetailsRequest> for GetBucketDetailsParams {
    type Error = GetBucketDetailsRequestParsingError;

    fn try_into(self) -> Result<GetBucketDetailsRequest, Self::Error> {
        Ok(GetBucketDetailsRequest {
            opt_bucket_ids: self.opt_bucket_ids.iter().fold(
                Vec::with_capacity(self.opt_bucket_ids.len()),
                |mut acc, num| {
                    acc.push(num.to_string());
                    acc // Return the accumulator after pushing the element
                },
            ),
            bucket_owner_id: self.bucket_owner_id.to_string(),
            offset: self.offset,
            limit: self.limit,
        })
    }
}

pub struct UploadFile {
    pub target_directory: String,
    pub source_file: VirtualFileDetails,
    pub content_type: mime::Mime,
}

pub struct UploadFilesParams {
    pub target_user_id: uuid::Uuid,
    pub target_bucket_id: uuid::Uuid,
    pub target_directory: String,
    pub source_files: Vec<UploadFile>,
    pub encryption: Option<BucketEncryption>,
    pub total_size_in_bytes: u64, // Can not go over this value. Going over will result in overwriting previous writes. pretty much unexpected behavior.
    pub hashed_password: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum UploadFilesRequestParsingError {}

impl TryInto<UploadFilesToBucketRequest> for UploadFilesParams {
    type Error = UploadFilesRequestParsingError;

    fn try_into(self) -> Result<UploadFilesToBucketRequest, Self::Error> {
        Ok(UploadFilesToBucketRequest {
            target_bucket_id: self.target_bucket_id.to_string(),
            target_bucket_owner_id: self.target_user_id.to_string(),
            target_directory: self.target_directory,
            source_files: self.source_files.iter().fold(Vec::<crate::query_client::backend_api::upload_files_to_bucket_request::File>::with_capacity(self.source_files.len()), |mut acc, num| {
                acc.push(crate::query_client::backend_api::upload_files_to_bucket_request::File {
                    file_path: num.source_file.path.clone(),
                    size_in_bytes: num.source_file.size_in_bytes,
                    content_type: num.content_type.to_string(),
                });
                acc
            }),
            hashed_password: self.hashed_password,
        })
    }
}

pub struct DownloadFilesParams {
    pub target_bucket_id: uuid::Uuid,
    pub target_user_id: uuid::Uuid,
    pub target_directory: String,
    pub files: Vec<VirtualFileDetails>,
    pub hashed_password: Option<String>,
    pub bucket_encryption: Option<BucketEncryption>,
}

#[derive(thiserror::Error, Debug)]
pub enum DownloadFilesParamsParsingError {}

impl TryInto<DownloadFilesRequest> for DownloadFilesParams {
    type Error = DownloadFilesParamsParsingError;

    fn try_into(self) -> Result<DownloadFilesRequest, Self::Error> {
        Ok(DownloadFilesRequest {
            bucket_id: self.target_bucket_id.to_string(),
            bucket_owner_id: self.target_user_id.to_string(),
            files: self.files.iter().fold(
                Vec::<File>::with_capacity(self.files.len()),
                |mut acc, val| {
                    acc.push(File {
                        file_path: val.path.clone(),
                        size_in_bytes: val.size_in_bytes,
                    }); //TODO: fix
                    acc
                },
            ),
            hashed_password: self.hashed_password,
        })
    }
}

pub struct DownloadBucketParams {
    pub bucket_guid: BucketGuid,
    pub hashed_password: Option<String>,
    pub format: Option<bucket_common_types::DownloadFormat>,
    pub keep_file_structure: bool,
}

#[derive(thiserror::Error, Debug)]
pub enum DownloadBucketParamsParsingError {}

impl TryInto<DownloadBucketRequest> for DownloadBucketParams {
    type Error = DownloadBucketParamsParsingError;

    fn try_into(self) -> Result<DownloadBucketRequest, Self::Error> {
        Ok(DownloadBucketRequest {
            bucket_id: self.bucket_guid.bucket_id.to_string(),
            bucket_owner_id: self.bucket_guid.user_id.to_string(),
            hashed_password: self.hashed_password,
            format: self.format.map(|format| format.to_string()),
        })
    }
}

pub struct MoveFilesInBucketParams {
    pub from_bucket_guid: BucketGuid,
    pub to_bukcet_owner_id: Option<uuid::Uuid>,
    pub to_bucket_id: uuid::Uuid,
    pub from_filepaths: Vec<String>,
    pub to_filepath: String,
    pub is_capacity_destructive: bool,
}
#[derive(thiserror::Error, Debug)]
pub enum MoveFilesInBucketRequestParsingError {}

impl TryInto<MoveFilesInBucketRequest> for MoveFilesInBucketParams {
    type Error = MoveFilesInBucketRequestParsingError;

    fn try_into(self) -> Result<MoveFilesInBucketRequest, Self::Error> {
        Ok(MoveFilesInBucketRequest {
            from_bucket_id: self.from_bucket_guid.bucket_id.to_string(),
            from_bucket_owner_id: self.from_bucket_guid.user_id.to_string(),
            from_filepaths: self.from_filepaths,
            to_bucket_id: self.to_bucket_id.to_string(),
            to_bucket_owner_id: self.to_bukcet_owner_id.map(|x| x.to_string()),
            to_directory: self.to_filepath,
            is_capacity_destructive: self.is_capacity_destructive,
        })
    }
}

pub struct DeleteFilesInBucketParams {
    pub bucket_guid: BucketGuid,
    pub filepaths: Vec<String>,
    pub is_capacity_destructive: bool,
}

#[derive(thiserror::Error, Debug)]
pub enum DeleteFilesInBucketParamsParsingError {}

impl TryInto<DeleteFilesInBucketRequest> for DeleteFilesInBucketParams {
    type Error = DeleteFilesInBucketParamsParsingError;

    fn try_into(self) -> Result<DeleteFilesInBucketRequest, Self::Error> {
        Ok(DeleteFilesInBucketRequest {
            bucket_id: self.bucket_guid.bucket_id.to_string(),
            bucket_owner_id: self.bucket_guid.user_id.to_string(),
            filepaths: self.filepaths,
            is_capacity_destructive: self.is_capacity_destructive,
        })
    }
}

pub struct GetFilesystemDetailsParams {
    pub target_bucket_id: uuid::Uuid,
    pub target_bucket_owner_id: Option<uuid::Uuid>,
    pub start_directory: Option<String>,
    pub continuation_token: Option<String>,
}
#[derive(thiserror::Error, Debug)]
pub enum GetFilesystemDetailsParamsParsingError {}

impl TryInto<GetBucketFilestructureRequest> for GetFilesystemDetailsParams {
    type Error = GetFilesystemDetailsParamsParsingError;

    fn try_into(self) -> Result<GetBucketFilestructureRequest, Self::Error> {
        Ok(GetBucketFilestructureRequest {
            bucket_id: self.target_bucket_id.to_string(),
            bucket_owner_id: self.target_bucket_owner_id.map(|x| x.to_string()),
            start_directory: self.start_directory,
            continuation_token: self.continuation_token,
        })
    }
}

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

pub struct GetAccountDetailsParams {
    pub target_user_id: Option<User>,
}
#[derive(thiserror::Error, Debug)]
pub enum GetAccountDetailsParamsParsingError {}

impl TryInto<GetAccountDetailsRequest> for GetAccountDetailsParams {
    type Error = GetAccountDetailsParamsParsingError;

    fn try_into(self) -> Result<GetAccountDetailsRequest, Self::Error> {
        Ok(GetAccountDetailsRequest {
            user: self.target_user_id,
        })
    }
}

pub struct CreateCheckoutParams {
    payment_model: PaymentModel,
    change_payment_model: bool,
}
#[derive(thiserror::Error, Debug)]
pub enum CreateCheckoutParamsParsingError {}
impl TryInto<CreateCheckoutRequest> for CreateCheckoutParams {
    type Error = CreateCheckoutParamsParsingError;

    fn try_into(self) -> Result<CreateCheckoutRequest, Self::Error> {
        Ok(CreateCheckoutRequest {
            payment_model: self.payment_model.to_string(),
            change_payment_model: self.change_payment_model,
        })
    }
}

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

#[derive(thiserror::Error, Debug)]
pub enum BucketApiError {
    // Parsing errors
    #[error(transparent)]
    CreateBucketParamsParsingError(#[from] CreateBucketParamsParsingError),
    #[error(transparent)]
    ParseDeleteBucketRequestError(#[from] ParseDeleteBucketRequestError),
    #[error(transparent)]
    GetBucketDetailsRequestParsingError(#[from] GetBucketDetailsRequestParsingError),
    #[error(transparent)]
    UploadFilesRequestParsingError(#[from] UploadFilesRequestParsingError),
    #[error(transparent)]
    MoveFilesInBucketRequestParsingError(#[from] MoveFilesInBucketRequestParsingError),
    #[error(transparent)]
    DeleteFilesInBucketParamsParsingError(#[from] DeleteFilesInBucketParamsParsingError),
    #[error(transparent)]
    DeleteAccountParamsParsingError(#[from] DeleteAccountParamsParsingError),
    #[error(transparent)]
    DownloadFilesParamsParsingError(#[from] DownloadFilesParamsParsingError),
    #[error(transparent)]
    DownloadBucketParamsParsingError(#[from] DownloadBucketParamsParsingError),
    #[error(transparent)]
    UpdateAccountParamsParsingError(#[from] UpdateAccountParamsParsingError),
    #[error(transparent)]
    GetFilesystemDetailsParamsParsingError(#[from] GetFilesystemDetailsParamsParsingError),
    #[error(transparent)]
    GetAccountDetailsParamsParsingError(#[from] GetAccountDetailsParamsParsingError),
    #[error(transparent)]
    CreateCheckoutParamsParsingError(#[from] CreateCheckoutParamsParsingError),
    #[error(transparent)]
    CreateBucketShareLinkParamsParsingError(#[from] CreateBucketShareLinkParamsParsingError),
    #[error(transparent)]
    UpdateBucketParamsParsingError(#[from] UpdateBucketParamsParsingError),

    // Request handling/controller failed
    #[error(transparent)]
    DownloadFilesFromBucketError(#[from] DownloadFilesFromBucketError),

    #[error(transparent)]
    DownloadError(#[from] DownloadError),
    #[error(transparent)]
    UploadError(#[from] UploadError),
}
