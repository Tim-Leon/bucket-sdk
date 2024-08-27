use crate::encryption::aead::DecryptionModule;
use crate::encryption::key::derived_key::EncryptionDerivedKey;
use aes_gcm::aead::generic_array::typenum;
use aes_gcm::Nonce;
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{ChaCha20Poly1305, KeyInit};
use generic_array::{ArrayLength, GenericArray};
use std::io::{Read, Write};

pub struct Chacha20poly1305DecryptModule<W: Write> {
    bucket_symmetric_encryption_key: chacha20poly1305::ChaCha20Poly1305,
    nonce: Nonce<typenum::U12>,
    writer: W,
}

#[derive(Debug, thiserror::Error)]
pub enum Chacha20poly1305DecryptionModuleError {}

impl<R: Read, W: Write, N: ArrayLength> DecryptionModule<R, W, N>
    for Chacha20poly1305DecryptModule<W>
{
    type Error = Chacha20poly1305DecryptionModuleError;

    fn new(
        writer: W,
        secrets: &EncryptionDerivedKey,
        nonce: GenericArray<u8, N>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            bucket_symmetric_encryption_key: secrets.get_chacha20poly1305().unwrap(),
            nonce: Nonce::from(nonce),
            writer,
        })
    }

    fn decrypt_block(&mut self, ciphertext: impl AsRef<[u8]>) -> Result<usize, Self::Error> {
        let a = self
            .bucket_symmetric_encryption_key
            .decrypt(&self.nonce, &ciphertext)
            .unwrap();
        let size = a.len();
        self.writer.write_all(&a.as_slice()).unwrap();
        Ok(size)
    }

    fn decrypt_stream(&mut self, cipher_stream: R) -> Result<usize, Self::Error> {
        todo!()
    }

    fn finalize(self) {
        todo!()
    }

    fn update(&mut self, ciphertext: impl AsRef<[u8]>) -> Result<Vec<u8>, Self::Error> {
        todo!()
    }
}
