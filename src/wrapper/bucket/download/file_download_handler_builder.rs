use std::io::{Read, Write};
use bucket_common_types::BucketGuid;
use generic_array::ArrayLength;
use url::Url;
use zero_knowledge_encryption::encryption::aead::DecryptionModule;
use crate::client::http::HttpUploadClientExt;
use crate::compression::DecompressModule;
use crate::wrapper::bucket::download::{FileDownloadHandler, FileDownloadHandlerBuilder};

pub struct DefaultFileDownloadHandlerBuilder<HTTP: HttpUploadClientExt, R: Read, W: Write, N: ArrayLength, DECOMPRESS: DecompressModule<R, W>, DECRYPT: DecryptionModule<R, W, N>> {
    pub target_bucket: BucketGuid,
    pub target_path: String,
    pub upload_url: Url,
    pub client: HTTP,
    pub download_size: Option<u64>,
    pub decompression_module: Option<DECOMPRESS>,
    pub decryption_module: Option<DECRYPT>,
    pub keep_file_structure: bool,
    
}

impl <HTTP: HttpUploadClientExt, CCH, ECH, N, W, R, DECOMPRESS: DecompressModule<R, W>, DECRYPT: DecryptionModule<R, W, N>> FileDownloadHandlerBuilder<R,W,N,HTTP, CCH, ECH> for DefaultFileDownloadHandlerBuilder<HTTP, R, W, N, DECOMPRESS, DECRYPT> {
    type OutputType = FileDownloadHandler<R, W, Error=()>;

    fn new(target_bucket: &BucketGuid, target_path: &String, upload_url: &Url, client: &HTTP) -> Self {
        Self {
            target_bucket: target_bucket,
            target_path: target_path,
            upload_url: upload_url,
            client: client,
            download_size: None,
            decompression_module: None,
            decryption_module: None,
            keep_file_structure: true,
        }
    }


    fn set_total_download_size(&mut self, total_download_size: u64) {
        self.download_size = Some(total_download_size);
    }

    fn set_keep_structure(&mut self, keep_file_structure: bool) {
        todo!()
    }

    fn set_decompressed_module<DC: DecompressModule<R, W>>(&mut self, decompressor: DC) {
        self.decompression_module = Some(decompressor);
    }

    fn set_decrypt_module<DM: DecryptionModule<R, W, N>>(&mut self, decrypt_module: DM) {
        self.decryption_module = Some(decrypt_module);
    }

    fn build(self) -> Self::OutputType {
        FileDownloadHandler {

        }
    }
}