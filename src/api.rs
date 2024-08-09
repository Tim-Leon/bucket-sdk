use std::fmt::Display;
use tonic::metadata::{Ascii, Binary, MetadataValue};
use tonic::{IntoRequest, Request};

use crate::client;
use crate::controller::bucket::bucket::{bucket_download, CreateFileDownloadHandler, DownloadFilesFromBucketError, ClientUploadExt};
use crate::controller::bucket::download_handler::BucketFileDownloadHandler;

use crate::controller::account::authentication::{AuthenticationClientExt, JwtToken, login, LoginParams, register};
use crate::controller::account::errors::{LoginError, RegistrationError};
use crate::controller::bucket::errors::{DownloadError, UploadError};
use crate::controller::bucket::upload_handler::{BucketFileReader, BucketFileUploadHandler};

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


// use client::query_client::backend_api::{CreateBucketRequest, DownloadBucketRequest};
// use client::query_client::backend_api::{
//     CreateBucketResponse, CreateBucketShareLinkRequest, CreateBucketShareLinkResponse,
//     CreateCheckoutRequest, CreateCheckoutResponse, DeleteAccountRequest, DeleteAccountResponse,
//     DeleteBucketRequest, DeleteFilesInBucketRequest, DeleteFilesInBucketResponse,
//     DownloadFilesRequest, GetAccountDetailsRequest, GetAccountDetailsResponse,
//     GetBucketDetailsRequest, GetBucketDetailsResponse, GetBucketFilestructureRequest,
//     GetBucketFilestructureResponse, MoveFilesInBucketRequest, MoveFilesInBucketResponse,
//     UpdateAccountRequest, UpdateAccountResponse,
// };
// use client::query_client::backend_api::{
//     UpdateBucketRequest, UpdateBucketResponse, UploadFilesToBucketRequest,
// };
use client::query_client::QueryClient;
use std::rc::Rc;
use std::str::FromStr;
use bucket_api::backend_api;
use bucket_api::backend_api::{CreateBucketRequest, CreateBucketResponse, DeleteBucketRequest, DeleteBucketResponse, DeleteFilesInBucketRequest, DeleteFilesInBucketResponse, DownloadBucketRequest, DownloadFilesRequest, GetBucketDetailsRequest, GetBucketDetailsResponse, GetBucketFilestructureRequest, GetBucketFilestructureResponse, MoveFilesInBucketRequest, MoveFilesInBucketResponse, UpdateBucketRequest, UpdateBucketResponse, UploadFilesToBucketRequest};
use email_address::EmailAddress;
use mime::Mime;
use tonic::transport::Uri;
use crate::client::QueryClientBuilder;
use crate::request_ext::RequestAuthorizationMetadataExt;

#[derive(Clone, Debug, PartialEq)]
pub struct ApiToken(String); //TODO: JWT Token

impl TryFrom<&str> for ApiToken {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self { 0: value.to_string() })
    }
}

impl From<JwtToken> for ApiToken {
    fn from(value: JwtToken) -> Self {
        Self {
            0: value,
        }
    }
}

impl Display for ApiToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
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

    #[error(transparent)]
    ProtobufRequestFailed(#[from] tonic::Status),

}


pub trait BucketClientBuilder: Sized {
    async fn from_token(api_url: Uri, api_token: ApiToken) -> Self;
    async fn from_env() -> Self;
    async fn plaintext_credentials_registration(api_url: Uri,
                                        email: &email_address::EmailAddress,
                                        username: &str,
                                        password: &str,
                                        captcha: &str) -> Result<Self, RegistrationError>;
    async fn plaintext_credentials_login(api_url: Uri, login_params: &LoginParams) -> Result<Self, LoginError>;
}

impl BucketClientBuilder for BucketClient {
    async fn from_token(api_url: Uri, api_token: ApiToken) -> Self {
        let client = QueryClient::build(api_url).await;
        BucketClient {
            client,
            api_token,
        }
    }

