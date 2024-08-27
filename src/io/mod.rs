use std::fmt::Debug;
use std::io::{Read, Write};
use mime::Mime;

pub mod file;
mod native;
mod web;
mod loading;

/// Wrapper for file io. Supports underlying filesystem through std::fs and wasm using gloo.
#[tonic::async_trait(?Send)]
pub trait FileWrapper: Read + Write + Sized {
    type Error: Debug;
    type FileHandle: Read + Write;
    fn create_file(filename: &str, mime: &Mime) -> Result<Self, Self::Error>
    where
        Self: Sized;
    fn from_file_handle(file_handle: Self::FileHandle, filename: String, mime: &Mime) -> Self
    where
        Self: Sized;
    /// Get the extension of file e.x "exe", "rar"...
    fn get_extension(&self) -> Option<String>;
    /// Get the mime-type from the extension.
    fn get_mime_type(&self) -> Result<Mime, Self::Error>;
    /// Uses the first couple of bytes in the file to determine the mime-type
    async fn infer_mime_type(&mut self) -> Result<infer::Type, Self::Error>;
    fn get_size(&self) -> u64;
}

#[cfg(test)]
mod tests {
    use crate::io::native::native_file::NativeFile;

    #[test]
    fn test_delete() {}

    #[test]
    fn test_file_creation() {}

    #[test]
    fn test_write() {
        let file = std::fs::File::open("hello").unwrap();


    }

    #[test]
    fn test_read() {}
}
