use std::io::{Read, Write};
use std::rc::Rc;
use bucket_api::backend_api::{DeleteFilesInBucketRequest, DeleteFilesInBucketResponse, DownloadBucketRequest, DownloadFilesRequest, GetBucketDetailsFromUrlResponse, GetBucketDetailsRequest, GetBucketDetailsResponse, GetBucketFilestructureRequest, MoveFilesInBucketRequest, MoveFilesInBucketResponse, UploadFilesToBucketRequest};
use bucket_common_types::{DownloadFormat, Encoding};
use bucket_common_types::exclusive_share_link::ExclusiveShareLink;
use bucket_common_types::share_link::ShareLink;
use generic_array::ArrayLength;
use mime::Mime;
use tonic::Request;
use url::Url;
use zero_knowledge_encryption::encryption::aead::EncryptionModule;
use crate::client::http::{HttpDownloadClientExt, HttpUploadClientExt};
use crate::compression::{CompressionChooserHandling, CompressorModule};
use crate::dto::bucket::DownloadFilesParams;
use crate::encryption::EncryptionChooserHandler;
use crate::io::FileWrapper;
use crate::wrapper::bucket::bucket::{DownloadFilesFromBucketError};
use crate::wrapper::bucket::errors::{DeleteFileInBucketError, DownloadError, GetFilesystemDetailsError, MoveFilesInBucketError, UploadError, UploadToUrlError};
use crate::wrapper::bucket::upload::FileUploadHandler;
use crate::token::ApiToken;
use crate::wrapper::bucket::download::FileDownloadHandlerBuilder;

pub mod bucket;
pub mod errors;
pub mod download;
pub mod upload;

pub trait ClientUploadExt {
    /// Note: THe api token need to be set for the request in order for it to work.
    async fn upload_files_to_bucket_raw<R: Read, W: Write, UH: FileUploadHandler<R, W>, HTTP: HttpUploadClientExt>(
        &mut self,
        req: tonic::Request<UploadFilesToBucketRequest>,
        upload_handler: UH,
        api_token: &ApiToken,
        http_client: HTTP,
    ) -> Result<(), UploadError>;
    async fn download_from_url_raw<R: Read, W: Write, N: ArrayLength, HTTP: HttpDownloadClientExt, CCH: CompressionChooserHandling<R, W>, ECH:EncryptionChooserHandler<R, W, N>,FDHB: FileDownloadHandlerBuilder<R, W, N, HTTP, CCH, ECH>>(
        &mut self,
        api_token: &ApiToken,
        url: ExclusiveShareLink,
        hashed_password: Option<String>,
        format: Option<DownloadFormat>,
        create_download_handler: FDHB,
    )    -> Result<(), DownloadError>;

    async fn download_files_from_bucket_raw<R: Read,W: Write, N: ArrayLength,HTTP: HttpDownloadClientExt,CCH: CompressionChooserHandling<R, W>, ECH: EncryptionChooserHandler<R, W, N>,FDHB: FileDownloadHandlerBuilder<R, W, N,HTTP, CCH, ECH>>(
        &mut self,
        param: DownloadFilesParams,
        file_download_handler_builder: FDHB
    )  -> Result<(), DownloadFilesFromBucketError>;

    async fn download_bucket_raw<R: Read, W: Write, N: ArrayLength, HTTP: HttpDownloadClientExt, CCH: CompressionChooserHandling<R, W>, ECH: EncryptionChooserHandler<R, W, N>, FDHB: FileDownloadHandlerBuilder<R, W, N, HTTP, CCH, ECH>>(
        &mut self,
        req: tonic::Request<DownloadBucketRequest>,
        keep_file_structure: bool,
        download_handler_builder: FDHB,
        api_token: &ApiToken,   http_client: HTTP,
    ) -> Result<Vec<String>, DownloadError>;
    /*
     * Upload to pre-signed url using PUT.
     */
    async fn upload_to_url_raw<
        R: Read,
        W: Write,
        N: ArrayLength,
        EM: EncryptionModule<R, W, N>,
        CM: CompressorModule<R, W>,
        BF: FileWrapper,
        HTTP: HttpUploadClientExt,
    >(
        url: &Url,
        chunk_size: u64,
        upload_handler: &mut BF,
        content_type: Mime,
        content_encoding: Option<Encoding>,
        http_client: HTTP,
    ) -> Result<u16, UploadToUrlError>;

    async fn get_bucket_details_raw(
        &mut self,
        req: Request<GetBucketDetailsRequest>,
    ) -> Result<GetBucketDetailsResponse, tonic::Status>;

    async fn get_bucket_details_from_url_raw(
        &mut self,
        share_link: ShareLink,
    ) -> Result<GetBucketDetailsFromUrlResponse, tonic::Status>;

    async fn move_files_in_bucket_raw(
        &mut self,
        req: Request<MoveFilesInBucketRequest>,
    ) -> Result<MoveFilesInBucketResponse, MoveFilesInBucketError>;

    async fn delete_files_in_bucket_raw(
        &mut self,
        req: Request<DeleteFilesInBucketRequest>,
    ) -> Result<DeleteFilesInBucketResponse, DeleteFileInBucketError>;

    async fn get_bucket_filestructure_raw(
        &mut self,
        req: Request<GetBucketFilestructureRequest>,
    ) -> Result<Vec<bucket_api::backend_api::File>, GetFilesystemDetailsError>;

}
