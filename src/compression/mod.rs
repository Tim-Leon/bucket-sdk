pub mod compression;
pub mod decompression;

use std::io::Cursor;
use bucket_common_types::BucketCompression;
use tonic::codegen::Bytes;


pub trait CompressorModule<R: std::io::Read, W: std::io::Write> {
    type Error;
    /// client compression if enabled will ensure the client is responsible for all compressions whenever
    /// BucketCompression is set.
    fn new(writer: W, bucket_compression: BucketCompression, use_client_side_compression: bool) -> Self;
    fn compress_chunk(&self, bytes: &Bytes) -> Result<Vec<u8>, Self::Error>;
    /// Compresses a stream of data from a reader and writes the compressed data to a writer.
    fn compress_stream(
        &self,
        reader: R,
    ) -> Result<(), Self::Error>;
    fn get_supported_compression() -> &'static [BucketCompression];
}

pub trait DecompressModule {
    type Error;
    fn new(bucket_compression: BucketCompression, use_client_side_decompression: bool) -> Self;
    fn decompress_chunk(&self, bytes: &Bytes) -> Result<Vec<u8>, Self::Error>;
    fn decompress_stream<R: std::io::Read, W: std::io::Write>(
        &self,
        reader: R,
        writer: W,
    ) -> Result<(), Self::Error>;
    fn get_supported_decompression() -> &'static [BucketCompression];
}