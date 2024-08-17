use std::fmt::Display;
use crate::api::authentication::JwtToken;
use zeroize::Zeroize;
#[derive(Clone, Debug, PartialEq)]
#[derive(Zeroize)]
pub struct ApiToken(String);

impl TryFrom<&str> for ApiToken {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self {
            0: value.to_string(),
        })
    }
}

impl From<JwtToken> for ApiToken {
    fn from(value: JwtToken) -> Self {
        Self { 0: value }
    }
}

impl Display for ApiToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Continuation token used for request where the client is able to page the data.
pub struct ContinuationToken(String);