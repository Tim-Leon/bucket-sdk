use tonic::metadata::{Ascii, Binary, MetadataValue};
use tonic::Request;

use crate::client;
use crate::client::query_client::backend_api;
use crate::controller::bucket::bucket::{
    bucket_download, download_files_from_bucket, upload_files_to_bucket, CreateFileDownloadHandler,
    DownloadFilesFromBucketError,
};
use crate::controller::bucket::download_handler::BucketFileDownloadHandler;

use crate::controller::account::authentication::register;
use crate::controller::account::errors::RegisterError;
use crate::controller::bucket::errors::{DownloadError, UploadError};
use crate::controller::bucket::upload_handler::BucketFileReader;

use crate::dto::dto::{
    CreateBucketParams, CreateBucketParamsParsingError, CreateBucketShareLinkParams,
    CreateBucketShareLinkParamsParsingError, CreateCheckoutParams,
    CreateCheckoutParamsParsingError, DeleteAccountParams, DeleteAccountParamsParsingError,
    DeleteBucketParams, DeleteFilesInBucketParams, DeleteFilesInBucketParamsParsingError,
    DownloadBucketParams, DownloadBucketParamsParsingError, DownloadFilesParams,
    DownloadFilesParamsParsingError, GetAccountDetailsParams, GetAccountDetailsParamsParsingError,
    GetBucketDetailsParams, GetBucketDetailsRequestParsingError, GetFilesystemDetailsParams,
    GetFilesystemDetailsParamsParsingError, MoveFilesInBucketParams,
    MoveFilesInBucketRequestParsingError, ParseDeleteBucketRequestError, UpdateAccountParams,
    UpdateAccountParamsParsingError, UpdateBucketParams, UpdateBucketParamsParsingError,
    UploadFilesParams, UploadFilesRequestParsingError,
};
use client::query_client::backend_api::{CreateBucketRequest, DownloadBucketRequest};
use client::query_client::backend_api::{
    CreateBucketResponse, CreateBucketShareLinkRequest, CreateBucketShareLinkResponse,
    CreateCheckoutRequest, CreateCheckoutResponse, DeleteAccountRequest, DeleteAccountResponse,
    DeleteBucketRequest, DeleteFilesInBucketRequest, DeleteFilesInBucketResponse,
    DownloadFilesRequest, GetAccountDetailsRequest, GetAccountDetailsResponse,
    GetBucketDetailsRequest, GetBucketDetailsResponse, GetBucketFilestructureRequest,
    GetBucketFilestructureResponse, MoveFilesInBucketRequest, MoveFilesInBucketResponse,
    UpdateAccountRequest, UpdateAccountResponse,
};
use client::query_client::backend_api::{
    UpdateBucketRequest, UpdateBucketResponse, UploadFilesToBucketRequest,
};
use client::query_client::QueryClient;
use std::rc::Rc;
use std::str::FromStr;

pub struct ApiToken(String);

pub struct BucketClient {
    client: QueryClient,
    api_token: String,
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

    // Response parsing error
    #[error("GetBucketDetailsRequestFullyResponseParsingError")]
    GetBucketDetailsRequestFullyResponseParsingError,
}

pub fn set_authorization_metadata<T>(api_token:&str, req: &mut Request<T>) {
    let meta = req.metadata_mut();
    let mut token: String = "Bearer ".to_string();
    token.push_str(api_token);
    let meta_data = MetadataValue::<Ascii>::from_str(token.as_str()).unwrap();
    meta.append("authorization", meta_data);
}

impl BucketClient {
    pub async fn new(api_url: &str, api_token: &str) -> Self {
        let client = QueryClient::build(api_url).await;
        BucketClient {
            client,
            api_token: api_token.to_string(),
        }
    }
    /// Uses enviorment variables:
    /// API_URL
    /// API_TOKEN
    pub async fn from_env() -> Self {
        let api_url = std::env::var("API_URL").unwrap();
        let api_token = std::env::var("API_TOKEN").unwrap();
        Self::new(&api_url, &api_token).await
    }

    pub async fn from_plaintext_credentials(
        api_url: &str,
        email: &email_address::EmailAddress,
        username: &str,
        password: &str,
        captcha: &str,
    ) -> Result<Self, RegisterError> {
        let mut client = QueryClient::build(api_url).await;
        let token = register(&mut client, email.as_str(), username, password, captcha).await?;
        Ok(Self {
            client: client,
            api_token: token.to_string(),
        })
    }


