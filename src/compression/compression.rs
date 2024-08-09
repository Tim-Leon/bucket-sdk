use std::io;
use std::io::{Cursor, Read, Write};
use bucket_common_types::BucketCompression;
use lz4_flex::frame::FrameEncoder;
use prost::bytes::Bytes;
use crate::compression::CompressorModule;

pub struct DefaultCompressionModule<W: Write> {
    pub bucket_compression: BucketCompression,
    pub use_client_side_compression: bool,
    pub buf: Vec<u8>,
    pub lz4_compression: lz4_flex::frame::FrameEncoder<W>,
}

#[derive(thiserror::Error, Debug)]
pub enum DefaultCompressionError {

}

impl<R: std::io::Read, W: std::io::Write> CompressorModule<R, W> for DefaultCompressionModule<R, W> {
    type Error = DefaultCompressionError;

    fn new(writer: W,bucket_compression: BucketCompression, use_client_side_compression: bool) -> Self {
        Self {
            bucket_compression,
            use_client_side_compression,
            buf: Vec::with_capacity(1024),
            lz4_compression: FrameEncoder::new(writer),
        }
    }

    fn compress_chunk(&self, bytes: &Bytes) -> Result<Vec<u8>, Self::Error> {
        todo!()
    }

    fn compress_stream(&self, reader: R, writer: W) -> Result<(), Self::Error> {
        todo!()
    }

    fn get_supported_compression() -> &'static [BucketCompression] {

    }
}
