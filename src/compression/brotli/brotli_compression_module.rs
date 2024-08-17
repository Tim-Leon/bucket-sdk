use std::io::{Read, Write};
use brotli::CompressorWriter;
use bucket_common_types::BucketCompression;
use crate::compression::CompressorModule;

pub struct BrotliCompressionModule<W: Write> {
    compressor_writer: CompressorWriter<W>,
    buf: Vec<u8>,
}
#[derive(thiserror::Error, Debug)]
pub enum BrotliCompressionModuleError {

}



impl<R:Read, W:Write> CompressorModule<R,W>  for BrotliCompressionModule<W>{
    type Error = BrotliCompressionModuleError;

    fn new(writer: W) -> Self {
        Self {
            compressor_writer: writer,
            buf: Vec::with_capacity(1024),
        }
    }

    fn compress_chunk(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.compressor_writer.write(bytes).unwrap();
        Ok(())
    }

    fn compress_stream(&mut self, reader: R) -> Result<(), Self::Error> {
        self.compressor_writer.write(reader).unwrap();
        Ok(())
    }

    fn finish(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn get_compression_algorithm() -> &'static BucketCompression {
        &BucketCompression::Brotli
    }
}