    pub fn set_authorization_metadata<T>(&self, req: &mut Request<T>) {
        set_authorization_metadata(&self.api_token, req);
    }

    pub async fn create_bucket(
        &mut self,
        _param: CreateBucketParams,
    ) -> Result<CreateBucketResponse, BucketApiError> {
        let cbr: CreateBucketRequest = _param.try_into()?;
        let mut req = Request::new(cbr);
        self.set_authorization_metadata(&mut req);
        Ok(self.client.create_bucket(req).await.unwrap().into_inner())
    }

    pub async fn delete_bucket(
        &mut self,
        _param: DeleteBucketParams,
    ) -> Result<backend_api::DeleteBucketResponse, BucketApiError> {
        let dbr: DeleteBucketRequest = _param.try_into()?;
        let mut req = Request::new(dbr);
        self.set_authorization_metadata(&mut req);
        Ok(self.client.delete_bucket(req).await.unwrap().into_inner())
    }

    pub async fn update_bucket(
        &mut self,
        _param: UpdateBucketParams,
    ) -> Result<UpdateBucketResponse, BucketApiError> {
        let ubr: UpdateBucketRequest = _param.try_into()?;
        let mut req = Request::new(ubr);
        self.set_authorization_metadata(&mut req);
        let resp = self.client.update_bucket(req).await.unwrap().into_inner();
        Ok(resp)
    }

    pub async fn get_bucket_details(
        &mut self,
        _param: GetBucketDetailsParams,
    ) -> Result<GetBucketDetailsResponse, BucketApiError> {
        let gbdr: GetBucketDetailsRequest = _param.try_into()?;
        let mut req = Request::new(gbdr);
        self.set_authorization_metadata(&mut req);
        Ok(self
            .client
            .get_bucket_details(req)
            .await
            .unwrap()
            .into_inner())
    }
    pub async fn upload_files_to_bucket(
        &mut self,
        param: UploadFilesParams,
        upload_file_handler: BucketFileReader,
    ) -> Result<(), BucketApiError> {
        let uftbr: UploadFilesToBucketRequest = param.try_into()?;
        let mut req = Request::new(uftbr);
        self.set_authorization_metadata(&mut req);
        upload_files_to_bucket(&mut self.client, req, upload_file_handler).await?;
        Ok(())
    }

    ///https://repost.aws/questions/QUxynkZDbASDaqrUcpx_sILQ/s3-support-multiple-byte-ranges-download
    pub async fn download_files_from_bucket<DH: BucketFileDownloadHandler, T>(
        &mut self,
        _param: DownloadFilesParams,
        //file_handle: BucketFileTrait<Error = BucketFileError, FileHandle = FileHandle>,
        // Hook function will take in the details for the file and either return a WebBucketFile or NativeBucketFile depending on enviorment implementation, diffrent between WASM and NATIVE.
        create_file_download_handler: impl CreateFileDownloadHandler<DH, T>,
        //file_choser: T where T impl (VirtualFileDetails, String) -> VirtualBucketFile,
        jwt_token: String,
        keep_file_structure: bool,
        additional_param: Rc<T>,
    ) -> Result<(), BucketApiError> {
        let dfr: DownloadFilesRequest = _param.try_into()?;
        let mut req = Request::new(dfr);
        self.set_authorization_metadata(&mut req);
        download_files_from_bucket::<DH, T>(
            &mut self.client,
            req,
            create_file_download_handler,
            jwt_token,
            keep_file_structure,
            additional_param,
        )
        .await?;
        Ok(())
    }

    pub async fn bucket_download<DH: BucketFileDownloadHandler, T>(
        &mut self,
        param: DownloadBucketParams,
        create_file_download_handler: impl CreateFileDownloadHandler<DH, T>,
        additional_param: Rc<T>,
    ) -> Result<Vec<String>, BucketApiError> {
        let keep_file_structure = param.keep_file_structure;
        let dbr: DownloadBucketRequest = param.try_into()?;
        let mut req = Request::new(dbr);
        self.set_authorization_metadata(&mut req);

        let res = bucket_download::<DH, T>(
            &mut self.client,
            req,
            keep_file_structure,
            //&mut download_handler,
            create_file_download_handler,
            additional_param,
        )
        .await?;

        Ok(res)
    }

