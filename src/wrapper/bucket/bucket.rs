use crate::client::grpc::native::client::query_client::QueryClient;
use crate::client::http::{HttpDownloadClientExt, HttpUploadClientExt};
use bucket_api::backend_api::{DeleteFilesInBucketRequest, DeleteFilesInBucketResponse, GetBucketDetailsFromUrlRequest, GetBucketDetailsFromUrlResponse, GetBucketDetailsRequest, GetBucketDetailsResponse, GetBucketFilestructureRequest, GetBucketFilestructureResponse, MoveFilesInBucketRequest, MoveFilesInBucketResponse};
use bucket_api::backend_api::{
    DownloadBucketRequest, DownloadFilesRequest, UploadFilesToBucketRequest,
};
use bucket_common_types::exclusive_share_link::ExclusiveShareLink;
use bucket_common_types::share_link::ShareLink;
use bucket_common_types::{BucketGuid, DownloadFormat, Encoding};
use byte_unit::Byte;
use futures::io::BufReader;
use futures::AsyncReadExt;
use futures::StreamExt;
use mime::Mime;
use std::error::Error;
use std::fmt::Debug;
use std::io::{Read, Write};
use std::rc::Rc;
use bucket_common_types::bucket_search_query::UuidStringUnion::Uuid;
use generic_array::ArrayLength;
use tonic::{IntoRequest, Request, Status};
use url::Url;
use zero_knowledge_encryption::encryption::aead::EncryptionModule;
//use tokio::io::BufReader;

//use tokio_stream::StreamExt;

use crate::compression::{CompressionChooserHandling, CompressorModule};
use crate::token::ApiToken;
use crate::wrapper::bucket::download::{FileDownloadHandler, FileDownloadHandlerBuilder};
use crate::wrapper::bucket::errors::{
    DeleteFileInBucketError, DownloadError, FailedFilePaths, GetFilesystemDetailsError,
    MoveFilesInBucketError, UploadError, UploadToUrlError,
};
use crate::wrapper::bucket::ClientUploadExt;

use crate::client::grpc::request_ext::RequestAuthorizationMetadataExt;
use crate::client::http::native::http::HttpClient;
use crate::encryption::EncryptionChooserHandler;
use crate::io::file::VirtualFileDetails;
use crate::wrapper::bucket::download::download_handler::BucketDownloadHandlerErrors;
use crate::wrapper::bucket::upload::FileUploadHandler;


