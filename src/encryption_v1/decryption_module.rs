use std::sync::Arc;

use aes_gcm::aead::generic_array::typenum;
use aes_gcm::aead::Aead;
use aes_gcm::Nonce;
use ed25519_compact::{Noise, Signature};
use highway::HighwayHash;

use crate::encryption_v1::constants::{
    AES_GCM_NONCE, HIGHWAY_HASH_KEY, SHARE_LINK_SIGNATURE_NOISE,
};
use crate::encryption_v1::encryption::{generate_bucket_encryption_key, MasterKey};

use super::hash_based_signature::Ed25519HighwayHashBasedSignature;

pub trait DecryptionModule {
    type Error;
    /// Returns vector of decrypted data
    fn update(&mut self, ciphertext: impl AsRef<[u8]>) -> Result<Vec<u8>, Self::Error>;
    fn finalize(self);
}
#[derive(Clone)]
pub struct ZeroKnowledgeDecryptionModuleV1 {
    secrets: Arc<MasterKey>,
    bucket_symmetric_encryption_key: aes_gcm::Aes256Gcm,
    nonce: Nonce<typenum::U12>,
    hash_based_signature: Option<Ed25519HighwayHashBasedSignature>,
}

impl ZeroKnowledgeDecryptionModuleV1 {
    pub fn new(
        secrets: Arc<MasterKey>,
        bucket_id: &uuid::Uuid,
    ) -> Self {
        Self {
            secrets: secrets.clone(),
            bucket_symmetric_encryption_key: generate_bucket_encryption_key(
                secrets.clone(),
                bucket_id,
            )
            .unwrap(),
            nonce: *Nonce::from_slice(&AES_GCM_NONCE), //TODO: NONCE should come from the bucket name?
            hash_based_signature: None
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DecryptionError {
    #[error("Failed to decrypt chunk: {0}")]
    FailedToDecryptChunk(aes_gcm::Error),
    #[error(transparent)]
    InvalidSignature(#[from] ed25519_compact::Error),
}

impl DecryptionModule for ZeroKnowledgeDecryptionModuleV1 {
    type Error = DecryptionError;
    fn update(&mut self, ciphertext: impl AsRef<[u8]>) -> Result<Vec<u8>, Self::Error> {
        let plaintext = self
            .bucket_symmetric_encryption_key
            .decrypt(&self.nonce, ciphertext.as_ref())
            .map_err(DecryptionError::FailedToDecryptChunk)?;
        Ok(plaintext)
    }
    
    fn finalize(self) {
        
    }

}
