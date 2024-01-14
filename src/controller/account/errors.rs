use crate::encryption_v1::encryption::EncryptionSetupError;
use crate::encryption_v1::hash::PasswordStrengthError;
use opaque_ke::errors::ProtocolError;

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("Oprf protocol error")]
    OprfError,
    #[error(transparent)]
    TonicError(#[from] tonic::Status),
    #[error("No token found")]
    NoToken,
    #[error("Password Too Weak")]
    PasswordTooWeak,
    #[error(transparent)]
    PasswordError(#[from] PasswordStrengthError),
}

//https://stackoverflow.com/questions/74973908/how-to-use-thiserror-to-forward-an-error-with-a-generic-type-parameter
impl<T> From<ProtocolError<T>> for LoginError {
    fn from(_err: ProtocolError<T>) -> Self {
        // Get details from the error you want,
        // or even implement for both T variants.
        //Self::Unrar
        Self::OprfError
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RegisterError {
    #[error("Oprf protocol error")]
    OprfError,
    #[error(transparent)]
    TonicError(#[from] tonic::Status),
    #[error("Password too weak")]
    PasswordTooWeak,
    #[error(transparent)]
    PasswordStrengthError(#[from] PasswordStrengthError),
    #[error(transparent)]
    EncryptionSetupError(#[from] EncryptionSetupError),
}

//https://stackoverflow.com/questions/74973908/how-to-use-thiserror-to-forward-an-error-with-a-generic-type-parameter
impl<T> From<ProtocolError<T>> for RegisterError {
    fn from(_err: ProtocolError<T>) -> Self {
        // Get details from the error you want,
        // or even implement for both T variants.
        //Self::Unrar
        Self::OprfError
    }
}
