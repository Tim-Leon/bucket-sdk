use std::io::{Read, Write};
use bucket_common_types::BucketCompression;
use prost::bytes::Bytes;
use crate::compression::DecompressModule;

pub struct DefaultDecompressionModule {
    pub buf: Vec<u8>,
}

impl DecompressModule for DefaultDecompressionModule {
    type Error = ();

    fn new(bucket_compression: BucketCompression, use_client_side_decompression: bool) -> Self {
        Self {
            buf: Vec::with_capacity(1024),
        }
    }

    fn decompress_chunk(&self, bytes: &Bytes) -> Result<Vec<u8>, Self::Error> {
        todo!()
    }

    fn decompress_stream<R: Read, W: Write>(&self, reader: R, writer: W) -> Result<(), Self::Error> {

    }

    fn get_supported_decompression() -> &'static [BucketCompression] {
        todo!()
    }
}