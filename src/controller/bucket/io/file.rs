use std::{
    io::{Read, Write},
    vec,
};
use mime::Mime;
#[cfg(not(target_family = "wasm"))]
use crate::controller::bucket::io::native_file::VirtualNativeBucketFile;
#[cfg(target_family = "wasm")]
use crate::controller::bucket::io::web_file::{VirtualWebBucketFile, WebBucketFileError, WebFileHandle};

// Information from the API
pub struct VirtualFileDetails {
    pub path: String,
    pub date: Option<time::OffsetDateTime>,
    pub size_in_bytes: u64,
    pub file_format: mime::Mime,
}

/// Traits to collectively implement the read/write to the local filesystem depending on target.
/// While mapping the local file to web file.
/// Current supported targets are native using tokio, or web through WASM.
#[tonic::async_trait(?Send)]
pub trait BucketFileTrait {
    type Error;
    type FileHandle;
    ///// Creates a new local file from the virtual file.
    //fn new_from_virtual(target: VirtualFileDetails) -> Result<Self, Self::Error>;
    ///// Use existing local file and map it to the virtual file.
    //fn from_existing(detail: Arc<VirtualFileDetails>, file_handle: Self::FileHandle) -> Self;
    //fn new(create_file_handle:fn() -> Self::FileHandle) -> Self;

    // DON'T IMPLEMENT THE CREATION OF NEW FILE, ONLY TAKE EXISTING "FILE HANDLE" ONE KEEP IT SIMPLE.
    fn new(filename: &str, mime: &Mime) -> Result<Self, Self::Error> where Self: Sized;
    fn from(file_handle: Self::FileHandle, filename: String) -> Self where Self: Sized;
    fn get_file_handle(&self) -> &Self::FileHandle;
    async fn read_chunk(&self, size: u64, offset: u64) -> Result<Vec<u8>, Self::Error>;
    fn read_stream(&self) -> Result<Box<dyn Read>, Self::Error>;
    fn get_extension(&self) -> Result<String, Self::Error>;
    /// Get the mime-type from the extension.
    fn get_mime_type(&self) -> Result<Mime, Self::Error>;
    /// Uses the first couple of bytes in the file ot determine the mime-type
    async fn infer_mime_type(& self) -> Result<infer::Type, Self::Error>;
    fn write_chunk(&self, chunk: &vec::Vec<u8>, offset: u64) -> Result<(), Self::Error>;
    fn write_stream(&self, stream: &dyn Write) -> Result<(), Self::Error>;
    fn get_size(&self) -> u64;
}

//#[cfg(all(not(feature = "native"), feature = "web"))]
#[cfg(target_family = "wasm")]
pub type BuketFile = VirtualWebBucketFile;
#[cfg(not(target_family = "wasm"))]
pub type BucketFile = VirtualNativeBucketFile;


#[cfg(test)]
mod tests {
    #[test]
    fn test_delete() {}

    #[test]
    fn test_file_creation() {}

    #[test]
    fn test_write() {}

    #[test]
    fn test_read() {}
}