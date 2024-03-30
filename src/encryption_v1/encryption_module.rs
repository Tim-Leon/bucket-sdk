use std::sync::Arc;

use aes_gcm::aead::generic_array::typenum;
use aes_gcm::aead::Aead;
use aes_gcm::Nonce;
use highway::HighwayHash;

use crate::encryption_v1::constants::{
    AES_GCM_NONCE, HIGHWAY_HASH_KEY, SHARE_LINK_SIGNATURE_NOISE,
};
use crate::encryption_v1::encryption::{generate_bucket_encryption_key, MasterKey};

use super::hash_based_signature::{Ed25519HighwayHashBasedSignature, HashBasedSignature};

pub trait EncryptionModule: Clone + Sized{
    type Error;
    /// Returns vector of encrypted data
    fn update(&mut self, plaintext: impl AsRef<[u8]>) -> Result<Vec<u8>, Self::Error>;
    /// Finalize encryption by returning Vector which could contain hash/signature, get creative.
    fn finalize(self);
}

#[derive(Clone)]
pub struct ZeroKnowledgeEncryptionModuleV1 {
    secrets: Arc<MasterKey>,
    bucket_symmetric_encryption_key: aes_gcm::Aes256Gcm,
    nonce: Nonce<typenum::U12>,
    hash_based_signature: Option<Ed25519HighwayHashBasedSignature>,
}

#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("Failed to encrypt chunk: {0}")]
    FailedToEncryptChunk(aes_gcm::Error),
}

impl ZeroKnowledgeEncryptionModuleV1 {
    pub fn new(secrets: Arc<MasterKey>, bucket_id: &uuid::Uuid) -> Self {
        Self {
            secrets: secrets.clone(),
            bucket_symmetric_encryption_key: generate_bucket_encryption_key(
                secrets.clone(),
                bucket_id,
            )
            .unwrap(),
            nonce: *Nonce::from_slice(&AES_GCM_NONCE),
            hash_based_signature: None,
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
        Ok(ciphertext)
    }

    fn finalize(self) {
        
    }

}
