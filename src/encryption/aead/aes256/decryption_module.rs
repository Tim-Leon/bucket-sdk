use crate::encryption::aead::DecryptionModule;
use core::slice::SlicePattern;
use std::convert::Infallible;
use std::io::{Read, Write};

use crate::encryption::key::derived_key::EncryptionDerivedKey;
use aes_gcm::aead::generic_array::typenum;
use aes_gcm::aead::Aead;
use aes_gcm::Nonce;
use generic_array::{ArrayLength, GenericArray};

#[derive(Clone)]
// TODO: Maybe just use the same struct for both encryption and decryption, there seems to be no need to splitting the two.
pub struct Aes256DecryptionModule<W: Write> {
    bucket_symmetric_encryption_key: aes_gcm::Aes256Gcm,
    nonce: Nonce<typenum::U12>,
    writer: W,
}

impl<W: Write> Aes256DecryptionModule<W> {
    pub fn new(
        writer: W,
        secret: &EncryptionDerivedKey,
        nonce: &Nonce<typenum::U12>,
    ) -> Result<Self, Infallible> {
        Ok(Self {
            bucket_symmetric_encryption_key: secret.get_aead_encryption_key().unwrap(),
            nonce: nonce.clone(),
            writer,
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DecryptionError {
    #[error("Failed to decrypt chunk: {0}")]
    FailedToDecryptChunk(aes_gcm::Error),
    #[error(transparent)]
    InvalidSignature(#[from] ed25519_compact::Error),
}

impl<R: Read, W: Write, N: ArrayLength> DecryptionModule<R, W, N> for Aes256DecryptionModule<W> {
    type Error = DecryptionError;
    fn new(
        writer: W,
        secrets: &EncryptionDerivedKey,
        nonce: GenericArray<u8, N>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            bucket_symmetric_encryption_key: secrets.get_aead_encryption_key().unwrap(),
            nonce: Nonce::clone_from_slice(nonce.as_slice()),
            writer,
        })
    }

    fn decrypt_block(&mut self, ciphertext: impl AsRef<[u8]>) -> Result<usize, Self::Error> {
        let plaintext = self
            .bucket_symmetric_encryption_key
            .decrypt(&self.nonce, ciphertext.as_ref())
            .unwrap();
        let plaintext_size = plaintext.len();
        self.writer.write_all(plaintext.as_slice()).unwrap();
        Ok(plaintext_size)
    }

    fn decrypt_stream(&mut self, cipher_stream: R) -> Result<usize, Self::Error> {
        todo!()
    }

    fn finalize(self) {}

    fn update(&mut self, ciphertext: impl AsRef<[u8]>) -> Result<Vec<u8>, Self::Error> {
        let plaintext = self
            .bucket_symmetric_encryption_key
            .decrypt(&self.nonce, ciphertext.as_ref())
            .map_err(DecryptionError::FailedToDecryptChunk)?;
        Ok(plaintext)
    }
}
