use std::sync::Arc;

use aes_gcm::aead::generic_array::typenum;
use aes_gcm::aead::Aead;
use aes_gcm::Nonce;
use ed25519_compact::{Noise, Signature};
use highway::HighwayHash;

use crate::encryption_v1::constants::{
    AES_GCM_NONCE, HIGHWAY_HASH_KEY, SHARE_LINK_SIGNATURE_NOISE,
};
use crate::encryption_v1::encryption::{generate_bucket_encryption_key, ClientSecrets};

pub trait DecryptionModule {
    type Error;
    /// Returns vector of decrypted data
    fn update(&mut self, ciphertext: impl AsRef<[u8]>) -> Result<Vec<u8>, Self::Error>;
    /// Finalize decryption by returning Vector which could contain hash/signature, get creative.
    fn finalize(self) -> Result<(), Self::Error>;
}
#[derive(Clone)]
pub struct ZeroKnowledgeDecryptionModuleV1 {
    secrets: Arc<ClientSecrets>,
    bucket_symmetric_encryption_key: aes_gcm::Aes256Gcm,
    hasher: highway::HighwayHasher,
    nonce: Nonce<typenum::U12>,
    ed25519_noise: Noise,
    signature: Option<Signature>,
}

impl ZeroKnowledgeDecryptionModuleV1 {
    pub fn new(
        secrets: Arc<ClientSecrets>,
        bucket_id: &uuid::Uuid,
        signature: Option<Signature>,
    ) -> Self {
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
            signature,
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
        self.hasher.append(ciphertext.as_ref());
        let plaintext = self
            .bucket_symmetric_encryption_key
            .decrypt(&self.nonce, ciphertext.as_ref())
            .map_err(DecryptionError::FailedToDecryptChunk)?;
        Ok(plaintext)
    }

    fn finalize(self) -> Result<(), Self::Error> {
        let hash_result = self.hasher.finalize256();
        match self.signature {
            None => {}
            Some(signature) => {
                self.secrets
                    .ed25519_keypair
                    .pk
                    .verify(bytemuck::bytes_of(&hash_result), &signature)
                    .map_err(DecryptionError::InvalidSignature)?;
            }
        }
        Ok(())
    }
}
