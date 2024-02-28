use mime::{FromStrError, Mime};
use std::fs::File;
use std::path::Path;
use std::{io::Write, os::unix::prelude::FileExt, str::FromStr};
use tonic::async_trait;

use super::file::BucketFileTrait;

pub struct VirtualNativeBucketFile {
    //pub file_details: Arc<VirtualFileDetails>,
    //pub file_handle: Option<std::fs::File>,
    file_handle: std::fs::File,
    filename: String, // Must have because, unix system lack the ability to reverse file descriptor into a path.
    file_type: Mime,
}
#[derive(thiserror::Error, Debug)]
pub enum NativeBucketFileError {
    #[error("NoFileToRead")]
    NoFileToRead,
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("missing file extension")]
    MissingFileExtension,
    #[error("UnknownInferredFileType")]
    UnknownInferredFileType,
    #[error(transparent)]
    FromStrError(#[from] FromStrError),
}

#[async_trait(?Send)]
impl BucketFileTrait for VirtualNativeBucketFile {
    type Error = NativeBucketFileError;

    type FileHandle = std::fs::File;
    fn new(filename: &str, mime: &Mime) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let file = File::create(Path::new(filename))?;
        Ok(Self {
            file_handle: file,
            filename: filename.to_string(),
            file_type: mime.clone(),
        })
    }

    fn from(file_handle: Self::FileHandle, filename: String, mime: &Mime) -> Self {
        Self {
            file_handle,
            filename,
            file_type: mime.clone(),
        }
    }

    fn get_file_handle(&self) -> &Self::FileHandle {
        &self.file_handle
    }

    async fn read_chunk(&self, size: u64, offset: u64) -> Result<Vec<u8>, Self::Error> {
        let mut chunk = Vec::<u8>::with_capacity(size as usize);
        self.file_handle.read_exact_at(&mut chunk, offset).unwrap();
        Ok(chunk)
    }

    fn read_stream(&self) -> Result<Box<dyn std::io::prelude::Read>, Self::Error> {
        todo!()
    }

    fn get_extension(&self) -> Result<String, Self::Error> {
        let (_, extension) = self
            .filename
            .rsplit_once('.')
            .ok_or(NativeBucketFileError::MissingFileExtension)?;
        Ok(extension.to_string())
    }

    fn get_mime_type(&self) -> Result<Mime, Self::Error> {
        let extension = self.get_extension()?;
        let mime = mime::Mime::from_str(extension.as_str())?;
        Ok(mime)
    }

    async fn infer_mime_type(&self) -> Result<infer::Type, Self::Error> {
        let buf = self.read_chunk(16, 0).await.unwrap();
        let kind = infer::get(&buf);
        match kind {
            None => Err(NativeBucketFileError::UnknownInferredFileType),
            Some(kind) => Ok(kind),
        }
    }

    fn write_chunk(&self, chunk: &std::vec::Vec<u8>, offset: u64) -> Result<(), Self::Error> {
        todo!();
        self.file_handle.write_all(&chunk)?;
        Ok(())
    }

    fn write_stream(&self, stream: &dyn std::io::prelude::Write) -> Result<(), Self::Error> {
        todo!();
        //stream.write_all(&self.read_chunk(u64::MAX, 0)?)?;
        Ok(())
    }

    fn get_size(&self) -> u64 {
        self.file_handle.metadata().unwrap().len()
    }
}
