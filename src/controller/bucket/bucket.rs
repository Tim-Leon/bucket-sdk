use bucket_api::backend_api::{DeleteFilesInBucketRequest, File, GetBucketDetailsFromUrlRequest, GetBucketDetailsFromUrlResponse, GetBucketDetailsRequest, GetBucketDetailsResponse, GetBucketFilestructureRequest, MoveFilesInBucketRequest};
use std::error::Error;
use std::fmt::Debug;
use std::io::{Read, Write};
use std::rc::Rc;
use std::str::FromStr;
use bucket_api::backend_api::{DownloadBucketRequest, DownloadFilesRequest, UploadFilesToBucketRequest};
use bucket_common_types::exclusive_share_link::ExclusiveShareLink;
use bucket_common_types::share_link::ShareLink;
use bucket_common_types::{BucketCompression, DownloadFormat, Encoding};

use byte_unit::Byte;
use futures::io::BufReader;
use futures::AsyncReadExt;
use futures::StreamExt;
use gloo::{console::__macro::JsValue, net::http::Request};
use mime::Mime;
use tonic::Status;
use url::Url;
use uuid::Uuid;
//use tokio::io::BufReader;

//use tokio_stream::StreamExt;

use crate::api::{ApiToken};
use crate::client::query_client::QueryClient;
use crate::compression::CompressorModule;
use crate::controller::bucket::download_handler::BucketFileDownloadHandler;

use crate::controller::bucket::errors::{
    DeleteFileInBucketError, DownloadError, FailedFilePaths, GetFilesystemDetailsError,
    MoveFilesInBucketError, UploadError, UploadToUrlError,
};

use crate::controller::bucket::upload_handler::BucketFileReader;
use crate::controller::bucket::upload_handler::BucketFileUploadHandler;
use crate::http_request_ext::{RequestBuilderAuthorizationMetadataExt, RequestBuilderContentEncodingHeaderExt, RequestBuilderContentTypeHeaderExt};
use crate::request_ext::{RequestAuthorizationMetadataExt};
use super::download_handler::BucketDownloadHandlerErrors;
use super::io::file::VirtualFileDetails;

pub trait ClientUploadExt {
    /// Note: THe api token need to be set for the request in order for it to work.
    async fn upload_files_to_bucket_raw<R: std::io::Read,W: std::io::Write>(&mut self,
                                        req: tonic::Request<UploadFilesToBucketRequest>,
                                        upload_handler: impl BucketFileUploadHandler<R, W>) -> Result<(), UploadError>;
    async fn download_from_url_raw<T>(&mut self,api_token: &ApiToken,
                                   url: ExclusiveShareLink,
                                   hashed_password: Option<String>,
                                   format: Option<DownloadFormat>,
                                   //download_handler: DH,
                                   create_download_handler: impl CreateFileDownloadHandler<T>,
                                   additional_param: Rc<T>,
    ) -> Result<(), DownloadError>;

    async fn download_files_from_bucket_raw<T>(&mut self,
                             req: tonic::Request<DownloadFilesRequest>,
                             // Hook function to provide which file to write to.
                             create_file_download_handler_hook: impl CreateFileDownloadHandler<T>,
                             api_token: &ApiToken,
                             keep_file_structure: bool,
                             additional_param: Rc<T>,
    ) -> Result<(), DownloadFilesFromBucketError>;

    async fn download_bucket_raw<T>(&mut self,
        req: tonic::Request<DownloadBucketRequest>,
        keep_file_structure: bool, // Will keep the same file structure as on the server. This means directory/directory containing the file
        //download_handler: &mut BucketFileWriter,
        create_download_handler: impl CreateFileDownloadHandler<T>,
        additional_param: Rc<T>,
    ) -> Result<Vec<String>, DownloadError>;
    /*
    * Upload to pre-signed url using PUT.
    */
    async fn upload_to_url_raw<R: Read,W: Write>(
        url: &Url, chunk_size: u64, upload_handler: &mut BucketFileReader<R, W>, content_type: Mime, content_encoding: Option<Encoding>
    ) -> Result<u16, UploadToUrlError>;

    async fn get_bucket_details_raw(
        &mut self,
        req: GetBucketDetailsRequest,
    ) -> Result<GetBucketDetailsResponse, tonic::Status>;

    async fn get_bucket_details_from_url_raw(
        &mut self,
        share_link: ShareLink,
    ) -> Result<GetBucketDetailsFromUrlResponse, tonic::Status>;

