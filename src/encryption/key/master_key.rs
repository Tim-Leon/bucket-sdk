use crate::encryption::key::{SecureGenericArray};
use aes_gcm::aead::rand_core::CryptoRngCore;
use argon2::password_hash::{Salt, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher};
use core::slice::SlicePattern;
use ed25519_compact::KeyPair;
use secrecy::ExposeSecret;
//use hex_literal::hex;
use sha3::{Digest, Sha3_256};
use std::convert::Infallible;
use generic_array::GenericArray;


pub struct MasterKey {
    pub secrete: SecureGenericArray<u8, generic_array::typenum::U32>,
}

impl SlicePattern for MasterKey {
    type Item = u8;

    fn as_slice(&self) -> &[Self::Item] {
        self.secrete.0.expose_secret().as_slice()
    }
}

impl MasterKey {
    pub fn generate<R: CryptoRngCore>(mut csprng: &mut R) -> Self {
        let mut secrete: [u8; 32] = [0; 32];
        csprng.fill_bytes(&mut secrete);
        Self {
            secrete: SecureGenericArray {
                0: GenericArray::from_slice(secrete.as_slice()),
            },
        }
    }

    /// credential_nonce is most likely a username or email.
    pub fn from_plaintext_credentials(
        argon2: &Argon2,
        credential_nonce: &str,
        password: &str,
        salt: SaltString,
    ) -> Result<Self, Infallible> {
        let mut hasher = Sha3_256::new();
        hasher.update(credential_nonce.as_bytes());
        hasher.update(password.as_bytes());
        let mac = hasher.finalize();
        let master_key = argon2
            .hash_password(mac.as_slice(), salt.as_salt())
            .unwrap();
        Ok(MasterKey {
            secrete: SecureGenericArray::from(*GenericArray::from_slice(
                master_key.hash.unwrap().as_bytes(),
            )),
        })
    }

    pub fn from_phc_string(phc: &PasswordHash) -> Self {
        let output = phc.hash.unwrap();
        Self {
            secrete: SecureGenericArray::from(GenericArray::from_slice(output.as_bytes())),
        }
    }

    pub fn from_slice(slice: &[u8]) -> Self {
        Self {
            secrete: SecureGenericArray {
                0: GenericArray::from_slice(slice),
            },
        }
    }
}

pub struct MtESignatureKey {
    pub ed25519_key_pair: KeyPair,
}
impl MtESignatureKey {
    pub fn new(master_key: &MasterKey, salt: Salt) -> Result<Self, Infallible> {
        let mut hasher = Sha3_256::new();
        hasher.update(master_key.as_slice());
        hasher.update(salt.as_str().as_bytes());
        let slice = hasher.finalize();
        let seed = ed25519_compact::Seed::from_slice(&slice).unwrap(); //[0..32]
        let ed25519_key_pair = ed25519_compact::KeyPair::from_seed(seed);
        Ok(Self { ed25519_key_pair })
    }
}