    pub async fn move_files_in_bucket(
        &mut self,
        _param: MoveFilesInBucketParams,
    ) -> Result<MoveFilesInBucketResponse, BucketApiError> {
        let mfibr: MoveFilesInBucketRequest = _param.try_into()?;
        let mut req = Request::new(mfibr);
        self.set_authorization_metadata(&mut req);
        Ok(self
            .client
            .move_files_in_bucket(req)
            .await
            .unwrap()
            .into_inner())
    }

    pub async fn delete_files_in_bucket(
        &mut self,
        _param: DeleteFilesInBucketParams,
    ) -> Result<DeleteFilesInBucketResponse, BucketApiError> {
        let dfibr: DeleteFilesInBucketRequest = _param.try_into()?;
        let mut req = Request::new(dfibr);
        self.set_authorization_metadata(&mut req);
        Ok(self
            .client
            .delete_files_in_bucket(req)
            .await
            .unwrap()
            .into_inner())
    }
    /*
    what if we stop using file structure with subdirectory.
    and instead uses response message with target directory and store the full filename. Also if filename becomes too long, we just send a token and offset for the next request.
     */
    pub async fn get_bucket_filestructure_fully(
        &mut self,
        mut param: GetFilesystemDetailsParams,
    ) -> Result<Vec<backend_api::File>, BucketApiError> {
        let mut resulting_files: Vec<backend_api::File> = Vec::new();
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
                //param.start_directory = None;
                param.continuation_token = res.continuation_token;
            } else {
                break;
            }
        }
        Ok(resulting_files)
    }
    // Prefere using get_bucket_filestructure_fully() when trying to get fullstructure.
    pub async fn get_bucket_filestructure(
        &mut self,
        _param: GetFilesystemDetailsParams,
    ) -> Result<GetBucketFilestructureResponse, BucketApiError> {
        let gbfs: GetBucketFilestructureRequest = _param.try_into()?;
        let mut req = Request::new(gbfs);
        self.set_authorization_metadata(&mut req);
        Ok(self
            .client
            .get_bucket_filestructure(req)
            .await
            .unwrap()
            .into_inner())
    }

    pub async fn update_account(
        &mut self,
        _param: UpdateAccountParams,
    ) -> Result<UpdateAccountResponse, BucketApiError> {
        let ua: UpdateAccountRequest = _param.try_into()?;
        let mut req = Request::new(ua);
        self.set_authorization_metadata(&mut req);
        Ok(self.client.update_account(req).await.unwrap().into_inner())
    }

    pub async fn delete_account(
        &mut self,
        _param: DeleteAccountParams,
    ) -> Result<DeleteAccountResponse, BucketApiError> {
        let dar: DeleteAccountRequest = _param.try_into()?;
        let mut req = Request::new(dar);
        self.set_authorization_metadata(&mut req);
        Ok(self.client.delete_account(req).await.unwrap().into_inner())
    }

    pub async fn get_account_details(
        &mut self,
        _param: GetAccountDetailsParams,
    ) -> Result<GetAccountDetailsResponse, BucketApiError> {
        let gadr: GetAccountDetailsRequest = _param.try_into()?;
        let mut req = Request::new(gadr);
        self.set_authorization_metadata(&mut req);
        Ok(self
            .client
            .get_account_details(req)
            .await
            .unwrap()
            .into_inner())
    }

    pub async fn create_checkout(
        &mut self,
        _param: CreateCheckoutParams,
    ) -> Result<CreateCheckoutResponse, BucketApiError> {
        let ccr: CreateCheckoutRequest = _param.try_into()?;
        let mut req = Request::new(ccr);
        self.set_authorization_metadata(&mut req);
        Ok(self.client.create_checkout(req).await.unwrap().into_inner())
    }

    pub async fn create_bucket_share_link(
        &mut self,
        _param: CreateBucketShareLinkParams,
    ) -> Result<CreateBucketShareLinkResponse, BucketApiError> {
        let cbslr: CreateBucketShareLinkRequest = _param.try_into()?;
        let mut req = Request::new(cbslr);
        self.set_authorization_metadata(&mut req);
        Ok(self
            .client
            .create_bucket_share_link(req)
            .await
            .unwrap()
            .into_inner())
    }
}
