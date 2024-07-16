use bucket_common_types::BucketCompression;
use tonic::codegen::Bytes;

pub struct CompressionDetail {
    pub bucket_compression: BucketCompression,
}

pub trait Compressor {
    type Error;
    fn decompress_chunk(&self, bytes: &Bytes) -> Result<Vec<u8>, Self::Error>;

    fn decompress_stream();
}