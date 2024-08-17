use crate::compression::CompressorModule;
use bucket_common_types::BucketCompression;
use lz4_flex::frame::{AutoFinishEncoder, FrameEncoder};
use prost::bytes::Bytes;
use std::io;
use std::io::{Cursor, Read, Write};

pub struct Lz4CompressionModule<W: Write> {
    pub buf: Vec<u8>,
    pub lz4_compression: lz4_flex::frame::FrameEncoder<W>,
}

#[derive(thiserror::Error, Debug)]
pub enum DefaultCompressionError {}

impl<R: std::io::Read, W: std::io::Write> CompressorModule<R, W> for Lz4CompressionModule<W> {
    type Error = DefaultCompressionError;

    fn new(writer: W) -> Self {
        Self {
            buf: Vec::with_capacity(1024),
            lz4_compression: FrameEncoder::new(writer),
        }
    }

    fn compress_chunk(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.lz4_compression.write_all(bytes).unwrap();
        Ok(())
    }

    fn compress_stream(&mut self, mut reader: R) -> Result<(), Self::Error> {
        let buf = &mut self.buf; // Use the internal buffer
        loop {
            // Read data into the buffer
            let bytes_read = reader.read(buf).unwrap();
            if bytes_read == 0 {
                break; // End of stream
            }

            // Write the read data to the lz4 compressor
            self.lz4_compression.write_all(&buf[..bytes_read]).unwrap();
        }
        Ok(())
    }

    fn finish(self) -> Result<(), Self::Error> {
        self.lz4_compression.finish().unwrap();
        Ok(())
    }

    fn get_compression_algorithm() -> &'static BucketCompression {
        &BucketCompression::Lz4
    }
}
