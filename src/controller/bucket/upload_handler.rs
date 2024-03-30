use crate::encryption_v1::encryption_module::{EncryptionError, EncryptionModule, ZeroKnowledgeEncryptionModuleV1};
use async_trait::async_trait;
use bucket_common_types::BucketEncryption;

use super::io::file::{BucketFile, BucketFileTrait};

#[derive(Debug, thiserror::Error)]
pub enum BucketDownloadHandlerFileErrors {
    #[error("Encryption module not initialized when bucket is encrypted.")]
    EncryptionModuleNotInitialized,
    #[error(transparent)]
    FileReaderError(#[from] gloo::file::FileReadError),
    #[error(transparent)]
    EncryptionError(#[from] EncryptionError),
}

pub struct BucketFileReader {
    pub read_target_file: BucketFile,
    pub encryption_module: Option<ZeroKnowledgeEncryptionModuleV1>,
    pub offset: u64,
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
    fn on_upload_finish(self) -> Result<(), Self::Error>;
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
        match bucket_encryption {
            Some(_bucket_encryption) => match &self.encryption_module {
                Some(_encryption_module) => {
                }
                None => {
                    return Err(BucketDownloadHandlerFileErrors::EncryptionModuleNotInitialized);
                }
            },
            None => {}
        };

        let size = self.read_target_file.get_size();
        Ok(size)
    }

    async fn on_upload_chunk(&mut self, chunk_size: u64) -> Result<Vec<u8>, Self::Error> {
        let bytes = self
            .read_target_file
            .read_chunk(chunk_size, self.offset)
            .await.unwrap();

        match &mut self.encryption_module {
            Some(x) => {
                let encrypted_bytes = x.update(bytes)?;
                Ok(encrypted_bytes)
            }
            None => Ok(bytes),
        }
    }

    fn on_upload_finish(self) -> Result<(), Self::Error> {
        let signed_hash = match self.encryption_module {
            Some(x) => Some(x.finalize()),
            None => return Err(Self::Error::EncryptionModuleNotInitialized),
        };
        Ok(())
    }
}
