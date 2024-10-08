use mime::FromStrError;
use std::convert::Infallible;
use std::sync::PoisonError;

use crate::wrapper::bucket::download::download_handler::BucketDownloadHandlerErrors;
use crate::wrapper::bucket::upload::upload_handler::BucketDownloadHandlerFileErrors;

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("Bucket does not exist")]
    BucketNotFound,
    #[error(transparent)]
    BucketDownloadHandlerErrors(#[from] BucketDownloadHandlerErrors),
    #[error(transparent)]
    ParsingUrlError(#[from] Infallible),
    #[error("Failed to parse bucket id")]
    ParseBucketIdError(#[source] uuid::Error),
    #[error("Failed to parse user id")]
    ParseUserIdError(#[source] uuid::Error),
    #[error("GetBucketDetailsRequestFailed")]
    GetBucketDetailsRequestFailed(#[source] tonic::Status),
    #[error("GetBucketDetailsFromUrlRequestFailed")]
    GetBucketDetailsFromUrlRequestFailed(#[source] tonic::Status),
    #[error(transparent)]
    FromStrError(#[from] FromStrError),
}
#[derive(Debug, thiserror::Error)]
pub enum UploadToUrlError {
    #[error("Http response error code: {0}")]
    HttpResponseStatusError(u16),
    #[error(transparent)]
    BucketDownloadHandlerErrors(#[from] BucketDownloadHandlerErrors),
    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error(transparent)]
    BucketDownloadHandlerFileError(#[from] BucketDownloadHandlerFileErrors),
}

#[derive(Debug, thiserror::Error)]
pub enum UploadError {
    #[error(transparent)]
    UploadToUrlError(#[from] UploadToUrlError),
    #[error(transparent)]
    ParseError(#[from] url::ParseError),
    #[error(transparent)]
    PoisonError(#[from] Box<dyn std::error::Error>),
    #[error("StorageNotAvailable")]
    StorageNotAvailable,
}

impl<T: 'static> From<PoisonError<T>> for UploadError {
    fn from(err: PoisonError<T>) -> Self {
        // Get details from the error you want,
        // or even implement for both T variants.
        Self::PoisonError(Box::new(err))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetFilesystemDetailsError {
    #[error(transparent)]
    TonicError(#[from] tonic::Status),
    #[error("Empty filesystem")]
    EmptyFilesystem,
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteFileInBucketError {
    #[error(transparent)]
    TonicError(#[from] tonic::Status),
    #[error("Failed to delete filepaths")]
    FailedToDeleteFilepath(Vec<String>),
}

#[derive(Debug, thiserror::Error)]
pub enum MoveFilesInBucketError {
    #[error("Failed to move file with filepath: {0:?}")]
    FailedToMoveFileFilepath(FailedFilePaths),
    #[error(transparent)]
    TonicStatus(#[from] tonic::Status),
}

pub struct FailedFilePaths(pub Vec<String>);

impl std::fmt::Debug for FailedFilePaths {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to move file with filepath: {:?}", self.0)
    }
}

impl From<FailedFilePaths> for MoveFilesInBucketError {
    fn from(value: FailedFilePaths) -> Self {
        MoveFilesInBucketError::FailedToMoveFileFilepath(value)
    }
}
