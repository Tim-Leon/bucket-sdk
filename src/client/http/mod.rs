use std::fmt::Debug;
use std::io::Bytes;
use bucket_common_types::Encoding;
use mime::Mime;
use url::Url;
use crate::token::ApiToken;

#[cfg(target_arch = "wasm32")]
pub mod gloo;
#[cfg(not(target_arch = "wasm32"))]
pub mod native;
pub mod http_request_ext;

pub trait HttpUploadClientExt: Sized{
    type Error: Debug;
    async fn put(&self, url: Url,body: &[u8], api_token: &ApiToken, content_type: Mime, content_encoding: Option<Encoding>) -> Result<(), Self::Error>;
}

/// Trait implemented for an HTTP client
/// to be able to download a file from the URL supplied from the backend
pub trait HttpDownloadClientExt : Sized{
    type Error: Debug;
    async fn get(&self, url: Url, api_token: &ApiToken ,content_encoding: Option<Encoding>) -> Result<Bytes<u8>, Self::Error>;
}

