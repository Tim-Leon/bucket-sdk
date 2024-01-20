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

pub trait EncryptionModule {
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
    fn new(secrets: Arc<ClientSecrets>, bucket_id: &uuid::Uuid) -> Self {
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
    fn new(
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

#[cfg(test)]
mod tests {

    use crate::encryption_v1::encryption::setup;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    #[test]
    fn test_encryption_module() -> Result<(), ()> {
        let secrets = Arc::from(setup("jkl!9OSda_jdsAdjoi9839", "email@email.com").unwrap());
        let bucket_id = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let mut enc = ZeroKnowledgeEncryptionModuleV1::new(secrets.clone(), &bucket_id.clone());
        let data = "asjdipasjdaosdjapsdjeaomclcmsiufriunkdvnfdkvdnveruio90903r0jwifojef9qjpquogqhh83uqh8uieaaapöshuidxjnm,zxcbiarv";
        let _enc_data = enc.update(data).unwrap();
        let signature = Signature::from_slice(enc.finalize().unwrap().as_slice()).unwrap();
        let mut _dec = ZeroKnowledgeDecryptionModuleV1::new(secrets, &bucket_id, Some(signature));
        let data2 = _dec.update(_enc_data).unwrap();
        assert_eq!(data, std::str::from_utf8(data2.as_slice()).unwrap());
        Ok(())
    }
}
