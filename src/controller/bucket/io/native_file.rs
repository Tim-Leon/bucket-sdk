use std::{os::unix::prelude::FileExt, sync::Arc};
use mime::Mime;

use super::file::{BucketFileTrait, VirtualFileDetails};

struct VirtualNativeBucketFile {
    pub file_details: Arc<VirtualFileDetails>,
    pub file_handle: Option<std::fs::File>,
}
#[derive(thiserror::Error, Debug)]
pub enum NativeBucketFileError {
    #[error("NoFileToRead")]
    NoFileToRead,
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}
impl BucketFileTrait for VirtualNativeBucketFile {
    type Error = NativeBucketFileError;
    type FileHandle = std::fs::File;

    fn new(detail: Arc<VirtualFileDetails>, file_handle: Option<Self::FileHandle>) -> Self {
        Self {
            file_details: detail,
            file_handle,
        }
    }

    fn is_readable(&self) -> bool {
        self.file_handle.is_some()
    }

    fn get_file_handle(&self) -> &Option<Self::FileHandle> {
        &self.file_handle
    }

    fn read_chunk(&self, size: u64, offset: u64) -> Result<Vec<u8>, Self::Error> {
        let mut buffer = Vec::with_capacity(size as usize);
        match &self.file_handle {
            None => Err(NativeBucketFileError::NoFileToRead),
            Some(file) => {
                file.read_at(buffer.as_mut_slice(), offset)?;
                Ok(buffer)
            }
        }
    }


    

    fn get_extension(&self) -> Result<String, Self::Error> {
        todo!()
    }

    fn get_mime_type(&self) -> Result<Mime, Self::Error> {
        todo!()
    }

    fn infer_mime_type(&self) -> Result<infer::Type, Self::Error> {
        todo!()
    }

    fn write_chunk(&self, chunk: std::vec::Vec::<u8>, offset: u64) -> Result<(), Self::Error> {
        todo!()
    }

    fn write_stream(&self, stream: &dyn std::io::prelude::Write) -> Result<(),Self::Error> {
        todo!()
    }

    fn get_size(&self) -> Option<u64> {
        todo!()
    }

    fn read_stream(&self) -> Result<Box<dyn std::io::prelude::Read>, Self::Error> {
        todo!()
    }


    

}
