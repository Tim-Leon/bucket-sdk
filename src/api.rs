use crate::controller::bucket::bucket::{
    bucket_download, download_files_from_bucket,
    CreateFileDownloadHandler,
};
use crate::controller::bucket::download_handler::BucketFileDownloadHandler;


use crate::query_client::backend_api::{UpdateBucketRequest, UpdateBucketResponse};
use crate::query_client::QueryClient;
use crate::{
    dto::dto::*,
    query_client::backend_api::{
        CreateBucketRequest, CreateBucketResponse, CreateBucketShareLinkRequest,
        CreateBucketShareLinkResponse, CreateCheckoutRequest, CreateCheckoutResponse,
        DeleteAccountRequest, DeleteAccountResponse, DeleteBucketRequest,
        DeleteFilesInBucketRequest, DeleteFilesInBucketResponse, DownloadFilesRequest, GetAccountDetailsRequest, GetAccountDetailsResponse,
        GetBucketDetailsRequest, GetBucketDetailsResponse, GetBucketFilestructureRequest,
        GetBucketFilestructureResponse, MoveFilesInBucketRequest, MoveFilesInBucketResponse,
        UpdateAccountRequest, UpdateAccountResponse,
    },
};
use std::rc::Rc;
use std::str::FromStr;

pub struct ApiToken(String);

pub struct BucketApi {
    client: QueryClient,
    api_token: String,
}

impl BucketApi {
    pub fn new(api_url: &url::Url, api_token: &str) -> Self {
        let client = QueryClient::build(api_url);
        BucketApi {
            client,
            api_token: api_token.to_string(),
        }
    }
    /// Uses enviorment variables:
    /// API_URL
    /// API_TOKEN
    pub fn from_env() -> Self {
        let api_url = url::Url::from_str(std::env::var("API_URL").unwrap().as_str()).unwrap();
        let api_token = std::env::var("API_TOKEN").unwrap();
        Self::new(&api_url, &api_token)
    }

    pub async fn create_bucket(
        &mut self,
        _param: CreateBucketParams,
    ) -> Result<CreateBucketResponse, BucketApiError> {
        let req: CreateBucketRequest = _param.try_into()?;
        Ok(self.client.create_bucket(req).await.unwrap().into_inner())
    }

    pub async fn delete_bucket(
        &mut self,
        _param: DeleteBucketParams,
    ) -> Result<crate::query_client::backend_api::DeleteBucketResponse, BucketApiError> {
        let req: DeleteBucketRequest = _param.try_into()?;
        Ok(self.client.delete_bucket(req).await.unwrap().into_inner())
    }

    pub async fn update_bucket(
        &mut self,
        _param: UpdateBucketParams,
    ) -> Result<UpdateBucketResponse, BucketApiError> {
        let req: UpdateBucketRequest = _param.try_into()?;
        let resp = self.client.update_bucket(req).await.unwrap().into_inner();
        Ok(resp)
    }