    async fn move_files_in_bucket_raw(&mut self,
        from_bucket_id: &uuid::Uuid,
        from_bucket_owner_id: &uuid::Uuid,
        to_bucket_id: &uuid::Uuid,
        to_bucket_owner_id: Option<uuid::Uuid>,
        from_filepaths: Vec<String>,
        to_directory: String,
    ) -> Result<(), MoveFilesInBucketError>;

    async fn delete_files_in_bucket_raw(&mut self,
        req: DeleteFilesInBucketRequest,
    ) -> Result<(), DeleteFileInBucketError>;

    async fn get_filesystem_details_raw(&mut self,
        req: GetBucketFilestructureRequest,
    ) -> Result<Vec<bucket_api::backend_api::File>, GetFilesystemDetailsError>;

}



impl ClientUploadExt for QueryClient {
    async fn upload_files_to_bucket_raw<R: std::io::Read,W: std::io::Write>(&mut self, req: tonic::Request<UploadFilesToBucketRequest>, mut upload_handler: impl BucketFileUploadHandler<R, W>) -> Result<(), UploadError> {
        let total_upload_size: u64 = req.get_ref().source_files.iter().map(|file| file.size_in_bytes).sum();
        let temp_source_files_len = req.get_ref().source_files.len();

        let resp = self.upload_files_to_bucket(req).await.unwrap();
        let body = resp.into_inner();

        /// Will probably be caught by the backend with an error response before client can even check.
        if total_upload_size > body.size_in_bytes_limit {
            return Err(UploadError::StorageNotAvailable);
        }
        /*
         * Uploading uses multipart presigned urls meaning each file is uploaded in a max of 5 GiB chunks to each URL.
         * Files over the limit of 5 GiB will be divided up into multiple uploads a.k.a multiple URL's.
         * Each file is divided up into URL's. Then each URL upload is divided up into chunks depending on the memory limitations.
         * Each chunk is uploaded to the presigned url in sequence until the full load has been uploaded.
         * Uploading more will lead to overwriting previous uploads, just don't.
         * Each URL is 5 GiB
         */
        let mut upload_task_set = tokio::task::JoinSet::new();
        for filepath in body.filepaths.into_iter() {
            let upload_urls = filepath.upload_urls.clone();
            for upload_url in upload_urls.clone() {
                let url = url::Url::parse(upload_url.as_str())?;
                let chunk_size = (Byte::GIBIBYTE.as_u64() * 5) - filepath.total_file_size_in_bytes;
                //let chunk_size = GiB::from(1).to_bytes() as usize;
                upload_task_set.spawn(async {
                    Self::upload_to_url_raw(&url, chunk_size, &mut upload_handler.clone(), mime::APPLICATION_OCTET_STREAM, None)
                        .await
                        .unwrap()
                });
            }
        }
        while let Some(res) = upload_task_set.join_next().await {
            let out = res?;
            // ...
        }
        Ok(())
    }

