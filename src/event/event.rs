pub enum BucketEvent {
    FilesUploaded(Vec<String>),
    FilesDeleted(Vec<String>),
    FilesModified(Vec<String>),
    FilesMoved(Vec<String>, String),
    FilesCopied(Vec<String>, String),
    FilesRenamed(Vec<String>, String),
    UserDebited(rust_decimal::Decimal),
    BucketExpired(time::OffsetDateTime),
    BucketDeleted(uuid::Uuid, String),
    BucketUpdated(String),
}

//pub fn parse_bucket_event()