    /// Uses environment variables:
    /// API_URL
    /// API_TOKEN
    async fn from_env() -> Self {
        let api_url = std::env::var("API_URL").unwrap();
        let api_token = std::env::var("API_TOKEN").unwrap();
        let client = QueryClient::build(Uri::from_str(api_url.as_str()).unwrap()).await;

        Self {
            client,
            api_token: ApiToken::try_from(api_token.as_str()).unwrap(),
        }
    }

    async fn plaintext_credentials_registration(api_url: Uri, email: &EmailAddress, username: &str, password: &str, captcha: &str) -> Result<Self, RegistrationError> {
        let mut client = QueryClient::build(api_url).await;
        let api_token = register(&mut client, email, username, password, captcha).await?;
        Ok(Self {
            client,
            api_token,
        })
    }

    async fn plaintext_credentials_login(api_url: Uri, login_params: &LoginParams) -> Result<Self, LoginError> {
        let mut client = QueryClient::build(api_url).await;
        let api_token = client.login(login_params).await?;
        Ok(BucketClient { client, api_token })
    }
}



pub trait ClientBucketExt<R: std::io::Read,W: std::io::Write> {
    async fn create_bucket(&mut self, param:CreateBucketParams) -> Result<CreateBucketResponse, BucketApiError>;
    async fn delete_bucket(&mut self, param: DeleteBucketParams) -> Result<DeleteBucketResponse, BucketApiError>;

    async fn update_bucket(
        &mut self,
        param: UpdateBucketParams,
    ) -> Result<UpdateBucketResponse, BucketApiError>;

    async fn get_bucket_details(
        &mut self,
        param: GetBucketDetailsParams,
    ) -> Result<GetBucketDetailsResponse, BucketApiError>;

    async fn upload_files_to_bucket(
        &mut self,
        param: UploadFilesParams,
        upload_file_handler: impl BucketFileUploadHandler<R, W>,
    ) -> Result<(), BucketApiError>;
    ///https://repost.aws/questions/QUxynkZDbASDaqrUcpx_sILQ/s3-support-multiple-byte-ranges-download
    async fn download_files_from_bucket<T>(
        &mut self,
        param: DownloadFilesParams,
        //file_handle: BucketFileTrait<Error = BucketFileError, FileHandle = FileHandle>,
        // Hook function will take in the details for the file and either return a WebBucketFile or NativeBucketFile depending on enviorment implementation, diffrent between WASM and NATIVE.
        create_file_download_handler: impl CreateFileDownloadHandler<T>,
        //file_choser: T where T impl (VirtualFileDetails, String) -> VirtualBucketFile,
        additional_param: Rc<T>,
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
    async fn download_bucket<T>(
        &mut self,
        param: DownloadBucketParams,
                             create_file_download_handler: impl CreateFileDownloadHandler<T>,
                             additional_param: Rc<T>,
    ) -> Result<Vec<String>, BucketApiError>;

    async fn move_files_in_bucket(
        &mut self,
        param: MoveFilesInBucketParams,
    ) -> Result<MoveFilesInBucketResponse, BucketApiError>;

    async fn delete_files_in_bucket(
        &mut self,
        param: DeleteFilesInBucketParams,
    ) -> Result<DeleteFilesInBucketResponse, BucketApiError> ;


    async fn get_bucket_filestructure_fully(
        &mut self,
         param: GetFilesystemDetailsParams,
    ) -> Result<Vec< backend_api::File>, BucketApiError>;

    async fn get_bucket_filestructure(
        &mut self,
        param: GetFilesystemDetailsParams,
    ) -> Result<GetBucketFilestructureResponse, BucketApiError>;
}


impl<R: std::io::Read,W: std::io::Write> ClientBucketExt<R, W> for BucketClient {
    async fn create_bucket(&mut self, param: CreateBucketParams) -> Result<CreateBucketResponse, BucketApiError> {
        let cbr: CreateBucketRequest = param.try_into().unwrap();
        let mut req = Request::new(cbr);
        req.set_authorization_metadata(&self.api_token);
        Ok(self.client.create_bucket(req).await.unwrap().into_inner())
    }

