use core::slice::SlicePattern;
use crate::encryption::key::master_key::MasterKey;
use crate::encryption::key::{SecureGenericArray};
use aes_gcm::KeyInit;
use digest::Digest;
use generic_array::GenericArray;
use secrecy::ExposeSecret;
use sha3::digest;
use sha3::Sha3_256;
use std::convert::Infallible;

/// 256-bit key
pub struct EncryptionDerivedKey {
    secrete: SecureGenericArray<u8, generic_array::typenum::U32>,
}

impl SlicePattern for EncryptionDerivedKey {
    type Item = u8;

    fn as_slice(&self) -> &[Self::Item] {
        self.secrete.0.expose_secret().as_slice()
    }
}

impl EncryptionDerivedKey {
    pub fn new(master_key: &MasterKey, nonce: &[u8]) -> Self {
        let mut hasher = Sha3_256::new(); //Sha3_512::new();
        hasher.update(master_key.as_slice());
        hasher.update(nonce);
        Self {
            secrete: SecureGenericArray {
                0: GenericArray::from_slice(&hasher.finalize()),
            },
        }
    }

    pub fn get_aead_encryption_key(&self) -> Result<aes_gcm::Aes256Gcm, Infallible> {
        let aes_gcm_key =
            aes_gcm::Aes256Gcm::new_from_slice(self.secrete.0.expose_secret().0.as_slice())
                .unwrap(); // Infallible
        Ok(aes_gcm_key)
    }

    pub fn get_chacha20poly1305(&self) -> Result<chacha20poly1305::ChaCha20Poly1305, Infallible> {
        let key = chacha20poly1305::Key::from_slice(self.as_slice());
        let poly_key = chacha20poly1305::ChaCha20Poly1305::new_from_slice(self.as_slice()).unwrap();
        Ok(poly_key)
    }
}

pub struct EncryptionNonce {
    nonce: GenericArray<u8, generic_array::typenum::U12>,
}
