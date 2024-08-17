use bucket_common_types::BucketGuid;
use zero_knowledge_encryption::encryption::aead::DecryptionModule;
use crate::compression::DecompressModule;
use crate::wrapper::bucket::download::FileDownloadHandlerBuilder;

pub struct DefaultFileDownloadHandlerBuilder {

}

impl FileDownloadHandlerBuilder for DefaultFileDownloadHandlerBuilder {
    type OutputType = ();

    fn new(target_bucket: BucketGuid, target_path: String) -> Self {
        todo!()
    }

    fn set_total_download_size(&mut self, total_download_size: u64) {
        todo!()
    }

    fn set_decompressed_module<DC: DecompressModule<R, W>>(&mut self, decompressor: DC) {
        todo!()
    }

    fn set_decrypt_module<DM: DecryptionModule<R, W, N>>(&mut self, decrypt_module: DM) {
        todo!()
    }

    fn build(self) -> Self::OutputType {
        todo!()
    }
}