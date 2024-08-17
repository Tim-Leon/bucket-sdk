use async_trait::async_trait;
use brotli::Decompressor;
use bucket_common_types::{BucketCompression, BucketEncryption, BucketGuid};
use url::Url;
use zero_knowledge_encryption::encryption::aead::{DecryptionModule, EncryptionModule};
use crate::client::http::HttpDownloadClientExt;
use crate::compression::{CompressorModule, DecompressModule};

pub mod download_handler;
pub mod file_download_handler_builder;
// A handler is create for each file download.

pub trait FileDownloadHandlerBuilder<HTTP: HttpDownloadClientExt> {
    type OutputType;

    fn new(target_bucket: &BucketGuid, target_path: &String, upload_url: &Url, client: &HTTP) -> Self;
    fn set_total_download_size(&mut self, total_download_size: u64);
    fn set_decompressed_module<DC: DecompressModule<R, W>>(&mut self, decompressor: DC);
    fn set_decrypt_module<DM: DecryptionModule<R, W, N>>(&mut self, decrypt_module: DM);
    fn build(self) -> Self::OutputType;
}

#[derive(Debug, thiserror::Error)]
pub enum BucketUploadHandlerErrors {}

#[async_trait(? Send)]
pub trait FileDownloadHandler<R, W> {
    type Error: std::error::Error + Send + Sync + 'static;
    // Called when a chunk is downloaded. It's up to the user to decrypt the chunk if the bucket is encrypted, or to save the chunk to a file.
    async fn on_download_chunk(&mut self, chunk: &mut [u8]) -> Result<(), Self::Error>;
    // Called when the last chunk has been downloaded.
    fn on_download_finish(self) -> Result<(), Self::Error>;
}


