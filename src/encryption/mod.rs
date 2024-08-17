use std::io::{Read, Write};
use bucket_common_types::BucketEncryption;
use generic_array::ArrayLength;
use zero_knowledge_encryption::encryption::aead::EncryptionModule;
use crate::compression::DecompressModule;

mod encryption_chooser_handler;


/// This trait is used
/// to implement behavior related
/// to which encryption/decryption algorithm to use and also all the supported algorithms.
pub trait EncryptionChooserHandler<R: Read, W: Write, N: ArrayLength> {
    fn chose_encryption_handler(&self, bucket_encryption: Option<BucketEncryption>) -> Option<impl EncryptionModule<R, W, N>>;
    fn chose_decryption_handler(&self, bucket_encryption:Option<BucketEncryption>) -> Option<impl DecompressModule<R, W>>;
    fn get_supported_encryption_algorithms(&self) -> &[BucketEncryption];
    fn get_supported_decryption_algorithms(&self) -> &[BucketEncryption];
}