use aes_gcm::{
    self,
    aead::{generic_array::typenum, Aead},
    KeyInit, Nonce,
};

use std::sync::Arc;
//use sha2::Sha512;
use crate::encryption_v1::constants::V1_ENCRYPTION_PASSWORD_SALT;
use crate::encryption_v1::hash::{
    argon2id_hash_password, bucket_key_hash_sha512, PasswordHashErrors, PasswordStrengthError,
};

use highway::HighwayHash;

use sha3::{digest::InvalidLength, Digest};
use std::str;
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
*/
pub fn setup(password: &str, email: &str) -> Result<ClientSecrets, EncryptionSetupError> {
    let master_key = argon2id_hash_password(password, email, V1_ENCRYPTION_PASSWORD_SALT)?;
    let ed25519_keypair = ed25519_compact::KeyPair::from_slice(&master_key.as_bytes()).unwrap();
    Ok(ClientSecrets {
        master_key,
        ed25519_keypair,
    })
}

pub fn generate_bucket_encryption_key(
    secrets: Arc<ClientSecrets>,
    bucket_id: &uuid::Uuid,
) -> Result<aes_gcm::Aes256Gcm, InvalidLength> {
    let bucket_key = bucket_key_hash_sha512(secrets.master_key.clone(), bucket_id);
    //let aes_gcm_key = aes_gcm::Key::<aes_gcm::Aes256Gcm>::from_slice(bucket_key.as_slice());
    let aes_gcm_key = aes_gcm::Aes256Gcm::new_from_slice(bucket_key.as_slice());
    aes_gcm_key
}

#[derive(zeroize::Zeroize, Clone)]
pub struct ClientSecrets {
    master_key: String,
    pub ed25519_keypair: ed25519_compact::KeyPair,
}

impl ClientSecrets {
    pub fn get_ed25519_public_signing_key(&self) -> ed25519_compact::PublicKey {
        return self.ed25519_keypair.pk;
    }
}

/*
Use the aes_gcm symetric key to encrypt the file content.
Use HighwayHash to hash the encrypted file content.
Use the ed25519 keypair to sign the hash.
*/
pub async fn encrypted_upload_files(
    aes_gcm_symmetric_key: &aes_gcm::Key<aes_gcm::Aes256Gcm>,
    ed25519_signing_key: &ed25519_compact::KeyPair,
    file: gloo::file::File,
    upload_fn: fn(&[u8]),
    upload_finish: fn(&[u8]),
) {
    let key = highway::Key([1, 2, 3, 4]);
    let mut highway_hash = highway::HighwayHasher::new(key);
    let filename = file.name();
    let aes_gcm_cipher = aes_gcm::Aes256Gcm::new_from_slice(&aes_gcm_symmetric_key).unwrap();
    let nonce = aes_gcm::Nonce::from_slice(filename.as_bytes()); //TODO: Fix
                                                                 //let mut file_bytes = file.bytes();
                                                                 //file.read_to_end(&mut file_bytes);
                                                                 //TODO: Chunk it. Read 1MB at a time.
                                                                 //let file_reader = gloo::file::futures::read_as_array_bytes(&file, read_fn);
    let file_size = file.size();
    // Iterate over the file in 1MB chunks,
    // encrypt each chunk while also hashing it.
    // Then upload the encrypted chunk.
    // After the file is uploaded, sign the hash, upload the hash signature.
    // Done!
    for _it in 0..&file_size / 1024 {
        let chunk_data = gloo::file::futures::read_as_bytes(&file).await.unwrap(); //TODO: What if file is too big for memory?
        let ciphertext = aes_gcm_cipher
            .encrypt(&nonce, chunk_data.as_slice())
            .unwrap();
        highway_hash.append(&ciphertext);
        upload_fn(&ciphertext);
    }
    let hash = highway_hash.finalize256();
    let signature = ed25519_signing_key.sk.sign(bytemuck::bytes_of(&hash), None);
    // Upload
    upload_finish(signature.as_slice());
}

pub async fn encrypt_chunk(
    aes_gcm_symmetric_key: &aes_gcm::Key<aes_gcm::Aes256Gcm>,
    nonce: &Nonce<typenum::U12>,
    _ed25519_signing_key: &ed25519_compact::KeyPair,
    highway_hash: &mut highway::HighwayHasher,
    chunk_data: Vec<u8>,
) -> Vec<u8> {
    let aes_gcm_cipher = aes_gcm::Aes256Gcm::new(aes_gcm_symmetric_key);
    let ciphertext = aes_gcm_cipher
        .encrypt(nonce, chunk_data.as_slice())
        .unwrap();
    highway_hash.append(&ciphertext);
    ciphertext
}

// pub async fn encrypt_finalize(
//     ed25519_signing_key: &ed25519_compact::KeyPair,
//     highway_hash: &mut highway::HighwayHasher,
// ) -> Signature {
//     let hash = highway_hash.finalize256();
//     let signature = ed25519_signing_key.sk.sign(&bytemuck::bytes_of(&hash), None);
//     signature
// }

//pub fn generate_rsa_keypair()

#[cfg(test)]
mod tests {

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    #[test]
    fn test_setup() -> Result<(), ()> {
        let _secrets = setup("password", "email");
        //encrypted_upload_files();
        Ok(())
    }
}
