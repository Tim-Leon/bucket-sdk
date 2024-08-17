use std::io;
use bucket_common_types::BucketCompression;
use crate::compression::{CompressionChooserHandling, CompressorModule, DecompressModule};
use crate::compression::lz4::lz4_compression_module::Lz4CompressionModule;
use crate::compression::lz4::lz4_decompression_module::Lz4DecompressionModule;

pub struct DefaultCompressionChooserHandler {
}

impl<R: io::Read, W: io::Write> CompressionChooserHandling<R, W>
for DefaultCompressionChooserHandler
{
    fn chose_compression_handler(
        writer: W,
        bucket_compression: Option<BucketCompression>,
        use_client_side_compression: bool,
    ) -> Option<impl CompressorModule<R, W>> {
        if !use_client_side_compression {
            return None;
        }
        match bucket_compression {
            None => {
                None
            }
            Some(bucket_compression) => {
                match bucket_compression {
                    BucketCompression::Gzip => None,
                    BucketCompression::Brotli => None,
                    BucketCompression::Zstd => None,
                    BucketCompression::Lz4 => Some(Lz4CompressionModule::new(writer)),
                    BucketCompression::Custom(_) => None,
                }
            }
        }
    }

    fn choose_decompression_handler(
        reader: R,
        bucket_compression: Option<BucketCompression>,
        use_client_size_decompression: bool,
    ) -> Option<impl DecompressModule<R, W>> {
        if !use_client_size_decompression {
            return None;
        }
        match bucket_compression {
            None => {None}
            Some(bucket_compression) => {
                match bucket_compression {
                    BucketCompression::Gzip => None,
                    BucketCompression::Brotli => None,
                    BucketCompression::Zstd => None,
                    BucketCompression::Lz4 => Some(Lz4DecompressionModule::<R>::new(reader)),
                    BucketCompression::Custom(_) => None,
                }
            }
        }
    }

    fn get_supported_compression_algorithms(&self) -> &[BucketCompression] {
        &[BucketCompression::Lz4]
    }

    fn get_supported_decompression_algorithms(&self) -> &[BucketCompression] {
        &[BucketCompression::Lz4]
    }
}
