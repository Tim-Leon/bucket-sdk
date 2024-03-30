use aes_gcm::{
    self,
    aead::{generic_array::typenum, Aead},
    KeyInit, Nonce,
};
use argon2::password_hash::SaltString;

use std::sync::Arc;
//use sha2::Sha512;
use crate::encryption_v1::constants::V1_ENCRYPTION_PASSWORD_SALT;
use crate::encryption_v1::hash::{
    argon2id_hash_password, bucket_key_hash_sha256, PasswordHashErrors, PasswordStrengthError,
};

use highway::HighwayHash;

use sha3::{digest::InvalidLength, Digest, Sha3_256};
use std::str;

use super::constants::V1_X25519_SIGNATURE_HASH_SALT;
// struct DefaultCipherSuite;
// impl CipherSuite for DefaultCipherSuite {
//     type OprfCs = opaque_ke::Ristretto255;
//     type KeGroup = opaque_ke::Ristretto255;
//     type KeyExchange = opaque_ke::key_exchange::tripledh::TripleDh;
//     type Ksf = Argon2<'static>;
// }
// use std::iter::Peekable;

/* ------------INFO-----------
* Version 1.0.0
* The encryption module is responsible for encrypting and decrypting the data.
* The client need to share ED25519 public key with server. This will act as a way for the client to sign data.
* The master key is never shared with the server. And is only used to derive encryption keys and signing keys.
* The derived keys include ED25519, AES-GCM, X25519.
* ED25519 is used to sign data. and public key is shared with the server. Users can request the public ED25519 key of other users from the server. In the future the user will be able to supply this value thyself, for the extra paranoid or security critical systems that don't trust the server, externally supplying this value is the only way to secure against impersonating of website if certificates are stolen, even then no modification will go un-noticed.
* Highwayhash is used to hash the data to create a signature. The signature is then signed with ED25519 private key. The signature is then uploaded to the bucket under filename
'92788755736028022379305121440842586143418638236105851711197675179013382161072356419825278198723874874874859494628870810121940733961303339530173577799516328942537106587822423670792506928743842632331152.signature' (255 character limit) hopefully the user doesn't have a file named the same?.
* That signature is unique and hopefully you don't have a file with the same name.
* AES-GCM is used to encrypt the data. The key is derived from SHA-512 hashed values master key and bucket id. The Hash is to prevent reversing of the key.
* X25519 is used for asymmetric encryption.
* Server X25519-Public-Key is hard coded into the specific encryption version on the client. And used to encrypt link specific settings in SecreteShareLink.
*
* ----------LIBRARIES----------
* Zxcvbn is used to check the password strength. Password requirements: Length >= 8 and score > 3.
* Argon2id is used to hash the password. The password is hashed with the email as salt.
* X25519-dalek asymmetric encryption.
* ED25519-dalek signing/signature verification.
* Sha3-512 is used to hash the master key and bucket id to derive the AES-GCM key.
* AES-GCM is used to encrypt the data. This is a streaming encryption and we should not have to care about padding.
*/

// This key uses

//MasterKey
/*
* Version the client.
* So if a better implementation of the encryption is made, the client can be updated to use the new version while still supporting the old one.
* The hash_password always need to have a salt cause the server and client use OPAQUE-ke to authenticate the user and in this protocol argon2 is also used to hash the password.
*/

// Support both sha2 and sha3 family hash functions, in case of a security issue with one of them.

/*
* Hash the password with the password_hash with bucket_id to create an aes_gcm key which is specific to the bucket.
*/

#[derive(Debug, thiserror::Error)]
pub enum EncryptionSetupError {
    #[error("Encryption setup error")]
    PasswordHashError(#[from] PasswordHashErrors),
    #[error("Encryption setup error")]
    PasswordStrengthError(#[from] PasswordStrengthError),
}

/*
* Entry point for encryption module.
* This function is called after the user has logged in.
* TODO: Fuzz input
* MUST BE DETERMINISTIC
*/
pub fn setup(password: &str, email: &str) -> Result<MasterKey, EncryptionSetupError> {
    let salt = SaltString::from_b64(&V1_ENCRYPTION_PASSWORD_SALT).map_err(PasswordHashErrors::PasswordHashError)?;
    let master_key = argon2id_hash_password(password, email, salt.as_salt())?
        .hash
        .unwrap();
    Ok(MasterKey {
        0: master_key.to_string(),
    })
}



pub fn create_ed25519_signing_keys(master_key: &MasterKey) -> Result<ed25519_compact::KeyPair, ed25519_compact::Error>{
    let mut hasher = Sha3_256::new(); 
    hasher.update(master_key.0.as_bytes());
    hasher.update(V1_X25519_SIGNATURE_HASH_SALT);
    let slice = hasher.finalize();
    let seed = ed25519_compact::Seed::from_slice(&slice).unwrap(); //[0..32]
    let ed25519_key_pair = ed25519_compact::KeyPair::from_seed(seed); 
    Ok(ed25519_key_pair)
}

pub fn generate_bucket_encryption_key(
    master_key: Arc<MasterKey>,
    bucket_id: &uuid::Uuid,
) -> Result<aes_gcm::Aes256Gcm, InvalidLength> {
    let bucket_key = bucket_key_hash_sha256(&master_key, bucket_id);
    //let aes_gcm_key = aes_gcm::Key::<aes_gcm::Aes256Gcm>::from_slice(bucket_key.as_slice());
    let aes_gcm_key = aes_gcm::Aes256Gcm::new_from_slice(bucket_key.as_slice());
    aes_gcm_key
}
#[derive(zeroize::Zeroize)]
pub struct Secrets {
    pub master_key: MasterKey,
    pub signing_key: ed25519_compact::KeyPair,
}

#[derive(zeroize::Zeroize, Clone)]
pub struct MasterKey (pub String);


#[cfg(test)]
mod tests {

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    #[test]
    fn test_setup() -> Result<(), ()> {
        let _secrets = setup("sajMSudosajSADao839d(#", "email");
        //encrypted_upload_files();
        Ok(())
    }
}
