use bucket_common_types::BucketEncryption;
use zero_knowledge_encryption::encryption::aead::EncryptionModule;
use crate::compression::DecompressModule;
use crate::encryption::EncryptionChooserHandler;

pub struct DefaultEncryptionChooserHandler {

}

impl EncryptionChooserHandler<R, W, N> for DefaultEncryptionChooserHandler {
    fn chose_encryption_handler(&self, bucket_encryption: Option<BucketEncryption>) -> Option<impl EncryptionModule<R, W, N>> {
        todo!()
    }

    fn chose_decryption_handler(&self, bucket_encryption: Option<BucketEncryption>) -> Option<impl DecompressModule<R, W>> {
        todo!()
    }

    fn get_supported_encryption_algorithms(&self) {
        todo!()
    }

    fn get_supported_decryption_algorithms(&self) {
        todo!()
    }
}