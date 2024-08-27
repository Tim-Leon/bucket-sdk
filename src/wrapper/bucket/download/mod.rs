use std::io::{Read, Write};
use async_trait::async_trait;
use brotli::Decompressor;
use bucket_common_types::{BucketCompression, BucketEncryption, BucketGuid};
use generic_array::ArrayLength;
use url::Url;
use zero_knowledge_encryption::encryption::aead::{DecryptionModule, EncryptionModule};
use crate::client::http::HttpDownloadClientExt;
use crate::compression::{CompressionChooserHandling, CompressorModule, DecompressModule};
use crate::encryption::EncryptionChooserHandler;
use crate::token::ApiToken;

pub mod download_handler;
pub mod file_download_handler_builder;
// A handler is create for each file download.

pub trait FileDownloadHandlerBuilder<R,W,N,HTTP,CCH,ECH>
where 
R: Read,
W:Write,
N:ArrayLength,
HTTP: HttpDownloadClientExt,
CCH: CompressionChooserHandling<R, W>,
ECH: EncryptionChooserHandler<R, W, N>{
    type OutputType;
    fn new(target_bucket: &BucketGuid,
           api_token: &ApiToken,
           target_path: &String,
           upload_url: &Url,
           client: &HTTP) -> Self;
    fn set_total_download_size(&mut self, total_download_size: u64);
    fn set_keep_structure(&mut self, keep_file_structure: bool);
    fn set_decompressed_module<DC>(&mut self, decompressor: DC) where DC: DecompressModule<R, W>;
    fn set_decrypt_module<DM>(&mut self, decrypt_module: DM) where DM: DecryptionModule<R, W, N>;
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