    pub async fn get_bucket_details(
        &mut self,
        _param: GetBucketDetailsParams,
    ) -> Result<GetBucketDetailsResponse, BucketApiError> {
        let req: GetBucketDetailsRequest = _param.try_into()?;
        Ok(self
            .client
            .get_bucket_details(req)
            .await
            .unwrap()
            .into_inner())
    }
    // pub async fn upload_files_to_bucket<EncryptionModuleError>(
    //     &mut self,
    //     _param: UploadFilesParams,
    //     encryption_module: Box<dyn EncryptionModule<Error = EncryptionModuleError>>,
    // ) -> Result<(), BucketApiError> {
    //     let req: UploadFilesToBucketRequest = _param.try_into()?;
    //     for file in _param.source_files {
    //         let upload_handler = BucketFileReader {
    //             read_target_file: file.source_file.get_filehandle(),
    //             encryption_module: encryption_module.clone(),
    //             offset: 0,
    //         };
    //         upload_files_to_bucket(&mut self.client, req, upload_handler).await?;
    //     }
    //     Ok(())
    // }

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
        let req: DownloadFilesRequest = _param.try_into()?;
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
        let req = param.try_into()?;

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
        let req: MoveFilesInBucketRequest = _param.try_into()?;
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
        let req: DeleteFilesInBucketRequest = _param.try_into()?;
        Ok(self
            .client
            .delete_files_in_bucket(req)
            .await
            .unwrap()
            .into_inner())
    }

    pub async fn get_bucket_filestructure(
        &mut self,
        _param: GetFilesystemDetailsParams,
    ) -> Result<GetBucketFilestructureResponse, BucketApiError> {
        let req: GetBucketFilestructureRequest = _param.try_into()?;
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
        let req: UpdateAccountRequest = _param.try_into()?;
        Ok(self.client.update_account(req).await.unwrap().into_inner())
    }

    pub async fn delete_account(
        &mut self,
        _param: DeleteAccountParams,
    ) -> Result<DeleteAccountResponse, BucketApiError> {
        let req: DeleteAccountRequest = _param.try_into()?;
        Ok(self.client.delete_account(req).await.unwrap().into_inner())
    }

    pub async fn get_account_details(
        &mut self,
        _param: GetAccountDetailsParams,
    ) -> Result<GetAccountDetailsResponse, BucketApiError> {
        let req: GetAccountDetailsRequest = _param.try_into()?;
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
        let req: CreateCheckoutRequest = _param.try_into()?;
        Ok(self.client.create_checkout(req).await.unwrap().into_inner())
    }

    pub async fn create_bucket_share_link(
        &mut self,
        _param: CreateBucketShareLinkParams,
    ) -> Result<CreateBucketShareLinkResponse, BucketApiError> {
        let req: CreateBucketShareLinkRequest = _param.try_into()?;
        Ok(self
            .client
            .create_bucket_share_link(req)
            .await
            .unwrap()
            .into_inner())
    }

    // async fn create_virtual_filesystem(
    //     bucket_id: uuid::Uuid,
    //     bucket_owner_id: uuid::Uuid,
    //     password: Option<String>,
    // ) -> BucketVirtualFilesystemManager {
    //     let bucket_upload_handle:BucketFileUploadHandlerDyn = BucketFileUploadHandlerImpl;
    //     let client = QueryClient::new(Client { base_url: "".to_string(), options: None });
    //     BucketVirtualFilesystemManager::new(client, bucket_id, bucket_owner_id, password)
    // }
    //
    // async fn create_bucket<FileHandle>(&mut self, params:&CreateBucketParams<FileHandle>) -> Result<(),()> {
    //     let bucket_upload_handle:BucketFileUploadHandlerDyn = BucketFileUploadHandlerImpl::;
    //     crate::controller::bucket::upload(&mut self.client, &params.target_user_id, &params.target_bucket_id, &params.target_directory, &params.source_files, &params.encryption, &params.total_size_in_bytes, &params.hashed_password, bucket_upload_handle).await.unwrap();
    // }
    //
    // async fn upload_files<FileHandle>(&mut self, params:&UploadFilesDto<FileHandle>) {
    //     let bucket_file_handler = BucketFileUploadHandlerImpl::;
    //     upload(&mut self.client, &params.target_user_id, &params.target_bucket_id, &params.target_directory, &params.source_files, params.encryption, params.total_size_in_bytes, params.hashed_password, Arc::new(()));
    // }
    //
    // async fn download_files(&mut self, params:&DownloadFilesDto) {
    //     let download_handler = BucketFileDownloadHandlerDyn::;
    //     download(&mut self.client, &params.user_id, &params.bucket_id, &params.hashed_password, &params.bucket_encryption, &params.keep_file_structure, &download_handler);
    // }
    // async fn delete_files(&self, dto:&DeleteFilesDto) {
    //     delete_file()
    // }
    //
    // async fn delete_bucket(&self, dto:&DeleteBucketDto) {
    //
    //     delete_bucket()
    // }
}
