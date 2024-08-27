use std::io::{Read, Write};
use std::rc::Rc;
use bucket_api::backend_api;
use bucket_api::backend_api::{CreateBucketRequest, CreateBucketResponse, DeleteBucketRequest, DeleteBucketResponse, DeleteFilesInBucketRequest, DeleteFilesInBucketResponse, DownloadBucketRequest, DownloadFilesRequest, File, GetBucketDetailsRequest, GetBucketDetailsResponse, GetBucketFilestructureRequest, GetBucketFilestructureResponse, MoveFilesInBucketRequest, MoveFilesInBucketResponse, UpdateBucketRequest, UpdateBucketResponse, UploadFilesToBucketRequest};
use generic_array::ArrayLength;
use tonic::{IntoRequest, Request};
use crate::api::{BucketApiError, BucketClient};
use crate::client::http::{HttpDownloadClientExt, HttpUploadClientExt};
use crate::dto::bucket::{CreateBucketParams, DeleteBucketParams, DeleteFilesInBucketParams, DownloadBucketParams, DownloadFilesParams, GetBucketDetailsParams, GetFilesystemDetailsParams, MoveFilesInBucketParams, UpdateBucketParams, UploadFilesParams};
use crate::client::grpc::request_ext::RequestAuthorizationMetadataExt;
use crate::compression::CompressionChooserHandling;
use crate::encryption::EncryptionChooserHandler;
use crate::io::FileWrapper;
use crate::token::ContinuationToken;
use crate::wrapper::bucket::bucket::DownloadFilesFromBucketError;
use crate::wrapper::bucket::upload::FileUploadHandler;
use crate::wrapper::bucket::ClientUploadExt;
use crate::wrapper::bucket::download::FileDownloadHandlerBuilder;

impl<R: std::io::Read, W: std::io::Write> crate::api::ClientBucketExt<R, W> for BucketClient {
    async fn create_bucket(
        &mut self,
        param: CreateBucketParams,
    ) -> Result<CreateBucketResponse, BucketApiError> {
        let cbr: CreateBucketRequest = param.try_into().unwrap();
        let mut req = Request::new(cbr);
        req.set_authorization_metadata(&self.api_token);
        Ok(self.client.create_bucket(req).await.unwrap().into_inner())
    }

    async fn delete_bucket(
        &mut self,
        param: DeleteBucketParams,
    ) -> Result<DeleteBucketResponse, BucketApiError> {
        let dbr: DeleteBucketRequest = param.try_into().unwrap();
        let mut req = Request::new(dbr);
        req.set_authorization_metadata(&self.api_token);
        Ok(self.client.delete_bucket(req).await.unwrap().into_inner())
    }

    async fn update_bucket(
        &mut self,
        param: UpdateBucketParams,
    ) -> Result<UpdateBucketResponse, BucketApiError> {
        let ubr: UpdateBucketRequest = param.try_into().unwrap();
        let mut req = Request::new(ubr);
        req.set_authorization_metadata(&self.api_token);
        let resp = self.client.update_bucket(req).await.unwrap().into_inner();
        Ok(resp)
    }

    async fn get_bucket_details(
        &mut self,
        param: GetBucketDetailsParams,
    ) -> Result<GetBucketDetailsResponse, BucketApiError> {
        let gbdr: GetBucketDetailsRequest = param.try_into().unwrap();
        let mut req = Request::new(gbdr);
        req.set_authorization_metadata(&self.api_token);
        Ok(self
            .client
            .get_bucket_details(req)
            .await
            .unwrap()
            .into_inner())
    }

    async fn upload_files_to_bucket<File : FileWrapper, HTTP: HttpUploadClientExt>(
        &mut self,
        param: UploadFilesParams<File>,
        upload_file_handler: impl FileUploadHandler<R, W>,
        http_client: HTTP,
    ) -> Result<(), BucketApiError> {
        let uftbr: UploadFilesToBucketRequest = param.try_into().unwrap();
        let mut req = Request::new(uftbr);
        req.set_authorization_metadata(&self.api_token);
        self.client
            .upload_files_to_bucket_raw(req, upload_file_handler,&self.api_token, http_client)
            .await?;
        Ok(())
    }

