use crate::constants::PASSWORD_STRENGTH_SCORE;
use aes_gcm::aead::generic_array::{typenum, GenericArray};
use argon2::password_hash::{Salt, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher};
use base64::engine::Engine;
use sha3::{Digest, Sha3_256};

use super::encryption::MasterKey;

#[derive(Debug, thiserror::Error)]
pub enum PasswordHashErrors {
    //#[error("Email with too long username")]
    //EmailUsernameTooLong(#[from] ),
    #[error("Argon2id hashing error")]
    PasswordHashError(argon2::password_hash::Error),
    #[error("Password Strength error")]
    PasswordStrengthError(#[from] PasswordStrengthError),
    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),
}

/*
* Will be user to create the master key. This key is used to derive encryption keys and signing key for the user.
* It is essential that the password is kept secret or all the encrypted data can be lost.
*/
pub fn argon2id_hash_password<'a, 'b>(
    password: &'a str,
    email: &'a str,
    salt_addon: Salt<'a>, 
) -> Result<PasswordHash<'b>, PasswordHashErrors>
// Specifically the salt must outlive the password hash.
where 'a:'b
{
    // Hash the email with sha512 to get a 64 bytes hash. Which is the max size for argon2id salt. Perfect.
    let mut sha256_hasher = Sha3_256::new(); //Sha3_512::new();
    sha256_hasher.update(salt_addon.as_str().as_bytes()); // This is just additional entropy to make the email more unique. Might be useful?
    sha256_hasher.update(email.as_bytes());
    let email_hash: GenericArray<_, _> = sha256_hasher.finalize();
    let _password_strength = password_strength(email, password, None)?;
    // the max length for salt is 64 bytes so it should work out fine.
    // Hash the password with argon2id and the salt which is sha512(email).
    //let salt = Salt::from_b64(&str::from_utf8(email_hash.as_slice()));

    // Base64 expand size by 1/3
    let encoded = base64::engine::general_purpose::STANDARD_NO_PAD.encode(email_hash.as_slice());
    //let salt = SaltString::from_b64(&encoded).map_err(PasswordHashErrors::PasswordHashError)?; .as_salt()
    let argon2id = Argon2::default();
    let password_hash = argon2id
        .hash_password(password.as_bytes(), salt_addon)
        .map_err(PasswordHashErrors::PasswordHashError)?;

    Ok(password_hash)
}

#[derive(Debug, thiserror::Error)]
pub enum PasswordStrengthError {
    #[error("Password is too weak")]
    TooWeak,
    #[error("Password is too short")]
    TooShort,
    #[error("Password entropy error")]
    EntropyError(#[from] zxcvbn::ZxcvbnError),
    #[error("Password's do not match")]
    NotMatching,
}

// Will check password strength score against constant PASSWORD_STRENGTH_SCORE.
pub fn password_strength(
    email: &str,
    password: &str,
    repeated_password: Option<&str>,
) -> Result<u8, PasswordStrengthError> {
    if password.len() < 8 {
        return Err(PasswordStrengthError::TooShort);
    }
    let entropy = zxcvbn::zxcvbn(password, &[email])?;
    let score = entropy.score();
    if score < PASSWORD_STRENGTH_SCORE {
        return Err(PasswordStrengthError::TooWeak);
    }
    match repeated_password {
        Some(v) => {
            if password != v {
                return Err(PasswordStrengthError::NotMatching);
            }
        }
        None => {}
    }
    Ok(score)
}

pub fn bucket_key_hash_sha256(
    password_hash: &MasterKey,
    bucket_id: &uuid::Uuid,
) -> GenericArray<u8, typenum::U32> {
    let mut hasher = Sha3_256::new(); //Sha3_512::new();
    hasher.update(bucket_id.as_bytes());
    hasher.update(password_hash.0.as_bytes());

    hasher.finalize()
}
