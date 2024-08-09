use std::str::FromStr;
use gloo::net::http::RequestBuilder;
use mime::Mime;
use tonic::metadata::{Ascii, MetadataValue};
use tonic::Request;
use crate::api::ApiToken;

pub trait RequestAuthorizationMetadataExt<T> {

    fn set_authorization_metadata(&mut self, api_token: &ApiToken) ;
}

impl<T> RequestAuthorizationMetadataExt<T> for Request<T>
{
    fn set_authorization_metadata(&mut self, api_token: &ApiToken)  {
        let meta = self.metadata_mut();
        let mut token: String = "Bearer ".to_string();
        token.push_str(api_token.to_string().as_str());
        let meta_data = MetadataValue::<Ascii>::from_str(token.as_str()).unwrap();
        meta.append("authorization", meta_data);
    }
}


pub trait ContentTypeRequestExt<T> {
    fn set_content_type(&mut self, mime: &Mime);
}

impl <T> ContentTypeRequestExt<T> for Request<T> {
    fn set_content_type(&mut self, mime: &Mime) {
        let meta = self.metadata_mut();
        let meta_data = MetadataValue::<Ascii>::from_str(mime.to_string().as_str()).unwrap();
        meta.append("content-type", meta_data);
    }
}

