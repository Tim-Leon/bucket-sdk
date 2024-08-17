use bucket_common_types::Encoding;
use futures::SinkExt;
use mime::Mime;
use url::Url;
use wasm_bindgen::JsValue;
use crate::client::http::{HttpDownloadClientExt, HttpUploadClientExt};
use crate::client::http::http_request_ext::{HttpRequestContentEncodingHeaderExt, HttpRequestContentTypeHeaderExt};

pub enum HttpError {
    #[error("HttpPutFailed")]
    HttpPutError(#[from] gloo::net::Error),
    #[error("HttpGetFailed")]
    HttpGetError(#[source] gloo::net::Error),
}

pub struct HttpClient {

}

impl HttpUploadClientExt for HttpClient{
    type Error = HttpError;
    async fn put(&self, url: Url,body: &[u8], content_type: Mime, content_encoding: Option<Encoding>) -> Result<(), HttpError> {
        let val = JsValue::from_str(std::str::from_utf8(
            body,
        ).unwrap());
        let resp = gloo::net::http::Request::put(url.as_str()).set_content_type(content_type).set_content_encoding(&content_encoding).body(val).send().await.map_err(|e| Self::Error::HttpPutError(e))?;
        Ok(())
    }
}



impl HttpDownloadClientExt for HttpClient {
    type Error = HttpError;

    async fn get(&self, url: Url, body: String) -> Result<(), Self::Error> {
        let resp = gloo::net::http::Request::get(url.as_str()).send().await.map_err(|e| Self::Error::HttpGetError(e))?;
        Ok(())
    }
}