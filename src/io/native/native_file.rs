use mime::{FromStrError, Mime};
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use std::{io::Write, os::unix::prelude::FileExt, str::FromStr};
use std::io::Read;
use tonic::async_trait;
use crate::io::FileWrapper;

#[derive(Clone)]
pub struct NativeFile {
    //pub file_details: Arc<VirtualFileDetails>,
    //pub file_handle: Option<std::fs::File>,
    file_handle: Arc<std::fs::File>,
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

impl Read for NativeFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.file_handle.read(buf)
    }
}

impl Write for NativeFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.file_handle.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file_handle.flush()
    }
}

#[async_trait(?Send)]
impl FileWrapper for NativeFile {
    type Error = NativeBucketFileError;
    type FileHandle = std::fs::File;
    fn create_file(filename: &str, mime: &Mime) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let file = File::create(Path::new(filename))?;
        Ok(Self {
            file_handle: Arc::from(file),
            filename: filename.to_string(),
            file_type: mime.clone(),
        })
    }

    fn from_file_handle(file_handle: Self::FileHandle, filename: String, mime: &Mime) -> Self {
        Self {
            file_handle: Arc::new(file_handle),
            filename,
            file_type: mime.clone(),
        }
    }

    fn get_extension(&self) -> Option<String> {
        match self
            .filename
            .rsplit_once('.')
        {
            None => { None }
            Some((_,extension)) => { Some(extension.to_string()) }
        }
    }

    fn get_mime_type(&self) -> Result<Mime, Self::Error> {
        let extension = self.get_extension().unwrap();
        let mime = mime::Mime::from_str(extension.as_str())?;
        Ok(mime)
    }

    async fn infer_mime_type(&mut self) -> Result<infer::Type, Self::Error> {
        let mut buf = [1u8;16];
        self.file_handle.read_exact(&mut buf);
        let kind = infer::get(&buf);
        match kind {
            None => Err(NativeBucketFileError::UnknownInferredFileType),
            Some(kind) => Ok(kind),
        }
    }

    fn get_size(&self) -> u64 {
        self.file_handle.metadata().unwrap().len()
    }
}