impl ClientUploadExt for QueryClient {
    async fn upload_files_to_bucket_raw<R: std::io::Read, W: std::io::Write, UH: FileUploadHandler<R, W>, HTTP: HttpUploadClientExt>(
        &mut self,
        req: tonic::Request<UploadFilesToBucketRequest>,
        mut upload_handler: UH,
        api_token: &ApiToken,
        http_client: HTTP,
    ) -> Result<(), UploadError> {
        let total_upload_size: u64 = req
            .get_ref()
            .source_files
            .iter()
            .map(|file| file.size_in_bytes)
            .sum();
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

            let file_upload_handler = UH::new();
            for upload_url in upload_urls.clone() {
                let url = url::Url::parse(upload_url.as_str())?;
                let chunk_size = (Byte::GIBIBYTE.as_u64() * 5) - filepath.total_file_size_in_bytes;
                //let chunk_size = GiB::from(1).to_bytes() as usize;
                upload_task_set.spawn(async {
                    Self::upload_to_url_raw(
                        &url,
                        chunk_size,
                        &mut upload_handler,
                        mime::APPLICATION_OCTET_STREAM,
                        api_token,
                        None,
                        http_client
                    )
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

    async fn download_from_url_raw<R, W, N, HTTP: HttpDownloadClientExt, CCH: CompressionChooserHandling<R, W>, ECH:EncryptionChooserHandler<R, W, N>,FDHB: FileDownloadHandlerBuilder<R, W,N, HTTP, CCH, ECH>>(
        &mut self,
        api_token: &ApiToken,
        url: ExclusiveShareLink,
        hashed_password: Option<String>,
        format: Option<DownloadFormat>,
        create_download_handler: FDHB,
    )    -> Result<(), DownloadError> {
        let bucket_details = match url {
            //(user_id, bucket_id)
            ExclusiveShareLink::ShareLink(share) => {
                let detail = self
                    .get_bucket_details_from_url_raw(share)
                    .await
                    .map_err(|e| DownloadError::GetBucketDetailsFromUrlRequestFailed(e))?;
                vec![detail.buckets.unwrap()]
            }
            ExclusiveShareLink::SecretShareLink(secret) => {
                let req = GetBucketDetailsRequest {
                    opt_bucket_ids: vec![secret.bucket_id.to_string()],
                    bucket_owner_id: secret.user_id.to_string(),
                    offset: None,
                    limit: None,
                };
                let detail = self
                    .get_bucket_details_raw(req.into_request())
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
        let mut req = Request::new(bucket_download_req);
        req.set_authorization_metadata(api_token);

        self.download_bucket_raw(req, true, create_download_handler)
            .await?;
        Ok(())
    }

    async fn download_files_from_bucket_raw<R: Read,W: Write, N: ArrayLength,HTTP: HttpDownloadClientExt,CCH: CompressionChooserHandling<R, W>, ECH: EncryptionChooserHandler<R, W, N>,FDHB: FileDownloadHandlerBuilder<R, W, N,HTTP, CCH, ECH>>(
        &mut self,
        req: tonic::Request<DownloadFilesRequest>,
        file_download_handler_builder: FDHB,
        api_token: &ApiToken,
        keep_file_structure: bool,
    ) -> Result<(), DownloadFilesFromBucketError> {
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

                        let mut download_handler = file_download_handler_builder.handle(
                            virtual_detail,
                            keep_file_structure,
                        );
                        let url = url::Url::parse(file.download_url.as_str()).unwrap();
                        let mut size_left_in_bytes = file.file_size_in_bytes;
                        while size_left_in_bytes > 0 {
                            download_from_url(
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

    async fn download_bucket_raw<R: Read, W: Write, N: ArrayLength, HTTP: HttpDownloadClientExt, CCH: CompressionChooserHandling<R, W>, ECH: EncryptionChooserHandler<R, W, N>, FDHB: FileDownloadHandlerBuilder<R, W, N, HTTP, CCH, ECH>>(
        &mut self,
        req: tonic::Request<DownloadBucketRequest>,
        keep_file_structure: bool,
        download_handler_builder: FDHB,
        api_token: &ApiToken,   http_client: HTTP,
    ) -> Result<Vec<String>, DownloadError> {

        let bucket_owner_id = uuid::Uuid::try_parse(req.get_ref().bucket_owner_id.as_str());
        let bucket_id = uuid::Uuid::try_parse(req.get_ref().bucket_id.as_str());
        let bucket_guid = BucketGuid::new(bucket_owner_id.unwrap(), bucket_id.unwrap());

        let resp = self.download_bucket(req).await.unwrap();
        let target_path ;
        let mut res = resp.into_inner();
        let msg = res.message().await.unwrap().unwrap();
        for file in &msg.file.unwrap().filepaths {
            let url = url::Url::parse(file.download_url.as_str()).unwrap();
            let mut builder = FileDownloadHandlerBuilder::new(&bucket_guid,api_token,&target_path,&url, &http_client);

            let content_encoding= req.get_ref().format.unwrap();
            let http_resp = http_client.get(url, api_token,content_encoding).await.unwrap();
            let virtual_file = VirtualFileDetails {
                path: file.file_path.clone(),
                date: None,
                size_in_bytes: file.file_size_in_bytes,
                //file_format: mime::Mime::from_str(file.file_format.as_str())?,
            };

            let mut download_handler = create_download_handler.handle(
                virtual_file,
                keep_file_structure,
            );
            download_handler.on_download_chunk(&resp_bin).await.unwrap();
            //.map_err(|err| -> DownloadError {DownloadError::from(err)})?;
        }
        Ok(Vec::new())
    }

    async fn upload_to_url_raw<R: std::io::Read,W: std::io::Write,UH: FileUploadHandler<R,W>, HTTP: HttpUploadClientExt>(
        url: &Url,
        chunk_size: u64,
        upload_handler: &mut UH,
        content_type: Mime,
        api_token: &ApiToken,
        content_encoding: Option<Encoding>,
        http_client: HTTP,
    ) -> Result<u16, UploadToUrlError>
    {
        let file_chunk = upload_handler.on_upload_chunk(chunk_size).await.unwrap();
        let resp = http_client.put(url.clone(), file_chunk.as_slice(), api_token,content_type, content_encoding).await.unwrap();
        Ok()
    }

    async fn get_bucket_details_raw(
        &mut self,
        req: Request<GetBucketDetailsRequest>,
    ) -> Result<GetBucketDetailsResponse, Status> {
        let resp = self.get_bucket_details(req).await?;
        Ok(resp.into_inner())
    }

    async fn get_bucket_details_from_url_raw(
        &mut self,
        share_link: ShareLink,
    ) -> Result<GetBucketDetailsFromUrlResponse, Status> {
        let req = GetBucketDetailsFromUrlRequest {
            url: share_link.to_string(),
        };
        let resp = self.get_bucket_details_from_url(req).await?;
        Ok(resp.into_inner())
    }

    async fn move_files_in_bucket_raw(
        &mut self,
        req: Request<MoveFilesInBucketRequest>
    ) -> Result<MoveFilesInBucketResponse, MoveFilesInBucketError> {
        let resp = self.move_files_in_bucket(req).await?.into_inner();
        if !resp.failed_file_paths.is_empty() {
            return Err(MoveFilesInBucketError::FailedToMoveFileFilepath(
                FailedFilePaths(resp.failed_file_paths),
            ));
        }
        Ok(resp)
    }

    async fn delete_files_in_bucket_raw(
        &mut self,
        req: Request<DeleteFilesInBucketRequest>,
    ) -> Result<DeleteFilesInBucketResponse, DeleteFileInBucketError> {
        let res = self.delete_files_in_bucket(req).await?.into_inner();
        let failed_file_paths = &res.failed_file_paths;
        if !failed_file_paths.is_empty() {
            return Err(DeleteFileInBucketError::FailedToDeleteFilepath(
                failed_file_paths.clone(),
            ));
        }
        Ok(res)
    }

    async fn get_bucket_filestructure_raw(
        &mut self,
        req: Request<GetBucketFilestructureRequest>,
    ) -> Result<GetBucketFilestructureResponse, GetFilesystemDetailsError> {
        let resp = self.get_bucket_filestructure(req).await?.into_inner();
        Ok(resp)
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
#[derive(thiserror::Error, Debug)]
pub enum DownloadFromUrlError {
    #[error("Http response error code: {0}")]
    HttpResponseStatusError(u16),
    #[error("Empty body")]
    EmptyBody,
}

/// Uses HTTP client.
pub async fn download_from_url<R, W, N, HTTP: HttpDownloadClientExt, CCH: CompressionChooserHandling<R, W>, ECH: EncryptionChooserHandler<R, W, N>, FDHB: FileDownloadHandlerBuilder<R, W, N, HTTP, CCH, ECH>>(
    url: &url::Url,
    size_left_in_bytes: &mut u64,
    download_handler: &mut FDHB,
    api_token: &ApiToken,
) -> Result<u16, DownloadFromUrlError> {
    //https://docs.aws.amazon.com/AmazonS3/latest/API/API_CreateMultipartUpload.html


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


impl From<Vec<String>> for DeleteFileInBucketError {
    fn from(failed_file_paths: Vec<String>) -> Self {
        DeleteFileInBucketError::FailedToDeleteFilepath(failed_file_paths)
    }
}
