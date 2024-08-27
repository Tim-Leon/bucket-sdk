use crate::compression::{CompressorModule, DecompressModule};
use async_trait::async_trait;
use bucket_common_types::{BucketCompression, BucketEncryption, Encryption};
use futures::future::Either;
use generic_array::ArrayLength;
use mime::FromStrError;
use std::marker::PhantomData;
use uuid::Uuid;
use zero_knowledge_encryption::encryption::aead::decryption_module::DecryptionError;
use zero_knowledge_encryption::encryption::aead::DecryptionModule;
use crate::io::FileWrapper;
use crate::wrapper::bucket::download::FileDownloadHandler;

#[derive(Debug, thiserror::Error)]
pub enum BucketDownloadHandlerErrors {
    #[error(transparent)]
    FromStrError(#[from] FromStrError),
    #[error(transparent)]
    DecryptionError(#[from] DecryptionError),
}



//#[derive(Clone)]
pub struct WebBucketFileWriter<
    R: std::io::Read,
    W: std::io::Write,
    N: ArrayLength,
    DE: DecryptionModule<R, W, N>,
    DC: DecompressModule<R, W>,
    BF: FileWrapper<Error=(), FileHandle=()>
> {
    phantom_data_r: PhantomData<R>,
    phantom_data_w: PhantomData<W>,
    phantom_data: PhantomData<N>,
    //write_target_file: gloo::file::File,
    write_target_file: BF,
    pub offset: u64,
    pub decryption_module: Option<DE>,
    pub decompression_module: Option<DC>,
    // Will be none if no encryption was used. Everything encryption related is handled by the module.
    pub is_checking_signature: bool, // TODO: Add support for this feature. The file will be checked against another file with special signature to ensure the signature matches against the supplied signature. This ensures the content can't be tampered with.
}

pub struct DownloadStartParams {
    target_bucket_id: uuid::Uuid,
    target_user_id: Uuid,
    from_directory: String,
    from_filename: String,
    /// The bucket might not be encrypted, in that case this will return empty.
    encryption: Option<Encryption>,
    /// Just like encryption might be no compression for the bucket.
    bucket_compression: Option<BucketCompression>,
    /// The total size of the download.
    download_size_in_bytes: u64,
    size_left_in_bytes: u64,
}

#[async_trait(? Send)]
impl<R: std::io::Read, W: std::io::Write, N: generic_array::ArrayLength, DE: DecryptionModule<R, W, N>, DC: DecompressModule<R, W>, BF: FileWrapper<Error = (), FileHandle = ()>>
    FileDownloadHandler<R, W> for WebBucketFileWriter<R, W, N, DE, DC, BF>
{
    type Error = BucketDownloadHandlerErrors;

    // Called when a chunk is downloaded. It's up to the user to decrypt the chunk if the bucket is encrypted, or to save the chunk to a file.
    async fn on_download_chunk(&mut self, chunk: &mut [u8]) -> Result<(), Self::Error> {
        let decompressed_buffer;
        match &mut self.decryption_module {
            None => {}
            Some(de) => {
                let size = de.decrypt_block(chunk).unwrap();
            }
        };
        match &mut self.decompression_module {
            None => {}
            Some(dm) => {
                dm.decompress_chunk(chunk).unwrap();
            }
        };


        let decrypt_buffer;

        let decrypted_buffer: futures::future::Either<Vec<u8>, &Vec<u8>> =
            match &mut self.decryption_module {
                Some(x) => {
                    let mut decrypted_buffer: Vec<u8> = chunk.clone();
                    decrypted_buffer = x.update(chunk).unwrap();
                    Either::Left(decrypted_buffer)
                }
                None => Either::Right(chunk),
            };
        match decrypted_buffer {
            Either::Left(decrypted_buffer) => {
                self.write_target_file.write(&decrypted_buffer.as_slice()[(self.offset as usize)..(decrypted_buffer.len() - (self.offset as usize))])
                    .unwrap();
                self.offset += decrypted_buffer.len() as u64;
            }
            Either::Right(decrypted_buffer) => {
                self.write_target_file
                    .write_chunk(&decrypted_buffer, self.offset)
                    .unwrap();
                self.offset += decrypted_buffer.len() as u64;
            }
        }

        /*
         * Create custom object for URL to download the file.
         * The file will be stored as a "xxxx.temp" file. After the download the file is renamed to the correct filename.
         */

        //let reader = read_as_bytes(&self.write_target_file.slice(start, end)).await?;
        // let reader = read_as_array_buffer(blob, |buffer, |{
        //     Self.to_owned().on_download_chunk(r_bytes);
        //     let bytes = js_sys::Uint8Array::new(&buffer).to_vec();
        //     let r_bytes = byte;
        // });

        //let buffer = reader.as_slice();

        Ok(())
    }
    // Called when the last chunk has been downloaded.
    fn on_download_finish(self) -> Result<(), Self::Error> {
        //TODO: Check if file match checksums.
        match self.decryption_module {
            None => {}
            Some(module) => {
                module.finalize(); //TODO: Invalid Signature
            }
        }

        Ok(())
    }
}
