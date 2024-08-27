use std::rc::Rc;
use bucket_api::backend_api;
use bucket_api::backend_api::{CreateBucketResponse, DeleteAccountResponse, DeleteBucketResponse, DeleteFilesInBucketResponse, GetAccountDetailsResponse, GetBucketDetailsResponse, GetBucketFilestructureResponse, MoveFilesInBucketResponse, UpdateAccountResponse, UpdateBucketResponse};
use generic_array::ArrayLength;
use tonic::transport::Uri;
use crate::api::authentication::{LoginError, RegistrationError};
use crate::client::grpc::native::client::query_client::QueryClient;
use crate::client::http::{HttpDownloadClientExt, HttpUploadClientExt};
use crate::compression::CompressionChooserHandling;
use crate::wrapper::bucket::bucket::{ DownloadFilesFromBucketError};
use crate::wrapper::bucket::errors::{DownloadError, UploadError};
use crate::io::FileWrapper;
use crate::wrapper::bucket::upload::FileUploadHandler;
use crate::dto::account::{DeleteAccountParams, DeleteAccountParamsParsingError, GetAccountDetailsParams, GetAccountDetailsParamsParsingError, UpdateAccountParams, UpdateAccountParamsParsingError};
use crate::dto::authentication::{LoginParams, RegistrationParams};
use crate::dto::bucket::{CreateBucketParams, CreateBucketParamsParsingError, DeleteBucketParams, DeleteFilesInBucketParams, DeleteFilesInBucketParamsParsingError, DownloadBucketParams, DownloadBucketParamsParsingError, DownloadFilesParams, DownloadFilesParamsParsingError, GetBucketDetailsParams, GetBucketDetailsRequestParsingError, GetFilesystemDetailsParams, GetFilesystemDetailsParamsParsingError, MoveFilesInBucketParams, MoveFilesInBucketRequestParsingError, ParseDeleteBucketRequestError, UpdateBucketParams, UpdateBucketParamsParsingError, UploadFilesParams, UploadFilesRequestParsingError};
use crate::dto::checkout::CreateCheckoutParamsParsingError;
use crate::dto::sharing::CreateBucketShareLinkParamsParsingError;
use crate::encryption::EncryptionChooserHandler;
use crate::token::ApiToken;
use crate::wrapper::bucket::download::FileDownloadHandlerBuilder;

pub mod authentication;

pub mod bucket;
pub mod builder;
pub mod account;
pub mod payment;

pub trait ClientBucketExt<R: std::io::Read, W: std::io::Write> {
    async fn create_bucket(
        &mut self,
        param: CreateBucketParams,
    ) -> Result<CreateBucketResponse, BucketApiError>;
    async fn delete_bucket(
        &mut self,
        param: DeleteBucketParams,
    ) -> Result<DeleteBucketResponse, BucketApiError>;

    async fn update_bucket(
        &mut self,
        param: UpdateBucketParams,
    ) -> Result<UpdateBucketResponse, BucketApiError>;

    async fn get_bucket_details(
        &mut self,
        param: GetBucketDetailsParams,
    ) -> Result<GetBucketDetailsResponse, BucketApiError>;

    async fn upload_files_to_bucket<File: FileWrapper,HTTP: HttpUploadClientExt>(
        &mut self,
        param: UploadFilesParams<File>,
        upload_file_handler: impl FileUploadHandler<R, W>,
        http_client: HTTP,
    ) -> Result<(), BucketApiError>;
    ///https://repost.aws/questions/QUxynkZDbASDaqrUcpx_sILQ/s3-support-multiple-byte-ranges-download
    async fn download_files_from_bucket<N: ArrayLength,HTTP: HttpDownloadClientExt, CCH: CompressionChooserHandling<R, W>, ECH: EncryptionChooserHandler<R, W, N>, FDHB: FileDownloadHandlerBuilder<R,W,N,HTTP, CCH, ECH>>(
        &mut self,
        param: DownloadFilesParams,
        //file_handle: BucketFileTrait<Error = BucketFileError, FileHandle = FileHandle>,
        // Hook function will take in the details for the file and either return a WebBucketFile or NativeBucketFile depending on enviorment implementation, diffrent between WASM and NATIVE.
        create_file_download_handler: FDHB,
    ) -> Result<(), BucketApiError>;

    ///
    /// Downloads the entire bucket.
    /// # Arguments
    ///
    /// * `param`:
    /// * `create_file_download_handler`:
    /// * `additional_param`:
    ///
    /// returns: Result<Vec<String, Global>, BucketApiError>
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    async fn download_bucket<N:ArrayLength,HTTP: HttpDownloadClientExt,CCH: CompressionChooserHandling<R, W>, ECH: EncryptionChooserHandler<R, W, N>, FDHB: FileDownloadHandlerBuilder<R, W,N, HTTP, CCH, ECH>>(
        &mut self,
        param: DownloadBucketParams,
        file_download_handler_builder: FDHB,
        http_client: HTTP,
    ) -> Result<Vec<String>, BucketApiError>;

    async fn move_files_in_bucket(
        &mut self,
        param: MoveFilesInBucketParams,
    ) -> Result<MoveFilesInBucketResponse, BucketApiError>;

    async fn delete_files_in_bucket(
        &mut self,
        param: DeleteFilesInBucketParams,
    ) -> Result<DeleteFilesInBucketResponse, BucketApiError>;

    async fn get_bucket_filestructure_fully(
        &mut self,
        param: GetFilesystemDetailsParams,
    ) -> Result<Vec<backend_api::File>, BucketApiError>;

    async fn get_bucket_filestructure(
        &mut self,
        param: GetFilesystemDetailsParams,
    ) -> Result<GetBucketFilestructureResponse, BucketApiError>;

}


pub trait AccountClientExt {
    async fn get_account_details(
        &mut self,
        param: GetAccountDetailsParams,
    ) -> Result<GetAccountDetailsResponse, BucketApiError>;

    async fn update_account(
        &mut self,
        param: UpdateAccountParams,
    ) -> Result<UpdateAccountResponse, BucketApiError>;

    async fn delete_account(
        &mut self,
        param: DeleteAccountParams,
    ) -> Result<DeleteAccountResponse, BucketApiError>;
}

pub trait AuthenticationClientExt {
    async fn login(&mut self, param: &LoginParams) -> Result<ApiToken, LoginError>;

    async fn register(&mut self, param: &RegistrationParams)
                      -> Result<ApiToken, RegistrationError>;
}


pub trait BucketClientBuilder: Sized {
    async fn from_token(api_url: Uri, api_token: ApiToken) -> Self;
    async fn from_env() -> Self;
    async fn plaintext_credentials_registration(
        api_url: Uri,
        email: &email_address::EmailAddress,
        username: &str,
        password: &str,
        captcha: &str,
    ) -> Result<Self, RegistrationError>;
    async fn plaintext_credentials_login(
        api_url: Uri,
        login_params: &LoginParams,
    ) -> Result<Self, LoginError>;
}




pub struct BucketClient {
    pub client: QueryClient,
    pub api_token: ApiToken,
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

    // Request handling/wrapper failed
    #[error(transparent)]
    DownloadFilesFromBucketError(#[from] DownloadFilesFromBucketError),

    #[error(transparent)]
    DownloadError(#[from] DownloadError),
    #[error(transparent)]
    UploadError(#[from] UploadError),

    // Response parsing error
    #[error("GetBucketDetailsRequestFullyResponseParsingError")]
    GetBucketDetailsRequestFullyResponseParsingError,
}
