use crate::compression::DecompressModule;
use bucket_common_types::BucketCompression;
use prost::bytes::Bytes;
use std::io;
use std::io::{Read, Write};
use tokio::io::AsyncWriteExt;

pub struct Lz4DecompressionModule<R: Read> {
    pub decoder: lz4_flex::frame::FrameDecoder<R>,
    pub buf: Vec<u8>,
}
#[derive(Debug, thiserror::Error)]
pub enum Lz4DecompressionError {
    #[error("IoError")]
    IoError,
}

impl<R: io::Read, W: io::Write> DecompressModule<R, W> for Lz4DecompressionModule<R> {
    type Error = Lz4DecompressionError;

    fn new(reader: R) -> Self {
        Self {
            decoder: lz4_flex::frame::FrameDecoder::new(reader),
            buf: vec![],
        }
    }

    fn decompress_chunk(&mut self, output: &mut [u8]) -> Result<(), Self::Error> {
        self.decoder.read(output).unwrap();
        Ok(())
    }

    fn decompress_stream(&mut self, mut output: W) -> Result<(), Self::Error> {
        // Continuously decompress the stream until the end
        loop {
            let bytes_read = match self.decoder.read(&mut self.buf) {
                Ok(bytes_read) => {bytes_read}
                Err(_) => {return Err(Lz4DecompressionError::IoError)?}
            };
            if bytes_read == 0 {
                break;
            }
            output.write_all(&self.buf[..bytes_read]).unwrap();
        }
        Ok(())
    }

    fn get_decompression_algorithm() -> &'static BucketCompression {
        &BucketCompression::Lz4
    }
}
