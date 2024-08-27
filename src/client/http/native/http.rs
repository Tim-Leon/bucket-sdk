use crate::client::http::http_request_ext::{HttpRequestAuthorizationMetadataExt, HttpRequestContentEncodingHeaderExt, HttpRequestContentTypeHeaderExt};
use crate::client::http::{HttpDownloadClientExt, HttpUploadClientExt};
use crate::token::ApiToken;
use bucket_common_types::Encoding;
use futures::SinkExt;
use mime::Mime;
use reqwest::{Client, Error};
use std::io::{Bytes, Read};
use url::Url;

pub struct HttpClient {
    client: Client,
}

#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("HttpUploadError")]
    HttpUploadError(#[source] Error),
    #[error("HttpDownloadError")]
    HttpDownloadError(#[source] Error),
}

impl HttpUploadClientExt for  HttpClient{
type Error = HttpError;
    async fn put(&self, url: Url, body: &[u8], api_token: &ApiToken, content_type: Mime, content_encoding: Option<Encoding>) -> Result<(), Self::Error> {
        let resp = self.client.put(url).body(body).set_authorization_metadata(api_token).set_content_type(content_type).set_content_encoding(content_encoding).send().await.map_err(|e| Self::Error::HttpDownloadError(e))?;
        Ok(())
    }
}

impl HttpDownloadClientExt for HttpClient {
    type Error = HttpError;
    async fn get(&self, url: Url, api_token: &ApiToken, content_encoding: Option<Encoding>) -> Result<Bytes<u8>, Self::Error> {
        use HttpRequestContentEncodingHeaderExt;
        let resp = self.client.get(url.as_str()).set_authorization_metadata(api_token).set_content_encoding(content_encoding).send().await.map_err(|e| Self::Error::HttpDownloadError(e))?;
        let binary = resp.bytes().await.unwrap();
        Ok(binary)
    }
}