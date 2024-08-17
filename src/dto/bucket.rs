use bucket_api::backend_api;
use bucket_api::backend_api::{download_files_request, CreateBucketRequest, DeleteBucketRequest, DeleteFilesInBucketRequest, DownloadBucketRequest, DownloadFilesRequest, GetBucketDetailsRequest, GetBucketFilestructureRequest, MoveFilesInBucketRequest, UpdateBucketRequest, UploadFilesToBucketRequest};
use bucket_common_types::{BucketCompression, BucketEncryption, BucketGuid, BucketRedundancy, BucketStorageClass, BucketVisibility, RegionCluster};
use crate::io::file::{VirtualFileDetails};
use crate::io::FileWrapper;

pub struct CreateBucketParams {
    pub target_user_id: uuid::Uuid,
    pub name: String,
    pub visibility: Option<BucketVisibility>,
    pub encryption: Option<BucketEncryption>,
    pub password: Option<String>,
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

pub struct UploadFile<File: FileWrapper> {
    pub target_directory: String,
    pub source_file: File,
    //pub content_type: mime::Mime,
}

pub struct UploadFilesParams<File: FileWrapper> {
    pub target_user_id: uuid::Uuid,
    pub target_bucket_id: uuid::Uuid,
    pub target_directory: String,
    pub source_files: Vec<UploadFile<File>>,
    pub encryption: Option<BucketEncryption>,
    pub total_size_in_bytes: u64, // Can not go over this value. Going over will result in overwriting previous writes. pretty much unexpected behavior.
    pub hashed_password: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum UploadFilesRequestParsingError {}

impl TryInto<UploadFilesToBucketRequest> for UploadFilesParams<dyn FileWrapper<Error=(), FileHandle=()>> {
    type Error = UploadFilesRequestParsingError;

    fn try_into(self) -> Result<UploadFilesToBucketRequest, Self::Error> {
        Ok(UploadFilesToBucketRequest {
            target_bucket_id: self.target_bucket_id.to_string(),
            target_bucket_owner_id: self.target_user_id.to_string(),
            target_directory: self.target_directory,
            source_files: self.source_files.iter().fold(
                Vec::<backend_api::upload_files_to_bucket_request::File>::with_capacity(
                    self.source_files.len(),
                ),
                |mut acc, num| {
                    acc.push(backend_api::upload_files_to_bucket_request::File {
                        file_path: num.target_directory.clone(),
                        size_in_bytes: num.source_file.get_size(),
                        content_type: num.source_file.get_mime_type().unwrap().to_string(),
                    });
                    acc
                },
            ),
            hashed_password: self.hashed_password,
        })
    }
}

pub struct DownloadFilesParams {
    pub target_user_id: uuid::Uuid,
    pub target_bucket_id: uuid::Uuid,
    pub target_directory: String,
    pub files: Vec<VirtualFileDetails>,
    pub hashed_password: Option<String>,
    pub bucket_encryption: Option<BucketEncryption>,

    pub keep_file_structure: bool,
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
                Vec::<download_files_request::File>::with_capacity(self.files.len()),
                |mut acc, val| {
                    acc.push(download_files_request::File {
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
    pub to_bucket_owner_id: Option<uuid::Uuid>,
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
            to_bucket_owner_id: self.to_bucket_owner_id.map(|x| x.to_string()),
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

#[derive(Clone)]
pub struct GetFilesystemDetailsParams {
    pub target_bucket_id: uuid::Uuid,
    pub target_bucket_owner_id: Option<uuid::Uuid>,
    pub start_directory: Option<String>,
    pub continuation_token: Option<String>,
    pub page: bool,
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
            page: self.page,
        })
    }
}