    async fn download_from_url_raw<T>(&mut self, api_token: &ApiToken, url: ExclusiveShareLink, hashed_password: Option<String>, format: Option<DownloadFormat>, create_download_handler: impl CreateFileDownloadHandler<T>, additional_param: Rc<T>) -> Result<(), DownloadError> {
        let bucket_details = match url {
            //(user_id, bucket_id)
            ExclusiveShareLink::ShareLink(share) => {
                let detail = self.get_bucket_details_from_url_raw(share)
                    .await
                    .map_err(|e|DownloadError::GetBucketDetailsFromUrlRequestFailed(e))?;
                vec![detail.buckets.unwrap()]
            }
            ExclusiveShareLink::SecretShareLink(secret) => {
                let detail = self.get_bucket_details_raw(
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
        let mut req = tonic::Request::new(bucket_download_req);
        req.set_authorization_metadata(api_token);

        self.download_bucket_raw(
            req,
            true,
            create_download_handler,
            additional_param,
        )
            .await?;
        Ok(())
    }

    async fn download_files_from_bucket_raw<T>(&mut self, req: tonic::Request<DownloadFilesRequest>, create_file_download_handler_hook: impl CreateFileDownloadHandler<T>, api_token: &ApiToken, keep_file_structure: bool, additional_param: Rc<T>) -> Result<(), DownloadFilesFromBucketError> {
        let mut resp_stream = self.download_files(req).await.unwrap().into_inner();

        while let Some(item) = resp_stream.next().await {
            match item {
                Ok(item) => {
                    for file in item.filepaths {
                        let virtual_detail = VirtualFileDetails {
                            path: file.file_path,
                            date: None,
                            size_in_bytes: file.file_size_in_bytes,
                            //file_format: mime::Mime::from_str(file.file_format.as_str())?,
                        };

                        let mut download_handler = create_file_download_handler_hook.handle(
                            virtual_detail,
                            keep_file_structure,
                            additional_param.clone(),
                        );
                        let url = url::Url::parse(file.download_url.as_str()).unwrap();
                        let mut size_left_in_bytes = file.file_size_in_bytes;
                        while size_left_in_bytes > 0 {
                            donwload_from_url(
                                &url,
                                &mut size_left_in_bytes,
                                &mut download_handler,
                                api_token,
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

    async fn download_bucket_raw<T>(&mut self, req: tonic::Request<DownloadBucketRequest>, keep_file_structure: bool, create_download_handler: impl CreateFileDownloadHandler<T>, additional_param: Rc<T>) -> Result<Vec<String>, DownloadError> {
        let resp = self.download_bucket(req).await.unwrap();
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
                //file_format: mime::Mime::from_str(file.file_format.as_str())?,
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

    async fn upload_to_url_raw<R: std::io::Read,W: std::io::Write>(url: &Url, chunk_size: u64, upload_handler: &mut impl BucketFileUploadHandler<R, W>, content_type: Mime, content_encoding: Option<Encoding>) -> Result<u16, UploadToUrlError> {
        let file_chunk = upload_handler.on_upload_chunk(chunk_size).await?;
        //TypedArray::from
        let mut req = Request::put(url.as_str()).set_content_type(content_type);
        req = match content_encoding {
            None => { req }
            Some(encoding) => {
                req.set_content_encoding(&[encoding])
            }
        };

        let resp = req
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

    async fn get_bucket_details_raw(&mut self, req: GetBucketDetailsRequest) -> Result<GetBucketDetailsResponse, Status> {
        let resp = self.get_bucket_details(req).await?;
        Ok(resp.into_inner())
    }

    async fn get_bucket_details_from_url_raw(&mut self, share_link: ShareLink) -> Result<GetBucketDetailsFromUrlResponse, Status> {
        let req = GetBucketDetailsFromUrlRequest {
            url: share_link.to_string(),
        };
        let resp = self.get_bucket_details_from_url(req).await?;
        Ok(resp.into_inner())
    }

    async fn move_files_in_bucket_raw(&mut self, from_bucket_id: &Uuid, from_bucket_owner_id: &Uuid, to_bucket_id: &Uuid, to_bucket_owner_id: Option<Uuid>, from_filepaths: Vec<String>, to_directory: String) -> Result<(), MoveFilesInBucketError> {
        let req = MoveFilesInBucketRequest {
            from_bucket_id: from_bucket_id.to_string(),
            from_bucket_owner_id: from_bucket_owner_id.to_string(),
            to_bucket_id: to_bucket_id.to_string(),
            to_bucket_owner_id: to_bucket_owner_id.map(|id| id.to_string()),
            from_filepaths,
            to_directory,
            is_capacity_destructive: true,
        };
        let resp = self.move_files_in_bucket(req).await?.into_inner();
        if !resp.failed_file_paths.is_empty() {
            return Err(MoveFilesInBucketError::FailedToMoveFileFilepath(
                FailedFilePaths(resp.failed_file_paths),
            ));
        }
        Ok(())
    }

    async fn delete_files_in_bucket_raw(&mut self, req: DeleteFilesInBucketRequest) -> Result<(), DeleteFileInBucketError> {
        let resp = self.delete_files_in_bucket(req).await?.into_inner();
        let failed_file_paths = resp.failed_file_paths;
        if !failed_file_paths.is_empty() {
            return Err(DeleteFileInBucketError::FailedToDeleteFilepath(
                failed_file_paths,
            ));
        }
        Ok(())
    }

    async fn get_filesystem_details_raw(&mut self, req: GetBucketFilestructureRequest) -> Result<Vec<File>, GetFilesystemDetailsError> {
        let resp = self.get_bucket_filestructure(req).await?.into_inner();
        let files = match resp.filesystem {
            Some(filesystem) => filesystem.files,
            None => return Err(GetFilesystemDetailsError::EmptyFilesystem),
        };
        resp.continuation_token;
        Ok(files)
    }
}

pub struct UploadFileDescriptionState {
    pub file_path: String,
    pub size_in_bytes: u64,
    pub urls: Vec<String>,
}


#[derive(thiserror::Error, Debug)]
pub enum DownloadFilesFromBucketError {
    #[error("on download finish error")]
    DownloadFinishError(Box<dyn Error + 'static>),
    #[error(transparent)]
    DownloadFromUrlError(#[from] DownloadFromUrlError),
    #[error(transparent)]
    FromStrError(#[from] mime::FromStrError),
}

impl From<BucketDownloadHandlerErrors> for DownloadFilesFromBucketError {
    fn from(value: BucketDownloadHandlerErrors) -> Self {
        DownloadFilesFromBucketError::DownloadFinishError(Box::new(value))
    }
}
pub trait CreateFileDownloadHandler<T> {
    fn handle(
        &self,
        virtual_file: super::io::file::VirtualFileDetails,
        keep_file_structure: bool,
        additional_param: Rc<T>,
    ) -> impl BucketFileDownloadHandler;
}

// Example implementation of CreateFileDownloadHandler for a specific closure type
impl<F, T> CreateFileDownloadHandler<T> for F
where
    F: Fn(super::io::file::VirtualFileDetails, bool, Rc<T>) -> impl BucketFileDownloadHandler,
{
    fn handle(
        &self,
        virtual_file: super::io::file::VirtualFileDetails,
        keep_file_structure: bool,
        additional_param: Rc<T>,
    ) -> impl BucketFileDownloadHandler {
        self(virtual_file, keep_file_structure, additional_param)
    }
}


// TODO: Move to params to DTO
pub async fn bucket_download<T>(
    client: &mut QueryClient,
    req: tonic::Request<DownloadBucketRequest>,
    keep_file_structure: bool, // Will keep the same file structure as on the server. This means directory/directory containing the file
    //download_handler: &mut BucketFileWriter,
    create_download_handler: impl CreateFileDownloadHandler<T>,
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
            //file_format: mime::Mime::from_str(file.file_format.as_str())?,
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



#[derive(thiserror::Error, Debug)]
pub enum DownloadFromUrlError {
    #[error("Http response error code: {0}")]
    HttpResponseStatusError(u16),
    #[error("Empty body")]
    EmptyBody,
}
pub async fn donwload_from_url(
    url: &url::Url,
    size_left_in_bytes: &mut u64,
    download_handler: &mut impl BucketFileDownloadHandler,
    api_token: &ApiToken,
) -> Result<u16, DownloadFromUrlError> {
    //https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateMultipartUpload.html
    let resp = Request::get(url.as_str()).set_authorization_metadata(api_token)
        .send()
        .await
        .unwrap();

    if !resp.ok() {
        return Err(DownloadFromUrlError::HttpResponseStatusError(resp.status()));
    }
    let body = resp.binary().await.unwrap();
    let mut chunk = Vec::<u8>::new();
    let mut stream = BufReader::new(body.as_ref());
    while let Ok(size) = stream.read(&mut chunk).await {
        if size == 0 {
            break;
        }
        download_handler.on_download_chunk(&chunk).await.unwrap();
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
    fn from(failed_file_paths: Vec<String>) -> Self {
        DeleteFileInBucketError::FailedToDeleteFilepath(failed_file_paths)
    }
}

async fn delete_files_in_bucket(
    mut client: QueryClient,
    req: DeleteFilesInBucketRequest,
) -> Result<(), DeleteFileInBucketError> {
    let resp = client.delete_files_in_bucket(req).await?.into_inner();
    let failed_file_paths = resp.failed_file_paths;
    if !failed_file_paths.is_empty() {
        return Err(DeleteFileInBucketError::FailedToDeleteFilepath(
            failed_file_paths,
        ));
    }
    Ok(())
}

async fn get_filesystem_details(
    mut client: QueryClient,
    req: GetBucketFilestructureRequest,
) -> Result<Vec<bucket_api::backend_api::File>, GetFilesystemDetailsError> {
    let resp = client.get_bucket_filestructure(req).await?.into_inner();
    let files = match resp.filesystem {
        Some(filesystem) => filesystem.files,
        None => return Err(GetFilesystemDetailsError::EmptyFilesystem),
    };
    resp.continuation_token;
    Ok(files)
}
