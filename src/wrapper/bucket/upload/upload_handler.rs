use crate::compression::{CompressorModule, DecompressModule};
use crate::io::file::FileWrapper;
use crate::wrapper::bucket::upload::BucketFileUploadHandler;
use async_trait::async_trait;
use bucket_common_types::{BucketCompression, BucketEncryption};
use generic_array::ArrayLength;
use std::fmt::Debug;
use std::io::{Read, Write};
use std::marker::PhantomData;
use zero_knowledge_encryption::encryption::aead::encryption_module::EncryptionError;
use zero_knowledge_encryption::encryption::aead::EncryptionModule;
use crate::io::FileWrapper;

#[derive(Debug, thiserror::Error)]
pub enum BucketDownloadHandlerFileErrors {
    #[error("Encryption module not initialized when bucket is encrypted.")]
    EncryptionModuleNotInitialized,
    #[error(transparent)]
    EncryptionError(#[from] EncryptionError),
}

#[derive(Clone)]
pub struct BucketFileReader<
    R: Read,
    W: Write,
    N: ArrayLength,
    EC: EncryptionModule<R, W, N>,
    CM: CompressorModule<R, W>,
    BF: FileWrapper
> {
    phantom_data_r: PhantomData<R>,
    phantom_data_w: PhantomData<W>,
    phantom_data_n: PhantomData<N>,
    pub read_target_file: BF,
    pub encryption_module: Option<EC>,
    pub compression_module: Option<CM>,
    pub offset: u64,
    pub bucket_compression: BucketCompression,
    pub use_client_compression: bool,
}



#[async_trait(?Send)]
impl<R: std::io::Read, W: std::io::Write, N: ArrayLength, EM: EncryptionModule<R, W, N>, CM: CompressorModule<R, W>, BF: FileWrapper>
    BucketFileUploadHandler<R, W> for BucketFileReader<R, W, N, EM, CM, BF>
{
    //BucketDownloadHandlerFile
    type Error = BucketDownloadHandlerFileErrors;

    fn new<N: ArrayLength,BF: FileWrapper, EM: EncryptionModule<R, W, N>, CM: CompressorModule<R, W>>(read_target_file: BF, encryption_module: Option<EM>, compression_module: Option<CM>, bucket_compression: BucketCompression, use_client_compression: bool) -> Result<Self, Self::Error> {
        Ok(Self {
            phantom_data_r: Default::default(),
            phantom_data_w: Default::default(),
            phantom_data_n: Default::default(),
            read_target_file,
            encryption_module,
            compression_module,
            offset: 0,
            bucket_compression,
            use_client_compression,
        })
    }

    fn on_upload_start(
        &self,
        target_bucket_id: uuid::Uuid,
        target_user_id: uuid::Uuid,
        to_directory: String,
        to_filename: String,
        encryption: Option<BucketEncryption>,
        bucket_compression: Option<BucketCompression>,
        use_client_side_compression: bool,
        upload_size_in_bytes: u64,
    ) -> Result<u64, Self::Error> {
        match encryption {
            Some(_bucket_encryption) => match &self.encryption_module {
                Some(_encryption_module) => {}
                None => {
                    return Err(BucketDownloadHandlerFileErrors::EncryptionModuleNotInitialized);
                }
            },
            None => {}
        };

        let size = self.read_target_file.get_size();
        Ok(size)
    }

    async fn on_upload_chunk(&mut self, chunk_size: u64) -> Result<Vec<u8>, Self::Error> {
        let mut buffer = Vec::with_capacity(chunk_size as usize);
        self.read_target_file.read(&mut buffer.as_slice()).await.unwrap();

        match &mut self.compression_module {
            None => {}
            Some(compression_module) => {
                compression_module.compress_chunk(&mut buffer.as_slice()).unwrap();
            }
        }

        match &mut self.encryption_module {
            Some(x) => {
                let encrypted_bytes = x.encrypt_stream(bytes).unwrap();
                Ok(encrypted_bytes)
            }
            None => Ok(bytes),
        }
    }

    fn on_upload_finish(self) -> Result<(), Self::Error> {
        let signed_hash = match self.encryption_module {
            Some(x) => Some(x.finalize()),
            None => return Err(Self::Error::EncryptionModuleNotInitialized),
        };
        Ok(())
    }
}
