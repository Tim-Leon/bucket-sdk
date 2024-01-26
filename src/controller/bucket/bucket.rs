use std::error::Error;
use std::fmt::Debug;
use std::rc::Rc;

use bucket_common_types::exclusive_share_link::ExclusiveShareLink;
use bucket_common_types::share_link::ShareLink;
use bucket_common_types::DownloadFormat;

use byte_unit::Byte;
use gloo::{console::__macro::JsValue, net::http::Request};
use tokio::io::AsyncReadExt;
use tokio::io::BufReader;

use tokio_stream::StreamExt;

use crate::controller::bucket::download_handler::BucketFileDownloadHandler;
use crate::controller::bucket::download_handler::BucketFileWriter;
use crate::query_client::backend_api::DownloadFilesRequest;

use crate::query_client::{
    backend_api::{
        self, DeleteFilesInBucketRequest, GetBucketDetailsResponse, GetBucketFilestructureRequest,
        MoveFilesInBucketRequest, UploadFilesToBucketRequest,
    },
    backend_api::{DownloadBucketRequest, GetBucketDetailsRequest},
    QueryClient,
};

use crate::controller::bucket::errors::{
    DeleteFileInBucketError, DownloadError, FailedFilePaths, GetFilesystemDetailsError,
    MoveFilesInBucketError, UploadError, UploadToUrlError,
};

use crate::controller::bucket::upload_handler::BucketFileReader;
use crate::controller::bucket::upload_handler::BucketFileUploadHandler;
use crate::query_client::backend_api::{
    GetBucketDetailsFromUrlRequest, GetBucketDetailsFromUrlResponse,
};

use super::errors::BucketDownloadHandlerErrors;
use super::io::file::VirtualFileDetails;

pub type BucketFileUploadHandlerDyn = BucketFileReader;
pub type BucketFileDownloadHandlerDyn = BucketFileWriter;

pub struct UploadFileDescriptionState {
    pub file_path: String,
    pub size_in_bytes: u64,
    pub urls: Vec<String>,
}
//TODO: Most of these should just take a raw request instead of declearing the type and checking it. Do that in api.rs layer instead.

//TODO: Move to params to DTO layer.
pub async fn upload_files_to_bucket<FileHandle>(
    client: &mut QueryClient,
    req: UploadFilesToBucketRequest,
    mut upload_handler: BucketFileUploadHandlerDyn,
) -> Result<(), UploadError> {
    // let mut temp_source_files: Vec<backend_api::upload_files_to_bucket_request::File> =
    //     Vec::with_capacity(source_files.len());
    // for source_file in source_files {
    //     temp_source_files.push(backend_api::upload_files_to_bucket_request::File {
    //         file_path: source_file.path.clone(),
    //         size_in_bytes: source_file.size_in_bytes,
    //     });
    // }
    let temp_source_files_len = req.source_files.len();

    let resp = client.upload_files_to_bucket(req).await.unwrap();
    let body = resp.into_inner();

    let _size_in_bytes_limit = body.size_in_bytes_limit;
    let mut upload_files: Vec<UploadFileDescriptionState> =
        Vec::with_capacity(temp_source_files_len);
    for filepath in body.filepaths.clone() {
        upload_files.push(UploadFileDescriptionState {
            file_path: filepath.file_path,
            size_in_bytes: filepath.file_size_in_bytes,
            urls: filepath.upload_urls,
        });
    }
    /*
     * Uploading uses multipart presigned urls meaning each file is uploaded in a max of 5 GiB chunks to each URL.
     * Files over the limit of 5 GiB will be divided up into multiple uploads a.k.a multiple URL's.
     * Each file is divided up into URL's. Then each URL upload is divided up into chunks depending on the memory limitations.
     * Each chunk is uploaded to the presigned url in sequence until the full load has been uploaded.
     * Uploading more will lead to overwriting previous uploads, just don't.
     * Each URL is 5 GiB
     */
    for filepath in body.filepaths.into_iter() {
        let upload_urls = filepath.upload_urls.clone();
        for upload_url in upload_urls.clone() {
            let url = url::Url::parse(upload_url.as_str())?;
            let chunk_size =
                ((Byte::GIBIBYTE.as_u64() * 5) as u64) - filepath.file_size_in_bytes; //TODO: ???
                                                                                             //let chunk_size = GiB::from(1).to_bytes() as usize;
            upload_to_url(&url, chunk_size, &mut upload_handler)
                .await
                .unwrap();
        }
    }
    Ok(())
}

