pub mod lz4;
mod default_compression_chooser_handler;
mod brotli;

use std::fmt::Debug;
use bucket_common_types::BucketCompression;
use std::io;
use std::io::{Cursor, Read, Write};
use tonic::codegen::Bytes;
use crate::compression::lz4::lz4_compression_module::Lz4CompressionModule;
use crate::compression::lz4::lz4_decompression_module::Lz4DecompressionModule;

/// When doing compression, the client will get to choose the compression module, this behaviour can be changed by overiding the handler to other

// TODO: Have trait restricted to read write.
pub trait CompressorModule<R: Read, W: Write>: Sized {
    type Error: Debug;

    ///
    ///
    /// # Arguments
    ///
    /// * `writer`: The output writer where compressed data will be written.
    ///
    /// returns: Self
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    fn new(writer: W) -> Self;
    fn compress_chunk(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;
    /// Compresses a stream of data from a reader and writes the compressed data to a writer.
    fn compress_stream(&mut self, reader: R) -> Result<(), Self::Error>;
    fn finish(self) -> Result<(), Self::Error>;

    fn get_compression_algorithm() -> &'static BucketCompression;
}
// TODO: Have trait restricted to read write.
pub trait DecompressModule<R: Read, W: Write>: Sized {
    type Error: Debug;
    fn new(reader: R) -> Self;
    fn decompress_chunk(&mut self, bytes: &mut [u8]) -> Result<(), Self::Error>;
    fn decompress_stream(&mut self, writer: W) -> Result<(), Self::Error>;
    fn get_decompression_algorithm() -> &'static BucketCompression;
}

/// Decide which compression algorithm/decompression is used
pub trait CompressionChooserHandling<R: Read, W: Write> {
    fn chose_compression_handler(
        writer: W,
        bucket_compression: Option<BucketCompression>,
        use_client_side_compression: bool,
    ) -> Option<impl CompressorModule<R, W>>;

    fn choose_decompression_handler(
        reader: R,
        bucket_compression: Option<BucketCompression>,
        use_client_size_decompression: bool,
    ) -> Option<impl DecompressModule<R, W>>;

    fn get_supported_compression_algorithms(&self) -> &[BucketCompression];

    fn get_supported_decompression_algorithms(&self) -> &[BucketCompression];
}

