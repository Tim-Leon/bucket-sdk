#[cfg(not(target_family = "wasm"))]
use crate::io::native::native_file::NativeFile;
#[cfg(target_family = "wasm")]
use crate::io::web::web_file::{
    VirtualWebBucketFile, WebBucketFileError, WebFileHandle,
};
use mime::Mime;
use std::{
    io::{Read, Write},
    vec,
};
use std::fmt::Debug;
use time::OffsetDateTime;

// Information from the API
pub struct VirtualFileDetails {
    pub path: String,
    pub date: Option<time::OffsetDateTime>,
    pub size_in_bytes: u64,
    //pub file_format: mime::Mime,
}

pub struct VirtualBucketFileMetadata {
    pub path: String,
    pub date: Option<OffsetDateTime>,
    pub size_in_bytes: u64,
}



