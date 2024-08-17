use mime::Mime;
use crate::token::ApiToken;

/// Note this is for HTTP request
pub trait HttpRequestAuthorizationMetadataExt {
    fn set_authorization_metadata(self, api_token: &ApiToken) -> Self;
}



pub trait HttpRequestContentTypeHeaderExt {
    fn set_content_type(self, content_type: Mime) -> Self;
}


pub trait HttpRequestContentEncodingHeaderExt {
    fn set_content_encoding(self, content_encoding: &[bucket_common_types::Encoding]) -> Self;
}



// pub trait RequestBuilderCompressionExt<R: std::io::Read,W: std::io::Write>  {
//     fn set_compression(self, compression: &impl CompressorModule<R, W>) -> Self;
// }
//
// impl <R: std::io::Read,W: std::io::Write> RequestBuilderCompressionExt<R, W> for RequestBuilder
// where
//  {
//     fn set_compression(self, compression: &impl CompressionChooserHandling<R, W>) -> Self {
//         self.set_content_encoding(content_encoding);
//     }
// }

