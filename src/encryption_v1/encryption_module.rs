use std::sync::Arc;

use aes_gcm::aead::generic_array::typenum;
use aes_gcm::aead::Aead;
use aes_gcm::Nonce;
use ed25519_compact::Noise;
use highway::HighwayHash;

use crate::encryption_v1::constants::{
    AES_GCM_NONCE, HIGHWAY_HASH_KEY, SHARE_LINK_SIGNATURE_NOISE,
};
use crate::encryption_v1::encryption::{generate_bucket_encryption_key, ClientSecrets};

pub trait EncryptionModule: Clone + Sized{
    type Error;
    /// Returns vector of encrypted data
    fn update(&mut self, plaintext: impl AsRef<[u8]>) -> Result<Vec<u8>, Self::Error>;
    /// Finalize encryption by returning Vector which could contain hash/signature, get creative.
    fn finalize(self) -> Result<Vec<u8>, Self::Error>;
}

#[derive(Clone)]
pub struct ZeroKnowledgeEncryptionModuleV1 {
    secrets: Arc<ClientSecrets>,
    bucket_symmetric_encryption_key: aes_gcm::Aes256Gcm,
    hasher: highway::HighwayHasher,
    nonce: Nonce<typenum::U12>,
    ed25519_noise: Noise,
}

#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("Failed to encrypt chunk: {0}")]
    FailedToEncryptChunk(aes_gcm::Error),
}

impl ZeroKnowledgeEncryptionModuleV1 {
    pub fn new(secrets: Arc<ClientSecrets>, bucket_id: &uuid::Uuid) -> Self {
        Self {
            secrets: secrets.clone(),
            bucket_symmetric_encryption_key: generate_bucket_encryption_key(
                secrets.clone(),
                bucket_id,
            )
            .unwrap(),
            hasher: highway::HighwayHasher::new(highway::Key(HIGHWAY_HASH_KEY)),
            nonce: *Nonce::from_slice(&AES_GCM_NONCE), //TODO: NONCE should come from the bucket name?
            ed25519_noise: Noise::from_slice(&SHARE_LINK_SIGNATURE_NOISE).unwrap(),
        }
    }
}

impl EncryptionModule for ZeroKnowledgeEncryptionModuleV1 {
    type Error = EncryptionError;
    fn update(&mut self, plaintext: impl AsRef<[u8]>) -> Result<Vec<u8>, Self::Error> {
        let ciphertext = self
            .bucket_symmetric_encryption_key
            .encrypt(&self.nonce, plaintext.as_ref())
            .map_err(EncryptionError::FailedToEncryptChunk)?;
        self.hasher.append(&ciphertext);
        Ok(ciphertext)
    }

    fn finalize(self) -> Result<Vec<u8>, Self::Error> {
        let hash_result = self.hasher.finalize256();
        let signature = self
            .secrets
            .ed25519_keypair
            .sk
            .sign(bytemuck::bytes_of(&hash_result), Some(self.ed25519_noise));
        Ok(signature.to_vec())
    }
}