pub async fn download_from_url<DH: BucketFileDownloadHandler + Clone, T>(
    client: &mut QueryClient,
    url: ExclusiveShareLink,
    hashed_password: Option<String>,
    format: Option<DownloadFormat>,
    //download_handler: DH,
    create_download_handler: impl CreateFileDownloadHandler<DH, T>,
    additional_param: Rc<T>,
) -> Result<(), DownloadError> {
    let bucket_details = match url {
        //(user_id, bucket_id)
        ExclusiveShareLink::ShareLink(share) => {
            let detail = get_bucket_details_from_url(client, share)
                .await
                .map_err(DownloadError::GetBucketDetailsFromUrlRequestFailed)?;
            vec![detail.buckets.unwrap()]
        }
        ExclusiveShareLink::SecretShareLink(secret) => {
            let detail = get_bucket_details(
                client,
                GetBucketDetailsRequest {
                    opt_bucket_ids: vec![secret.bucket_id.to_string()],
                    bucket_owner_id: secret.user_id.to_string(),
                    offset: None,
                    limit: None,
                },
            )
            .await
            .map_err(|_x| DownloadError::BucketNotFound)?;

            detail.buckets
        }
    };
    if !bucket_details.is_empty() {
        return Err(DownloadError::BucketNotFound);
    }

    let detail = bucket_details.first();
    let detail = match detail {
        None => {
            return Err(DownloadError::BucketNotFound);
        }
        Some(x) => x,
    };

    let user_id = uuid::Uuid::try_parse(detail.owner_user_id.as_str())
        .map_err(DownloadError::ParseUserIdError)?;
    let bucket_id = uuid::Uuid::try_parse(detail.bucket_id.as_str())
        .map_err(DownloadError::ParseBucketIdError)?;

    let bucket_download_req = DownloadBucketRequest {
        bucket_id: bucket_id.to_string(),
        bucket_owner_id: user_id.to_string(),
        hashed_password,
        format: format.map(|x| x.to_string()),
    };

    bucket_download::<DH, T>(
        client,
        bucket_download_req,
        true,
        create_download_handler,
        additional_param,
    )
    .await?;
    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum DownloadFilesFromBucketError {
    #[error("on download finish error")]
    DownloadFinishError(Box<dyn Error + 'static>),
    #[error(transparent)]
    DownloadFromUrlError(#[from] DownloadFromUrlError),
}

impl From<BucketDownloadHandlerErrors> for DownloadFilesFromBucketError {
    fn from(value: BucketDownloadHandlerErrors) -> Self {
        DownloadFilesFromBucketError::DownloadFinishError(Box::new(value))
    }
}

pub trait CreateFileDownloadHandler<DH: BucketFileDownloadHandler, T> {
    fn handle(
        &self,
        virtual_file: super::io::file::VirtualFileDetails,
        keep_file_structure: bool,
        additional_param: Rc<T>,
    ) -> DH;
}

// Example implementation of CreateFileDownloadHandler for a specific closure type
impl<F, DH: BucketFileDownloadHandler, T> CreateFileDownloadHandler<DH, T> for F
where
    F: Fn(super::io::file::VirtualFileDetails, bool, Rc<T>) -> DH,
{
    fn handle(
        &self,
        virtual_file: super::io::file::VirtualFileDetails,
        keep_file_structure: bool,
        additional_param: Rc<T>,
    ) -> DH {
        self(virtual_file, keep_file_structure, additional_param)
    }
}

pub async fn download_files_from_bucket<DH: BucketFileDownloadHandler, T>(
    client: &mut QueryClient,
    req: DownloadFilesRequest,
    // Hook function to provide which file to write to.
    create_file_download_handler_hook: impl CreateFileDownloadHandler<DH, T>,
    jwt_token: String,
    keep_file_structure: bool,
    additional_param: Rc<T>,
) -> Result<(), DownloadFilesFromBucketError>
//where
    //Error: std::error::Error + Debug + Send + Sync + 'static,
    //DownloadFilesFromBucketError: From<Error>
{
    let mut resp_stream = client.download_files(req).await.unwrap().into_inner();

    while let Some(item) = resp_stream.next().await {
        match item {
            Ok(item) => {
                for file in item.filepaths {
                    let virtual_detail = VirtualFileDetails {
                        path: file.file_path,
                        date: None,
                        size_in_bytes: file.file_size_in_bytes,
                    };

                    let mut download_handler = create_file_download_handler_hook.handle(
                        virtual_detail,
                        keep_file_structure,
                        additional_param.clone(),
                    );
                    let url = url::Url::parse(file.download_url.as_str()).unwrap();
                    let mut size_left_in_bytes = file.file_size_in_bytes;
                    while size_left_in_bytes > 0 {
                        donwload_from_url::<DH>(
                            &url,
                            &mut size_left_in_bytes,
                            &mut download_handler,
                            jwt_token.clone().as_str(),
                        )
                        .await?;
                    }
                    download_handler.on_download_finish().map_err(
                        |err| -> DownloadFilesFromBucketError {
                            DownloadFilesFromBucketError::DownloadFinishError(Box::new(err))
                        },
                    )?;
                }
            }
            Err(_e) => {
                todo!()
            }
        }
    }
    Ok(())
}

// TODO: Move to params to DTO
pub async fn bucket_download<DH: BucketFileDownloadHandler, T>(
    client: &mut QueryClient,
    req: DownloadBucketRequest,
    keep_file_structure: bool, // Will keep the same file structure as on the server. This means directory/directory containing the file
    //download_handler: &mut BucketFileWriter,
    create_download_handler: impl CreateFileDownloadHandler<DH, T>,
    additional_param: Rc<T>,
) -> Result<Vec<String>, DownloadError> {
    let resp = client.download_bucket(req).await.unwrap();
    let mut res = resp.into_inner();
    let msg = res.message().await.unwrap().unwrap();
    for file in &msg.file.unwrap().filepaths {
        let url = url::Url::parse(file.download_url.as_str()).unwrap();
        let http_resp = Request::get(url.as_str()).send().await?;
        let resp_bin = http_resp.binary().await?;
        let virtual_file = VirtualFileDetails {
            path: file.file_path.clone(),
            date: None,
            size_in_bytes: file.file_size_in_bytes,
        };
        let mut download_handler = create_download_handler.handle(
            virtual_file,
            keep_file_structure,
            additional_param.clone(),
        );
        // let download_urls = file.download_urls.clone();
        // for download_url in download_urls {
        //     let url = url::Url::parse(download_url.as_str()).unwrap();
        //     let http_resp = Request::get(url.as_str())
        //         .body(file)
        //         .unwrap()
        //         .send()
        //         .await?; //TODO: Fix
        //     let resp_bin = http_resp.binary().await?; // TODO: Test? might need body instead. IDK
        //
        download_handler.on_download_chunk(&resp_bin).await.unwrap();
        //.map_err(|err| -> DownloadError {DownloadError::from(err)})?;
    }
    Ok(Vec::new())
}

/*
* Upload to pre-signed url using PUT.
*/
pub async fn upload_to_url(
    url: &url::Url,
    chunk_size: u64,
    upload_handler: &mut BucketFileReader,
) -> Result<u16, UploadToUrlError> {
    let file_chunk = upload_handler.on_upload_chunk(chunk_size).await?;
    //TypedArray::from
    let resp = Request::put(url.as_str())
        .header("Content-Type", "application/octet-stream")
        .body(Some(JsValue::from_str(std::str::from_utf8(
            file_chunk.as_slice(),
        )?)))?
        .send()
        .await?;
    if !resp.ok() {
        return Err(UploadToUrlError::HttpResponseStatusError(resp.status()));
    }
    Ok(resp.status())
}
#[derive(thiserror::Error, Debug)]
pub enum DownloadFromUrlError {
    #[error("Http response error code: {0}")]
    HttpResponseStatusError(u16),
    #[error("Empty body")]
    EmptyBody,
}
pub async fn donwload_from_url<DH: BucketFileDownloadHandler>(
    url: &url::Url,
    size_left_in_bytes: &mut u64,
    download_handler: &mut DH,
    jwt_token: &str,
) -> Result<u16, DownloadFromUrlError> {
    //https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateMultipartUpload.html
    let resp = Request::get(url.as_str())
        .header("Authorization", jwt_token)
        .send()
        .await
        .unwrap();
    if !resp.ok() {
        return Err(DownloadFromUrlError::HttpResponseStatusError(resp.status()));
    }
    let body = resp.binary().await.unwrap();
    let mut chunk = Vec::<u8>::new();
    let mut stream = BufReader::new(body.as_ref());
    while let Ok(size) = stream.read_buf(&mut chunk).await {
        if size == 0 {
            break;
        }
        download_handler.on_download_chunk(&chunk).await;
        *size_left_in_bytes -= size as u64;
        chunk.clear();
    }
    Ok(resp.status())
}

// TODO: Use parms in DTO.
async fn get_bucket_details(
    client: &mut QueryClient,
    req: GetBucketDetailsRequest,
) -> Result<GetBucketDetailsResponse, tonic::Status> {
    let resp = client.get_bucket_details(req).await?;
    Ok(resp.into_inner())
}

pub async fn get_bucket_details_from_url(
    client: &mut QueryClient,
    share_link: ShareLink,
) -> Result<GetBucketDetailsFromUrlResponse, tonic::Status> {
    let req = GetBucketDetailsFromUrlRequest {
        url: share_link.to_string(),
    };
    let resp = client.get_bucket_details_from_url(req).await?;
    Ok(resp.into_inner())
}

type AbsoluteBucketId = String;

async fn move_files_in_bucket(
    mut client: QueryClient,
    from_bucket_id: &uuid::Uuid,
    from_bucket_owner_id: &uuid::Uuid,
    to_bucket_id: &uuid::Uuid,
    to_bucket_owner_id: Option<uuid::Uuid>,
    from_filepaths: Vec<String>,
    to_directory: String,
) -> Result<(), MoveFilesInBucketError> {
    let req = MoveFilesInBucketRequest {
        from_bucket_id: from_bucket_id.to_string(),
        from_bucket_owner_id: from_bucket_owner_id.to_string(),
        to_bucket_id: to_bucket_id.to_string(),
        to_bucket_owner_id: to_bucket_owner_id.map(|id| id.to_string()),
        from_filepaths,
        to_directory,
        is_capacity_destructive: true,
    };
    let resp = client.move_files_in_bucket(req).await?.into_inner();
    if !resp.failed_file_paths.is_empty() {
        return Err(MoveFilesInBucketError::FailedToMoveFileFilepath(
            FailedFilePaths(resp.failed_file_paths),
        ));
    }
    Ok(())
}

impl From<Vec<String>> for DeleteFileInBucketError {
    fn from(failed_filepaths: Vec<String>) -> Self {
        DeleteFileInBucketError::FailedToDeleteFilepath(failed_filepaths)
    }
}

async fn delete_files_in_bucket(
    mut client: QueryClient,
    req: DeleteFilesInBucketRequest,
) -> Result<(), DeleteFileInBucketError> {
    let resp = client.delete_files_in_bucket(req).await?.into_inner();
    let failed_filepaths = resp.failed_file_paths;
    if !failed_filepaths.is_empty() {
        return Err(DeleteFileInBucketError::FailedToDeleteFilepath(
            failed_filepaths,
        ));
    }
    Ok(())
}

async fn get_filesystem_details(
    mut client: QueryClient,
    req: GetBucketFilestructureRequest,
) -> Result<Option<backend_api::Directory>, GetFilesystemDetailsError> {
    let resp = client.get_bucket_filestructure(req).await?.into_inner();
    let directory = match resp.filesystem {
        Some(filesystem) => filesystem.root,
        None => return Err(GetFilesystemDetailsError::EmptyFilesystem),
    };
    resp.continuation_token;
    Ok(directory)
}