    async fn download_files_from_bucket<N: ArrayLength,HTTP: HttpDownloadClientExt,CCH: CompressionChooserHandling<R, W>, ECH: EncryptionChooserHandler<R, W, N>,FDHB: FileDownloadHandlerBuilder<R, W, N,HTTP, CCH, ECH, >>(
        &mut self,
        param: DownloadFilesParams,
        file_download_handler_builder: FDHB
    )  -> Result<(), DownloadFilesFromBucketError> {
        let keep_file_structure = param.keep_file_structure;
        let dfr: DownloadFilesRequest = param.try_into().unwrap();
        let mut req = Request::new(dfr);
        req.set_authorization_metadata(&self.api_token);
        self.client
            .download_files_from_bucket_raw::<R, W, N, HTTP, CCH, ECH, FDHB>(
                req,
                file_download_handler_builder,
                &self.api_token,
                keep_file_structure,
            )
            .await?;
        Ok(())
    }

    async fn download_bucket<N:ArrayLength,HTTP: HttpDownloadClientExt,CCH: CompressionChooserHandling<R, W>, ECH: EncryptionChooserHandler<R, W, N>, FDHB: FileDownloadHandlerBuilder<R, W, N,HTTP, CCH, ECH>>(
        &mut self,
        param: DownloadBucketParams,
        file_download_handler_builder: FDHB,
        http_client: HTTP,
    ) -> Result<Vec<String>, BucketApiError> {
        let keep_file_structure = param.keep_file_structure;
        let dbr: DownloadBucketRequest = param.try_into().unwrap();
        let mut req = dbr.into_request();
        req.set_authorization_metadata(&self.api_token);
        let resp = self
            .client
            .download_bucket_raw::<R, W, N, HTTP, CCH, ECH, FDHB>(req, keep_file_structure, file_download_handler_builder, &self.api_token ,http_client)
            .await
            .unwrap();
        Ok(resp)
    }

    async fn move_files_in_bucket(
        &mut self,
        param: MoveFilesInBucketParams,
    ) -> Result<MoveFilesInBucketResponse, BucketApiError> {
        let mfibr: MoveFilesInBucketRequest = param.try_into()?;
        let mut req = Request::new(mfibr);
        req.set_authorization_metadata(&self.api_token);
        Ok(self
            .client
            .move_files_in_bucket_raw(req)
            .await
            .unwrap())
    }

    async fn delete_files_in_bucket(
        &mut self,
        param: DeleteFilesInBucketParams,
    ) -> Result<DeleteFilesInBucketResponse, BucketApiError> {
        let df: DeleteFilesInBucketRequest = param.try_into()?;
        let mut req = df.into_request();
        req.set_authorization_metadata(&self.api_token);
        let resp = self.client.delete_files_in_bucket_raw(req).await.unwrap();
        Ok(resp)
    }

    async fn get_bucket_filestructure_fully(
        &mut self,
        param: GetFilesystemDetailsParams,
    ) -> Result<Vec<backend_api::File>, BucketApiError> {
        let mut resulting_files: Vec<backend_api::File> = Vec::new();
        let mut continuation_token;
        loop {
            let res = self.get_bucket_filestructure(param.clone()).await?;
            // Append to filesturcture
            match res.filesystem {
                Some(filesystem) => {
                    if filesystem.files.len() <= 0 {
                        return Err(
                            BucketApiError::GetBucketDetailsRequestFullyResponseParsingError,
                        );
                    }
                    for f in filesystem.files {
                        resulting_files.push(f);
                    }
                }
                None => {}
            }
            if res.continuation_token.is_some() {
                param.continuation_token = res.continuation_token;
            } else {
                break;
            }
        }
        Ok(resulting_files)
    }

    async fn get_bucket_filestructure(
        &mut self,
        param: GetFilesystemDetailsParams,
    ) -> Result<(Vec<File>, Option<ContinuationToken>), BucketApiError> {
        let gfd: GetBucketFilestructureRequest = param.try_into()?;
        let mut req =gfd.into_request();
        req.set_authorization_metadata(&self.api_token);
        let resp = self.client.get_bucket_filestructure_raw(req).await.unwrap();
        let files = match resp.filesystem {
            Some(filesystem) => filesystem.files,
            None => return Err(BucketApiError::EmptyFilesystem),
        };
        let continuation_token = resp.continuation_token;
        Ok((files, continuation_token ))
    }
}