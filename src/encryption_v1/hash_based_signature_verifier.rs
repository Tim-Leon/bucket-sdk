use ed25519_compact::Signature;


pub trait HashBasedSignatureVerifier: Clone + Sized {
    type Error; 

    fn verify_hash(self, hash: impl AsRef<[u8]>) -> Result<(),Self::Error>;
}
#[derive(thiserror::Error, Debug)]
pub enum Ed25519HighwayHashBasedSignatureVerifierError {
    #[error(transparent)]
    SignatureValidationError(#[from] ed25519_compact::Error),
}

#[derive(Clone)]
pub struct Ed25519HighwayHashBasedSignatureVerifier { 
    pub signature: Signature, 
    pub public_key: ed25519_compact::PublicKey,
}

impl HashBasedSignatureVerifier for Ed25519HighwayHashBasedSignatureVerifier {
    type Error = Ed25519HighwayHashBasedSignatureVerifierError;
    
    fn verify_hash(self, hash: impl AsRef<[u8]>) -> Result<(),Self::Error> {
        Ok(self.public_key.verify(hash, &self.signature)?)
    }
}

impl Ed25519HighwayHashBasedSignatureVerifier {
    fn new(
        signature: Signature,
        pk: ed25519_compact::PublicKey,
    ) -> Result<Self, Ed25519HighwayHashBasedSignatureVerifierError>{
        Ok(Self {
            signature,
            public_key: pk,
        })
    }
}
