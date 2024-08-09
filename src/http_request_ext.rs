use bucket_common_types::{BucketCompression};
use gloo::net::http::RequestBuilder;
use mime::Mime;
use crate::api::ApiToken;
use crate::compression::CompressorModule;

/// Note this is for HTTP request
pub trait RequestBuilderAuthorizationMetadataExt {
    fn set_authorization_metadata(self, api_token: &ApiToken) -> Self;
}

impl RequestBuilderAuthorizationMetadataExt for RequestBuilder  {
    fn set_authorization_metadata(self, api_token: &ApiToken) -> Self {
        self.header("authorization",format!("Bearer {0}", api_token.to_string()).as_str())
    }
}

pub trait RequestBuilderContentTypeHeaderExt {
    fn set_content_type(self, content_type: Mime) -> Self;
}


impl RequestBuilderContentTypeHeaderExt for RequestBuilder  {
    fn set_content_type(self, content_type: Mime) -> Self {
        self.header("content-type", content_type.to_string().as_str())
    }
}

pub trait RequestBuilderContentEncodingHeaderExt {
    fn set_content_encoding(self,content_encoding: &[bucket_common_types::Encoding]) -> Self;
}

impl RequestBuilderContentEncodingHeaderExt for RequestBuilder {
    fn set_content_encoding(self, content_encoding: &[bucket_common_types::Encoding]) -> Self {
        let encoding_str = content_encoding
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        self.header("content-encoding", encoding_str.as_str() )
    }
}

pub trait RequestBuilderCompressionExt<R: std::io::Read,W: std::io::Write>  {
    fn set_compression(self, compression: &impl CompressorModule<R, W>) -> Self;
}

impl <R: std::io::Read,W: std::io::Write> RequestBuilderCompressionExt<R, W> for RequestBuilder {
    fn set_compression(self, compression: &impl CompressorModule<R, W>) -> Self {
        let supported_compression =
        compression.get_supported_compression();

        let content_encoding = match compression {

        };
        self.set_content_encoding(content_encoding);
    }
}