use crate::controller::bucket::errors::BucketDownloadHandlerFileErrors;
use crate::encryption_v1::module::{EncryptionModule, ZeroKnowledgeEncryptionModuleV1};
use async_trait::async_trait;
use bucket_common_types::BucketEncryption;
use gloo::file::futures::read_as_bytes;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct BucketFileReader {
    pub read_target_file: gloo::file::File, //Arc<Mutex<_>
    pub encryption_module: Option<ZeroKnowledgeEncryptionModuleV1>,
    pub offset: u64,
}

pub struct SyncBucketDownloadHandlerFile {
    inner: Arc<Mutex<BucketFileReader>>,
}
// A handler is created for each file upload. And will have multiple handlers running in parallel.
#[async_trait(?Send)]
pub trait BucketFileUploadHandler {
    // : Send + Sync
    type Error;
    // Called when the upload starts.
    fn on_upload_start(
        &self,
        target_bucket_id: uuid::Uuid,
        target_user_id: uuid::Uuid,
        to_directory: String,
        to_filename: String,
        encryption: Option<BucketEncryption>,
        upload_size_in_bytes: u64,
    ) -> Result<u64, Self::Error>;
    // Called when a chunk is uploaded. returns the chunk to be uploaded. It's up to the implementation to encrypt the chunk if the bucket is encrypted.
    async fn on_upload_chunk(&mut self, chunk_size: u64) -> Result<Vec<u8>, Self::Error>;
    // Called when the last chunk has been uploaded. In this method the user is still able to upload data, if so it will return a Vec.
    fn on_upload_finish(self) -> Result<Option<Vec<u8>>, Self::Error>;
}

#[async_trait(?Send)]
impl BucketFileUploadHandler for BucketFileReader {
    //BucketDownloadHandlerFile
    type Error = BucketDownloadHandlerFileErrors;
    fn on_upload_start(
        &self,
        _target_bucket_id: uuid::Uuid,
        _target_user_id: uuid::Uuid,
        _to_directory: String,
        _to_filename: String,
        bucket_encryption: Option<BucketEncryption>,
        _upload_size_in_bytes: u64,
    ) -> Result<u64, Self::Error> {
        wasm_bindgen_futures::spawn_local(async move {});
        match bucket_encryption {
            Some(_bucket_encryption) => match &self.encryption_module {
                Some(_encryption_module) => {}
                None => {
                    return Err(BucketDownloadHandlerFileErrors::EncryptionModuleNotInitialized);
                }
            },
            None => {}
        };

        let size = self.read_target_file.size();
        Ok(size)
    }

    async fn on_upload_chunk(&mut self, chunk_size: u64) -> Result<Vec<u8>, Self::Error> {
        let bytes = read_as_bytes(&self.read_target_file.slice(self.offset, chunk_size)).await?;
        // let reader = read_as_array_buffer(blob, |buffer|(&self){
        //     self.on_donwload_chunk();
        //     let bytes = js_sys::Uint8Array::new(&buffer).to_vec();
        //     let r_bytes = byte;
        // });

        match &mut self.encryption_module {
            Some(x) => {
                let decrypted_bytes = x.update(bytes)?;
                Ok(decrypted_bytes)
            }
            None => Ok(bytes),
        }
    }

    fn on_upload_finish(self) -> Result<Option<Vec<u8>>, Self::Error> {
        let signed_hash: Option<Vec<u8>> = match self.encryption_module {
            Some(x) => Some(x.finalize()?),
            None => return Err(Self::Error::EncryptionModuleNotInitialized),
        };
        Ok(signed_hash)
    }
}
