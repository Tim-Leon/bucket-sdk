use crate::encryption::aead::EncryptionModule;
use crate::encryption::key::derived_key::EncryptionDerivedKey;
use chacha20poly1305::{KeyInit, Nonce};
use core::slice::SlicePattern;
use generic_array::{ArrayLength, GenericArray};
use std::io::{Read, Write};

pub struct Chacha20poly1305EncryptModule<W: Write> {
    buf: Vec<u8>,
    bucket_symmetric_encryption_key: chacha20poly1305::ChaCha20Poly1305,
    nonce: Nonce,
    writer: W,
}

#[derive(thiserror::Error, Debug)]
pub enum Chacha20Poloy1305EncryptionModuleError {}

impl<R: Read, W: Write> EncryptionModule<R, W, generic_array::typenum::N12>
    for Chacha20poly1305EncryptModule<W>
{
    type Error = Chacha20Poloy1305EncryptionModuleError;

    fn new(
        writer: W,
        secrets: &EncryptionDerivedKey,
        nonce: GenericArray<u8, generic_array::typenum::N12>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            buf: Vec::with_capacity(1024),
            bucket_symmetric_encryption_key: secrets.get_chacha20poly1305().unwrap(),
            nonce: Nonce::clone_from_slice(nonce.as_slice()),
            writer,
        })
    }

    fn encrypt_block(&mut self, plaintext: impl AsRef<[u8]>) -> Result<usize, Self::Error> {
        todo!()
    }

    fn encrypt_stream(&mut self, stream: R) -> Result<usize, Self::Error> {
        todo!()
    }

    fn finalize(self) {
        todo!()
    }
}
