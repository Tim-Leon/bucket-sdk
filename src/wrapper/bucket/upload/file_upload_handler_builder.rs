use bucket_common_types::{BucketCompression, BucketEncryption, BucketGuid};
use zero_knowledge_encryption::encryption::aead::EncryptionModule;
use crate::compression::CompressorModule;
use crate::wrapper::bucket::upload::FileUploadHandlerBuilder;

pub struct DefaultFileUploadHandlerBuilder {

}

impl FileUploadHandlerBuilder for DefaultFileUploadHandlerBuilder {
    type OutputType = ();

    fn new(target: BucketGuid, bucket_compression: Option<BucketCompression>, bucket_encryption: Option<BucketEncryption>, use_client_compression: bool) -> Self {
        todo!()
    }

    fn set_total_upload_size(&mut self, total_upload_size: u64) {
        todo!()
    }

    fn set_encryption_module<EC: EncryptionModule<R, W, N>>(&mut self, encryption_module: EC) {
        todo!()
    }

    fn set_compression_module<CM: CompressorModule<R, W>>(&mut self, compression_module: CM) {
        todo!()
    }

    fn build(&self) -> Self::OutputType {
        todo!()
    }
}