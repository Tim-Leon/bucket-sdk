use std::io::{Read, Write};
use bucket_common_types::BucketEncryption;
use generic_array::ArrayLength;
use crate::compression::DecompressModule;
use crate::encryption::aead::EncryptionModule;

pub mod aead;
pub mod key;
pub mod mte;
pub mod encryption_chooser_handler;


/// This trait is used
/// to implement behavior related
/// to which encryption/decryption algorithm to use and also all the supported algorithms.
pub trait EncryptionChooserHandler<R, W, N>
where R:Read, W:Write, N:ArrayLength {
    type Error;
    fn chose_encryption_handler(&self, bucket_encryption: Option<BucketEncryption>, allow_client_side_encryption: bool) -> Result<Option<impl EncryptionModule<R, W, N>>, Self::Error>;
    fn chose_decryption_handler(&self, bucket_encryption: Option<BucketEncryption>, allow_client_side_decryption: bool) -> Result<Option<impl DecompressModule<R, W>>, Self::Error>;
    fn get_supported_encryption_algorithms(&self) -> &[BucketEncryption];
    fn get_supported_decryption_algorithms(&self) -> &[BucketEncryption];
}