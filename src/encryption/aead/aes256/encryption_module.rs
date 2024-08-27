use crate::encryption::aead::EncryptionModule;
use crate::encryption::key::derived_key::EncryptionDerivedKey;
use aes_gcm::aead::generic_array::typenum;
use aes_gcm::aead::Aead;
use aes_gcm::Nonce;
use core::slice::SlicePattern;
use generic_array::GenericArray;
use std::io::Write;

pub struct Aes256EncryptionModule<W: Write> {
    buf: Vec<u8>,
    bucket_symmetric_encryption_key: aes_gcm::Aes256Gcm,
    nonce: Nonce<typenum::U12>,
    writer: W,
}

#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("Failed to encrypt chunk: {0}")]
    FailedToEncryptChunk(aes_gcm::Error),
}

impl<R: std::io::Read, W: std::io::Write, N: generic_array::ArrayLength> EncryptionModule<R, W, N>
    for Aes256EncryptionModule<W>
{
    type Error = EncryptionError;

    fn new(
        writer: W,
        secrets: &EncryptionDerivedKey,
        nonce: GenericArray<u8, N>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            buf: Vec::with_capacity(1024),
            bucket_symmetric_encryption_key: secrets.get_aead_encryption_key().unwrap(),
            nonce: Nonce::clone_from_slice(nonce.as_slice()),
            writer,
        })
    }

    fn encrypt_block(&mut self, plaintext: impl AsRef<[u8]>) -> Result<usize, Self::Error> {
        let ciphertext = self
            .bucket_symmetric_encryption_key
            .encrypt(&self.nonce, plaintext.as_ref())
            .map_err(EncryptionError::FailedToEncryptChunk)?;
        let total_block_size = ciphertext.len();
        self.writer.write_all(ciphertext.as_slice()).unwrap();
        Ok(total_block_size)
    }

    fn encrypt_stream(&mut self, mut stream: R) -> Result<usize, EncryptionError> {
        let mut buf = &mut self.buf;
        let mut total_bytes_read = 0;
        loop {
            let bytes_read = stream.read(buf).unwrap();
            total_bytes_read += bytes_read;
            if bytes_read == 0 {
                break;
            }

            let ciphertext = self
                .bucket_symmetric_encryption_key
                .encrypt(&self.nonce, &buf[..bytes_read])
                .unwrap();
            self.writer.write_all(ciphertext.as_slice()).unwrap();
        }

        Ok(total_bytes_read)
    }

    fn finalize(self) {}
}
