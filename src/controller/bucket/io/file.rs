use std::{
    io::{Read, Write},
    vec,
};
use infer::Type;

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
pub trait BucketFileTrait {
    type Error;
    type FileHandle;
    ///// Creates a new local file from the virtual file.
    //fn new_from_virtual(target: VirtualFileDetails) -> Result<Self, Self::Error>;
    ///// Use existing local file and map it to the virtual file.
    //fn from_existing(detail: Arc<VirtualFileDetails>, file_handle: Self::FileHandle) -> Self;
    //fn new(create_file_handle:fn() -> Self::FileHandle) -> Self;

    // DON'T IMPLEMENT THE CREATION OF NEW FILE, ONLY TAKE EXISTING "FILE HANDLE" ONE KEEP IT SIMPLE.

    fn from(file_handle: Self::FileHandle, filename: String) -> Self where Self: Sized;
    fn get_file_handle(&self) -> Self::FileHandle;
    async fn read_chunk(&self, size: u64, offset: u64) -> Result<Vec<u8>, Self::Error>;
    fn read_stream(&self) -> Result<Box<dyn Read>, Self::Error>;
    fn get_extension(&self) -> Result<String, Self::Error>;
    /// Get the mime-type from the extension.
    fn get_mime_type(&self) -> Result<Mime, Self::Error>;
    /// Uses the first couple of bytes in the file ot determine the mime-type
    async fn infer_mime_type(&self) -> Result<infer::Type, Self::Error>;
    fn write_chunk(&self, chunk: vec::Vec<u8>, offset: u64) -> Result<(), Self::Error>;
    fn write_stream(&self, stream: &dyn Write) -> Result<(), Self::Error>;
    fn get_size(&self) -> u64;
}

//#[cfg(all(not(feature = "native"), feature = "web"))]
#[cfg(target_family = "wasm")]
pub type BuketFile = VirtualWebBucketFile;
#[cfg(not(target_family = "wasm"))]
pub type BucketFile = VirtualNativeBucketFile;
/*

pub enum BucketFile {
    WebFile(VirtualWebBucketFile),
    #[cfg(feature = "native")]
    NativeFile(VirtualNativeBucketFile),
}

impl BucketFileTrait for BucketFile {
    type Error = WebBucketFileError;
    type FileHandle = WebFileHandle;

    fn from(file_handle: Self::FileHandle, filename: String) -> Self where Self: Sized {
        Self::WebFile(VirtualWebBucketFile::from(file_handle, filename))
    }

    fn get_file_handle(&self) -> Self::FileHandle {
        match self {
            BucketFile::WebFile(web) => {
                web.get_file_handle()
            }
            BucketFile::NativeFile(_) => { panic!() }
        }
    }

    async fn read_chunk(&self, size: u64, offset: u64) -> Result<Vec<u8>, Self::Error> {
        match self {
            BucketFile::WebFile(web) => {
                web.read_chunk(size,offset)
            }
            BucketFile::NativeFile(_) => { panic!() }
        }
    }

    fn read_stream(&self) -> Result<Box<dyn Read>, Self::Error> {
        match self {
            BucketFile::WebFile(web) => {
                web.read_stream()
            }
            BucketFile::NativeFile(_) => { panic!() }
        }
    }

    fn get_extension(&self) -> Result<String, Self::Error> {
        match self {
            BucketFile::WebFile(web) => {
                web.get_extension()
            }
            BucketFile::NativeFile(_) => { panic!() }
        }
    }

    fn get_mime_type(&self) -> Result<Mime, Self::Error> {
        match self {
            BucketFile::WebFile(web) => {
                web.get_mime_type()
            }
            BucketFile::NativeFile(_) => { panic!() }
        }
    }

    fn infer_mime_type(&self) -> Result<Type, Self::Error> {
        match self {
            BucketFile::WebFile(web) => {
                web.infer_mime_type()
            }
            BucketFile::NativeFile(_) => { panic!() }
        }
    }

    fn write_chunk(&self, chunk: Vec<u8>, offset: u64) -> Result<(), Self::Error> {
        match self {
            BucketFile::WebFile(web) => {
                web.write_chunk(chunk, offset)
            }
            BucketFile::NativeFile(_) => { panic!() }
        }
    }

    fn write_stream(&self, stream: &dyn Write) -> Result<(), Self::Error> {
        match self {
            BucketFile::WebFile(web) => {
                web.write_stream(stream)
            }
            BucketFile::NativeFile(_) => { panic!() }
        }
    }

    fn get_size(&self) -> u64 {
        match self {
            BucketFile::WebFile(web) => {
                web.get_size()
            }
            BucketFile::NativeFile(_) => { panic!() }
        }
    }
}

#[cfg(feature = "native")]
impl BucketFileTrait for VirtualNativeBucketFile {
    type Error = ();
    type FileHandle = ();

    fn from(file_handle: Self::FileHandle, filename: String) -> Self where Self: Sized {
        todo!()
    }

    fn get_file_handle(&self) -> Self::FileHandle {
        todo!()
    }

    async fn read_chunk(&self, size: u64, offset: u64) -> Result<Vec<u8>, Self::Error> {
        todo!()
    }

    fn read_stream(&self) -> Result<Box<dyn Read>, Self::Error> {
        todo!()
    }

    fn get_extension(&self) -> Result<String, Self::Error> {
        todo!()
    }

    fn get_mime_type(&self) -> Result<Mime, Self::Error> {
        todo!()
    }

    fn infer_mime_type(&self) -> Result<Type, Self::Error> {
        todo!()
    }

    fn write_chunk(&self, chunk: Vec<u8>, offset: u64) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_stream(&self, stream: &dyn Write) -> Result<(), Self::Error> {
        todo!()
    }

    fn get_size(&self) -> u64 {
        todo!()
    }
}*/