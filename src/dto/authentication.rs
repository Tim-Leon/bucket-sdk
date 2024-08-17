use email_address::EmailAddress;
use crate::captcha::Captcha;

pub struct LoginParams {
    pub email_address: EmailAddress,
    pub password: String,
    pub captcha: Option<Captcha>,
    pub totp_code: Option<String>,
}

pub struct RegistrationParams {
    pub email_address: EmailAddress,
    pub username: String,
    pub password: String,
    pub captcha: Captcha,
}
