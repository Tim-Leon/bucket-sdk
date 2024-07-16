use crate::controller::bucket::io::file::{BucketFile, BucketFileTrait};
use async_trait::async_trait;
use bucket_common_types::BucketEncryption;
use futures::future::Either;
use mime::{FromStrError, Mime};
use std::str::FromStr;
use zero_knowledge_encryption::encryption::aead::decryption_module::{DecryptionError, ZeroKnowledgeDecryptionModuleV1};
use zero_knowledge_encryption::encryption::aead::DecryptionModule;

#[derive(Debug, thiserror::Error)]
pub enum BucketDownloadHandlerErrors {
    #[error(transparent)]
    FromStrError(#[from] FromStrError),
    #[error(transparent)]
    FileReaderError(#[from] gloo::file::FileReadError),
    #[error(transparent)]
    DecryptionError(#[from] DecryptionError),
}

#[derive(Debug, thiserror::Error)]
pub enum BucketUploadHandlerErrors {}

// A handler is create for each file download.
#[async_trait(? Send)]
pub trait BucketFileDownloadHandler {
    type Error: std::error::Error + Send + Sync + 'static;
    type DecryptionModule: DecryptionModule;

    fn on_download_start(
        &mut self,
        target_bucket_id: uuid::Uuid,
        target_user_id: uuid::Uuid,
        from_directory: String,
        from_filename: String,
        encryption: Option<BucketEncryption>,
        download_size_in_bytes: u64,
    ) -> Result<(), Self::Error>;
    // Called when a chunk is downloaded. It's up to the user to decrypt the chunk if the bucket is encrypted, or to save the chunk to a file.
    async fn on_download_chunk(&mut self, chunk: &Vec<u8>) -> Result<(), Self::Error>;
    // Called when the last chunk has been downloaded.
    fn on_download_finish(self) -> Result<(), Self::Error>;
}

//#[derive(Clone)]
pub struct WebBucketFileWriter {
    //write_target_file: gloo::file::File,
    write_target_file: Box<BucketFile>,
    pub offset: u64,
    pub decryption_module: Option<ZeroKnowledgeDecryptionModuleV1>,
    // Will be none if no encryption was used. Everything encryption related is handled by the module.
    pub is_checking_signature: bool, // TODO: Add support for this feature. The file will be checked against another file with special signature to ensure the signature matches against the supplied signature. This ensures the content can't be tampered with.
}

#[async_trait(? Send)]
impl BucketFileDownloadHandler for WebBucketFileWriter {
    type Error = BucketDownloadHandlerErrors;
    type DecryptionModule = ZeroKnowledgeDecryptionModuleV1;

    fn on_download_start(
        &mut self,
        _target_bucket_id: uuid::Uuid,
        _target_user_id: uuid::Uuid,
        _from_directory: String,
        from_filename: String,
        _encryption: Option<BucketEncryption>,
        _download_size_in_bytes: u64,
    ) -> Result<(), Self::Error> {
        //let blob = Blob::from(self.write_target_file.clone());
        //let bytes = read_as_bytes(&blob).await?;
        let file = BucketFile::new(
            from_filename.as_str(),
            &Mime::from_str("application/octet-stream").unwrap(),
        )
        .unwrap();
        Ok(())
    }
    // Called when a chunk is downloaded. It's up to the user to decrypt the chunk if the bucket is encrypted, or to save the chunk to a file.
    async fn on_download_chunk(&mut self, chunk: &Vec<u8>) -> Result<(), Self::Error> {
        let decrypted_buffer: futures::future::Either<Vec<u8>, &Vec<u8>> =
            match &mut self.decryption_module {
                Some(x) => {
                    let mut decrypted_buffer: Vec<u8> = chunk.clone();
                    decrypted_buffer = x.update(chunk)?;
                    Either::Left(decrypted_buffer)
                }
                None => Either::Right(chunk),
            };
        match decrypted_buffer {
            Either::Left(decrypted_buffer) => {
                self.write_target_file
                    .write_chunk(&decrypted_buffer, self.offset).unwrap();
                self.offset += decrypted_buffer.len() as u64;
            }
            Either::Right(decrypted_buffer) => {
                self.write_target_file
                    .write_chunk(&decrypted_buffer, self.offset).unwrap();
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
