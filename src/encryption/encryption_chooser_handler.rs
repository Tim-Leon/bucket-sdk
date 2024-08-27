use std::io::{Read, Write};
use bucket_common_types::{BucketEncryption, Role};
use generic_array::ArrayLength;
use zero_knowledge_encryption::encryption::aead::EncryptionModule;
use crate::compression::DecompressModule;
use crate::encryption::EncryptionChooserHandler;

pub struct DefaultEncryptionChooserHandler {

}

#[derive(Debug, thiserror::Error)]
pub enum EncryptionChooserHandlerError {
    #[error("RequiredClientSideEncryptionNotSupported")]
    RequiredClientSideEncryptionNotSupported,
}

impl <R:Read,W:Write,N:ArrayLength> EncryptionChooserHandler<R, W, N> for DefaultEncryptionChooserHandler {
    type Error = EncryptionChooserHandlerError;
    fn chose_encryption_handler(&self, bucket_encryption: Option<BucketEncryption>, allow_client_side_encryption: bool) -> Result<Option<impl EncryptionModule<R, W, N>>, Self::Error> {
        match bucket_encryption {
            None => {Ok(None)}
            Some(bucket_encryption) => {
                match bucket_encryption {
                    BucketEncryption { responsible, encryption, signature, version } => {
                        match responsible {
                            Role::Server => {
                                Ok(None)
                            }
                            Role::Client => {
                                /// Client side encryption must be allowed for the client to be able to encrypt to the bucket.
                                if !allow_client_side_encryption{
                                    return Err(EncryptionChooserHandlerError::RequiredClientSideEncryptionNotSupported);
                                }


                                for supported_encryption_algorithm in self.get_supported_encryption_algorithms() {
                                    if supported_encryption_algorithm.encryption == encryption && version == supported_encryption_algorithm.version{

                                    }
                                }

                                Ok(None)
                            }
                        }
                    }
                }
            }
        }
    }

    fn chose_decryption_handler(&self, bucket_encryption: Option<BucketEncryption>, allow_client_side_decryption: bool) -> Result< Option<impl DecompressModule<R, W>> ,Self::Error>  {
        match bucket_encryption {
            None => { Ok(None) }
            Some(bucket_encryption) => { match bucket_encryption {
                BucketEncryption { responsible, encryption, signature, version } => {
                    match responsible {
                        Role::Server => {
                            Ok(None)
                        }
                        Role::Client => {
                            if !allow_client_side_decryption {
                                return Err(EncryptionChooserHandlerError::RequiredClientSideEncryptionNotSupported);
                            }
                            match self.get_supported_encryption_algorithms() {
                                &x => {
                                    
                                }
                            }
                        }
                        }
                }
            } }
        }
    }


    fn get_supported_encryption_algorithms(&self) -> &[BucketEncryption] {
        &[BucketEncryption {
            version: 0,
            responsible: Role::Server,
            encryption: ,
        }]
    }

    fn get_supported_decryption_algorithms(&self) -> &[BucketEncryption] {
        &[BucketEncryption {
            responsible: Role::Server,
            version: 0,
        }]
    }
}