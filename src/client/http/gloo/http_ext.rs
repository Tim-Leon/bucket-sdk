use gloo::net::http::RequestBuilder;
use mime::Mime;
use crate::client::http::http_request_ext::{HttpRequestAuthorizationMetadataExt, HttpRequestContentEncodingHeaderExt, HttpRequestContentTypeHeaderExt};
use crate::token::ApiToken;

impl HttpRequestAuthorizationMetadataExt for RequestBuilder {
    fn set_authorization_metadata(self, api_token: &ApiToken) -> Self {
        self.header(
            "authorization",
            format!("Bearer {0}", api_token.to_string()).as_str(),
        )
    }
}

impl HttpRequestContentTypeHeaderExt for RequestBuilder {
    fn set_content_type(self, content_type: Mime) -> Self {
        self.header("content-type", content_type.to_string().as_str())
    }
}

impl HttpRequestContentEncodingHeaderExt for RequestBuilder {
    fn set_content_encoding(self, content_encoding: &[bucket_common_types::Encoding]) -> Self {
        let encoding_str = content_encoding
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        self.header("content-encoding", encoding_str.as_str())
    }
}