use std::io::Cursor;
use async_trait::async_trait;
use bucket_common_types::{BucketCompression, BucketEncryption};
use prost::bytes::Bytes;
use zero_knowledge_encryption::encryption::aead::encryption_module::{EncryptionError, ZeroKnowledgeEncryptionModuleV1};
use zero_knowledge_encryption::encryption::aead::EncryptionModule;
use crate::compression::{CompressorModule, DecompressModule};
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

#[derive(Clone)]
pub struct BucketFileReader<R, W> {
    pub read_target_file: BucketFile,
    pub encryption_module: Option<ZeroKnowledgeEncryptionModuleV1>,
    pub offset: u64,
    pub compressor: dyn CompressorModule<R, W, Error=()>,
    pub bucket_compression: BucketCompression,
    pub use_client_compression : bool,
}

// A handler is created for each file upload. And will have multiple handlers running in parallel.
#[async_trait(?Send)]
pub trait BucketFileUploadHandler<R: std::io::Read,W: std::io::Write> : CompressorModule<R, W> + Clone {
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
        bucket_compression: Option<BucketCompression>,
        use_client_side_compression: bool,
        upload_size_in_bytes: u64,
    ) -> Result<u64, Self::Error>;
    // Called when a chunk is uploaded. returns the chunk to be uploaded. It's up to the implementation to encrypt the chunk if the bucket is encrypted.
    async fn on_upload_chunk(&mut self, chunk_size: u64) -> Result<Vec<u8>, Self::Error>;
    // Called when the last chunk has been uploaded. In this method the user is still able to upload data, if so it will return a Vec.
    fn on_upload_finish(self) -> Result<(), Self::Error>;
}

pub enum CompressionModuleError {

}

impl<R: std::io::Read, W: std::io::Write> CompressorModule<R, W> for BucketFileReader {
    type Error = CompressionModuleError;

    fn new(writer: W,bucket_compression: BucketCompression, use_client_side_compression: bool) -> Self {
        Self{
            read_target_file: (),
            encryption_module: None,
            offset: 0,
            bucket_compression,
            use_client_compression: false,
        }
    }

    fn compress_chunk(&self, bytes: &Bytes) -> Result<Vec<u8>, Self::Error> {
        todo!()
    }

    fn compress_stream(&self, reader: R) -> Result<(), Self::Error> {

    }

    fn get_supported_compression() -> &'static [BucketCompression] {
        todo!()
    }
}

#[async_trait(?Send)]
impl<R: std::io::Read, W: std::io::Write> BucketFileUploadHandler<R, W> for BucketFileReader<R, W> {
    //BucketDownloadHandlerFile
    type Error = BucketDownloadHandlerFileErrors;
    fn on_upload_start(
        &self,
        target_bucket_id: uuid::Uuid,
        target_user_id: uuid::Uuid,
        to_directory: String,
        to_filename: String,
        encryption: Option<BucketEncryption>,
        bucket_compression: Option<BucketCompression>,
        use_client_side_compression: bool,
        upload_size_in_bytes: u64,
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
