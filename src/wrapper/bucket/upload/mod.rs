use std::fmt::Debug;
use async_trait::async_trait;
use bucket_common_types::{BucketCompression, BucketEncryption, BucketGuid};
use generic_array::ArrayLength;
use zero_knowledge_encryption::encryption::aead::EncryptionModule;
use crate::compression::CompressorModule;
use crate::io::file::FileWrapper;

pub mod upload_handler;
pub mod file_upload_handler_builder;

pub trait FileUploadHandlerBuilder {
    type OutputType;
    fn new(target: BucketGuid,
           bucket_compression: Option<BucketCompression>,
           bucket_encryption: Option<BucketEncryption>,
           use_client_compression: bool) -> Self;
    fn set_total_upload_size(&mut self, total_upload_size: u64);
    fn set_encryption_module<EC: EncryptionModule<R, W, N>>(&mut self, encryption_module: EC);
    fn set_compression_module<CM: CompressorModule<R, W>>(&mut self, compression_module: CM);
    fn build(&self) -> Self::OutputType;
}

// A handler is created for each file upload. And will have multiple handlers running in parallel.
#[async_trait(?Send)]
pub trait BucketFileUploadHandler<R: std::io::Read, W: std::io::Write>: Sized {
    // : Send + Sync
    type Error: Debug;

    fn new<N: ArrayLength,BF: FileWrapper, EM: EncryptionModule<R, W, N>, CM: CompressorModule<R, W>>(read_target_file: BF, encryption_module: Option<EM>, compression_module: Option<CM>, bucket_compression: BucketCompression, use_client_compression: bool) -> Result<Self,Self::Error>;
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
    // Called when a chunk is uploaded. returns the chunk to be uploaded. It's up to the implementation to encrypt the chunk and do compression if needed.
    async fn on_upload_chunk(&mut self, chunk_size: u64) -> Result<Vec<u8>, Self::Error>;
    // Called when the last chunk has been uploaded.
    // In this method, the user is still able to upload data, if so it will return a Vec.
    fn on_upload_finish(self) -> Result<(), Self::Error>;
}

