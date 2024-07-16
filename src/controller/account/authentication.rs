use std::convert::Infallible;
use crate::client::query_client::backend_api::{AccountLoginFinishRequest, AccountLoginStartRequest, CreateAccountFinishRequest, CreateAccountStartRequest};
use crate::client::query_client::QueryClient;
use crate::controller::account::errors::{LoginError, RegisterError};

use crate::{
    constants::PASSWORD_STRENGTH_SCORE,
};
use argon2::Argon2;
use argon2::password_hash::SaltString;

use opaque_ke::{
    rand, ClientLogin, ClientLoginFinishParameters, ClientRegistrationFinishParameters,
    CredentialResponse, RegistrationResponse,
};
use zero_knowledge_encryption::master_key::{MasterKey, MtESignatureKey};

//The ciphersuite trait allows to specify the underlying primitives that will
//be used in the OPAQUE protocol
#[allow(dead_code)]
pub struct DefaultCipherSuite;

//#[cfg(feature = "ristretto255")]
impl opaque_ke::CipherSuite for DefaultCipherSuite {
    type OprfCs = opaque_ke::Ristretto255;
    type KeGroup = opaque_ke::Ristretto255;
    type KeyExchange = opaque_ke::key_exchange::tripledh::TripleDh;
    //type Ksf = argon2::Argon2<'static> for Ksf;
    //type Ksf = opaque_ke::ksf::Identity;
    type Ksf = Argon2<'static>;
}

type JwtToken = String;

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



pub async fn login(
    query_client: &mut QueryClient,
    email: String,
    password: String,
    totp_code: Option<String>,
) -> Result<JwtToken, LoginError> {
    let password_strength_score = password_strength(&email, &password, None)?;
    if password_strength_score < PASSWORD_STRENGTH_SCORE {
        return Err(LoginError::PasswordTooWeak);
    }
    let mut rng = rand::thread_rng();
    let oprf_start = ClientLogin::<DefaultCipherSuite>::start(&mut rng, password.as_bytes())?;

    let start_req = AccountLoginStartRequest {
        email,
        oprf: oprf_start.message.serialize().to_vec(),
    };
    let start_resp = query_client
        .account_login_start(start_req)
        .await
        .unwrap()
        .into_inner();

    let oprf_finish = oprf_start
        .state
        .finish(
            password.as_bytes(),
            CredentialResponse::deserialize(start_resp.oprf.as_slice())?,
            ClientLoginFinishParameters::default(),
        )
        .unwrap();

    let finish_req = AccountLoginFinishRequest {
        oprf: oprf_finish.message.serialize().to_vec(),
        session_id: start_resp.session_id,
        totp_code,
    };

    let finish_resp = query_client
        .account_login_finish(finish_req)
        .await?
        .into_inner();
    Ok(finish_resp.jwt_token as JwtToken)
}

pub async fn register(
    query_client: &mut QueryClient,
    email: &str,
    username: &str,
    password: &str,
    captcha: &str,

) -> Result<JwtToken, RegisterError> {
    if password_strength(&email, &password, None)? < PASSWORD_STRENGTH_SCORE {
        return Err(RegisterError::PasswordTooWeak);
    }
    let mut rng = rand::thread_rng();
    let master_key = MasterKey::generate(&mut rng); // setup(password, email)?;
    let oprf_start =
        opaque_ke::ClientRegistration::<DefaultCipherSuite>::start(&mut rng, password.as_bytes())
            .unwrap();

    let start_req = CreateAccountStartRequest {
        email: email.to_string(),
        oprf: oprf_start.message.serialize().to_vec(),
        captcha: captcha.to_string(),
    };
    let start_resp = query_client
        .create_account_start(start_req)
        .await?
        .into_inner();

    let oprf_finish = oprf_start.state.finish(
        &mut rng,
        password.as_bytes(),
        RegistrationResponse::deserialize(start_resp.oprf.as_slice())?,
        ClientRegistrationFinishParameters::default(),
    )?;

    let signing_key = MtESignatureKey::new(&master_key, SaltString::from_b64()) //create_ed25519_signing_keys(&master_key).unwrap();

    let finish_req = CreateAccountFinishRequest {
        oprf: oprf_finish.message.serialize().to_vec(),
        username: username.to_string(),
        session_id: start_resp.session_id,
        public_signing_key: signing_key.pk.to_vec(),
    };
    let finish_resp = query_client
        .create_account_finish(finish_req)
        .await
        .unwrap()
        .into_inner();
    let jwt_token = finish_resp.jwt_token;
    Ok(jwt_token)
}

// pub fn set_jwt_token_cookie(token: JwtToken) {
//     //https://docs.rs/cookie/0.17.0/cookie/struct.CookieJar.html#method.private
//     let cookie = Cookie::build("jwt_token", token)
//         .path("/")
//         .expires(time::OffsetDateTime::now_utc() + time::Duration::days(1))
//         .http_only(true)
//         .finish();
//     // Generate a secure key.
//     let key = Key::generate();

//     // Add a private (signed + encrypted) cookie.
//     let mut jar = CookieJar::new();
//     jar.private_mut(&key).add(cookie);

//     // The cookie's contents are encrypted.
//     assert_ne!(jar.get("private").unwrap().value(), "text");

//     // They can be decrypted and verified through the child jar.
//     assert_eq!(jar.private(&key).get("private").unwrap().value(), "text");

//     // A tampered with cookie does not validate but still exists.
//     let mut cookie = jar.get("private").unwrap().clone();
//     jar.add(Cookie::new("private", cookie.value().to_string() + "!"));
//     assert!(jar.private(&key).get("private").is_none());
//     assert!(jar.get("private").is_some());

// }

// pub fn get_jwt_token_cookie() -> JwtToken {
//     let mut jar = CookieJar::new();
//     let key = Key::generate();
//     let token = jar.private_mut(&key).get("jwt_token").unwrap().value().to_string();
//     JwtToken::from_str(&token).unwrap()
// }

// // Used for sign out
// pub fn remove_jwt_token_cookie() {
//     let mut jar = CookieJar::new();
//     jar.remove(cookie::Cookie::named("jwt_token"))
// }
