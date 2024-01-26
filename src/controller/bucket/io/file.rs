use byte_unit::TryFromIntError;
use infer::Type;
use mime::Mime;


use std::io::{BufRead, Read, Write};
use std::str::FromStr;
use std::sync::Arc;
use std::vec;
use web_sys::ReadableStream;

// Information from the API
pub struct VirtualFileDetails {
    pub path: String,
    pub date: Option<time::OffsetDateTime>,
    pub size_in_bytes: u64,
}

trait VirtualFileWriteTrait {}

trait VirtualFileReadTrait {}

trait VirtualFileDetailTrait {}

pub trait BucketFileTrait {
    type Error;
    type FileHandle;
    fn new(detail: Arc<VirtualFileDetails>, file_handle: Option<Self::FileHandle>) -> Self;
    fn is_readable(&self) -> bool;
    fn get_file_handle(&self) -> &Option<Self::FileHandle>;
    fn read_chunk(&self, size: u64, offset: u64) -> Result<Vec<u8>, Self::Error>;
    fn read_stream(&self) -> Result<Box<dyn Read>, Self::Error>;
    fn get_extension(&self) -> Result<String, Self::Error>;
    /// Get the mime-type from the extension.
    fn get_mime_type(&self) -> Result<Mime, Self::Error>;
    /// Uses the first couple of bytes in the file ot determine the mime-type
    fn infer_mime_type(&self) -> Result<infer::Type, Self::Error>;
    fn write_chunk(&self, chunk: vec::Vec::<u8>, offset: u64) -> Result<(), Self::Error>;
    fn write_stream(&self, stream: &dyn Write) -> Result<(),Self::Error>;
    fn get_size(&self) -> Option<u64>;
}
#[derive(thiserror::Error, Debug)]
pub enum WebBucketFileError {
    #[error("No file handle")]
    NoFileHandle,
    #[error("Unknown file type")]
    UnknownFileType,
    #[error("Empty")]
    Empty,
    #[error("No extension")]
    NoExtension,
    #[error(transparent)]
    TryFromIntError(#[from] TryFromIntError),
}
pub struct VirtualWebBucketFile {
    pub file_handle: Option<web_sys::HtmlInputElement>,
    pub virtual_file_details: Arc<VirtualFileDetails>,
}

impl BucketFileTrait for VirtualWebBucketFile {
    type Error = WebBucketFileError;
    type FileHandle = web_sys::HtmlInputElement;

    fn new(detail: Arc<VirtualFileDetails>, file_handle: Option<Self::FileHandle>) -> Self {
        Self {
            file_handle,
            virtual_file_details: detail,
        }
    }
    // Remember read is for uploading and write is for downloading. Kinda reversed if you think about it.
    fn is_readable(&self) -> bool {
        self.file_handle.is_some()
    }
    fn get_file_handle(&self) -> &Option<Self::FileHandle> {
        &self.file_handle
    }

    fn read_chunk(&self, size: u64, offset: u64) -> Result<Vec<u8>, Self::Error> {
        match &self.file_handle {
            Some(x) => {
                let file = x.files().unwrap();
                let _rs = file.get(0).unwrap().stream();
                let start = size - offset;
                let str = file
                    .get(offset.try_into()?)
                    .unwrap()
                    .slice_with_i32(i32::try_from(start).unwrap())
                    .unwrap()
                    .array_buffer()
                    .as_string();
                match str {
                    None => Err(WebBucketFileError::Empty),
                    Some(str) => Ok(str.into_bytes()),
                }
            }
            None => {
                // Can not read from file that does not have a corresponding handle attached.
                Err(WebBucketFileError::NoFileHandle)
            }
        }
    }

    fn read_stream(&self) -> Result<Box<dyn Read>, Self::Error> {
        todo!();
        //match &self.file_handle {
        //    Some(x) => {
        //        let file = x.files().unwrap();
        //        let rs = file.get(0).unwrap().stream();
        //        Ok(rs)
        //    }
        //    None => {
        //        // Can not read from file that does not have a corresponding handle attached.
        //        Err(WebBucketFileError::NoFileHandle)
        //    }
        //}
    }

    fn get_extension(&self) -> Result<String, Self::Error> {
        let extension = self
            .virtual_file_details
            .path
            .rsplit_once('.')
            .ok_or(WebBucketFileError::NoExtension)?;
        let (extension, _) = extension; // Unwrap the result
        Ok(extension.to_string())
    }

    // Checks file extension to get mime type.
    fn get_mime_type(&self) -> Result<Mime, Self::Error> {
        let extension = self.get_extension().unwrap();
        let mime = mime::Mime::from_str(extension.as_str()).unwrap();
        Ok(mime)
    }
    //Checks the first couple of bytes of the file to get mime type.
    fn infer_mime_type(&self) -> Result<infer::Type, Self::Error> {
        match &self.file_handle {
            None => Err(WebBucketFileError::NoFileHandle),
            Some(_handle) => {
                let buf = self.read_chunk(16, 0).unwrap();
                let kind = infer::get(&buf);
                match kind {
                    None => Err(WebBucketFileError::UnknownFileType),
                    Some(kind) => Ok(kind),
                }
            }
        }
    }

    fn write_chunk(&self, chunk: vec::Vec::<u8>, offset: u64) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_stream(&self, stream: &dyn Write) -> Result<(),Self::Error> {
        todo!()
    }

    fn get_size(&self) -> Option<u64> {
        todo!()
    }



}

// Virtual files can either be in the cloud, or on the device. If they are already on the device the NativeBucketFile will be used.
// pub enum VirtualFileDetails {
//     WebBucketFile(VirtualFileDetails, VirtualWebBucketFile),
//     NativeBucketFile(VirtualFileDetails, VirtualNativeBucketFile),
// }
// //https://stackoverflow.com/questions/49186751/sharing-a-common-value-in-all-enum-values
// impl Deref for VirtualFileDetails {
//     type Target = VirtualFileDetails;
//     fn deref(&self) -> &Self::Target {
//         match self {
//             VirtualFileDetails::WebBucketFile(n, _) => n,
//             VirtualFileDetails::NativeBucketFile(n, _) => n,
//         }
//     }
// }
