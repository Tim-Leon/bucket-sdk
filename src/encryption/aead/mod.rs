use crate::encryption::key::derived_key::EncryptionDerivedKey;
use generic_array::{ArrayLength, GenericArray};
use std::fmt::Debug;
use std::io::{Read, Write};
pub mod aes256;
pub mod chacha20poly1305;

pub trait EncryptionModule<R, W, N>
where
    Self: Sized,
    R: Read,
    W:Write,
    N:ArrayLength{
    type Error: Debug;
    /// writer is where the encryption output will be written to.
    /// secrets: the secrete key that is being used.
    fn new(
        writer: W,
        secrets: &EncryptionDerivedKey,
        nonce: GenericArray<u8, N>,
    ) -> Result<Self, Self::Error>;

    /// Returns vector of encrypted data
    fn encrypt_block(&mut self, plaintext: impl AsRef<[u8]>) -> Result<usize, Self::Error>;

    fn encrypt_stream(&mut self, stream: R) -> Result<usize, Self::Error>;

    fn finalize(self);
}

pub trait DecryptionModule<R, W, N>:
where
    Self: Sized,
    R:Read,
    W:Write,
    N:ArrayLength
{
    type Error: Debug;
    fn new(
        writer: W,
        secrets: &EncryptionDerivedKey,
        nonce: GenericArray<u8, N>,
    ) -> Result<Self, Self::Error>;
    /// Returns vector of decrypted data
    fn decrypt_block(&mut self, ciphertext: impl AsRef<[u8]>) -> Result<usize, Self::Error>;
    fn decrypt_stream(&mut self, cipher_stream: R) -> Result<usize, Self::Error>;
    fn finalize(self);
    fn update(&mut self, ciphertext: impl AsRef<[u8]>) -> Result<Vec<u8>, Self::Error>;
}
