use bucket_common_types::Encoding;
use mime::Mime;
use reqwest::RequestBuilder;
use crate::client::http::http_request_ext::{HttpRequestAuthorizationMetadataExt, HttpRequestContentEncodingHeaderExt, HttpRequestContentTypeHeaderExt, HttpRequestInTransitEncryptionHeaderExt};
use crate::token::ApiToken;

impl HttpRequestContentEncodingHeaderExt for RequestBuilder {
    fn set_content_encoding(self, content_encoding: Option<Encoding>) -> Self {
        match content_encoding {
            None => { self }
            Some(content_encoding) => {
                self.header("content-encoding", content_encoding.to_string())
            }
        }
    }
}

impl HttpRequestAuthorizationMetadataExt for RequestBuilder {
    fn set_authorization_metadata(self, api_token: &ApiToken) -> Self {
        todo!()
    }
}

impl HttpRequestContentTypeHeaderExt for RequestBuilder {
    fn set_content_type(self, content_type: Mime) -> Self {
        todo!()
    }
}