    async fn delete_bucket(&mut self, param: DeleteBucketParams) -> Result<DeleteBucketResponse, BucketApiError> {
        let dbr: DeleteBucketRequest = param.try_into().unwrap();
        let mut req = Request::new(dbr);
        req.set_authorization_metadata(&self.api_token);
        Ok(self.client.delete_bucket(req).await.unwrap().into_inner())

    }

    async fn update_bucket(&mut self, param: UpdateBucketParams) -> Result<UpdateBucketResponse, BucketApiError> {
        let ubr: UpdateBucketRequest = param.try_into().unwrap();
        let mut req = Request::new(ubr);
        req.set_authorization_metadata(&self.api_token);
        let resp = self.client.update_bucket(req).await.unwrap().into_inner();
        Ok(resp)
    }

    async fn get_bucket_details(&mut self, param: GetBucketDetailsParams) -> Result<GetBucketDetailsResponse, BucketApiError> {
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

    async fn upload_files_to_bucket(&mut self, param: UploadFilesParams, upload_file_handler: impl BucketFileUploadHandler<R, W>) -> Result<(), BucketApiError> {
        let uftbr: UploadFilesToBucketRequest = param.try_into()?;
        let mut req = Request::new(uftbr);
        req.set_authorization_metadata(&self.api_token);
        self.client.upload_files_to_bucket_raw(req, upload_file_handler).await?;
        Ok(())
    }

    async fn download_files_from_bucket<T>(&mut self, param: DownloadFilesParams, create_file_download_handler: impl CreateFileDownloadHandler<T>, additional_param: Rc<T>) -> Result<(), BucketApiError> {
        let keep_file_structure = param.keep_file_structure;
        let dfr: DownloadFilesRequest = param.try_into()?;
        let mut req = Request::new(dfr);
        req.set_authorization_metadata(&self.api_token);
        self.client.download_files_from_bucket_raw::<T>(
            req,
            create_file_download_handler,
            &self.api_token,
            keep_file_structure,
            additional_param,
        )
            .await?;
        Ok(())
    }

    async fn download_bucket<T>(&mut self, param: DownloadBucketParams, create_file_download_handler: impl CreateFileDownloadHandler<T>, additional_param: Rc<T>) -> Result<Vec<String>, BucketApiError> {
        let keep_file_structure = param.keep_file_structure;
        let dbr: DownloadBucketRequest = param.try_into().unwrap();
        let mut req = dbr.into_request();
        req.set_authorization_metadata(&self.api_token);
        let resp = self.client.download_bucket_raw(req, keep_file_structure,create_file_download_handler, additional_param).await.unwrap();
        Ok(resp)
    }

    async fn move_files_in_bucket(&mut self, param: MoveFilesInBucketParams) -> Result<MoveFilesInBucketResponse, BucketApiError> {
        todo!()
    }

    async fn delete_files_in_bucket(&mut self, param: DeleteFilesInBucketParams) -> Result<DeleteFilesInBucketResponse, BucketApiError> {
        todo!()
    }

    async fn get_bucket_filestructure_fully(&mut self, param: GetFilesystemDetailsParams) -> Result<Vec<backend_api::File>, BucketApiError> {
        todo!()
    }

    async fn get_bucket_filestructure(&mut self, param: GetFilesystemDetailsParams) -> Result<GetBucketFilestructureResponse, BucketApiError> {
        todo!()
    }
}


impl BucketClient {





    pub async fn bucket_download<DH: BucketFileDownloadHandler, T>(
        &mut self,
        param: DownloadBucketParams,
        create_file_download_handler: impl CreateFileDownloadHandler<T>,
        additional_param: Rc<T>,
    ) -> Result<Vec<String>, BucketApiError> {
        let keep_file_structure = param.keep_file_structure;
        let dbr: DownloadBucketRequest = param.try_into()?;
        let mut req = Request::new(dbr);
        req.set_authorization_metadata(&self.api_token);

        let res = bucket_download::<T>(
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
        req.set_authorization_metadata(&self.api_token);
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
        req.set_authorization_metadata(&self.api_token);
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
        req.set_authorization_metadata(&self.api_token);
        Ok(self
            .client
            .get_bucket_filestructure(req)
            .await
            .unwrap()
            .into_inner())
    }
}
