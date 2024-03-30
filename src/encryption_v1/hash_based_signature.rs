use ed25519_compact::{Noise, Signature, VerifyingState};
use highway::{HighwayHash, HighwayHasher, Key};

use super::{
    constants::{HIGHWAY_HASH_KEY, SHARE_LINK_SIGNATURE_NOISE},
    encryption::MasterKey,
};
// Used for creating signatures
pub trait HashBasedSignature: Clone + Sized {
    type Error;

    fn update(&mut self, ciphertext: impl AsRef<[u8]>);
    // Creates a signature. 
    fn finalize(self) -> Result<Signature, Self::Error>;
}

#[derive(Clone)]
pub struct Ed25519HighwayHashBasedSignature {
    highway_hash: highway::HighwayHasher,
    secret_key: ed25519_compact::SecretKey,
    ed25519_noise: Noise,
    //signature: Option<Signature>,
}
#[derive(thiserror::Error, Debug)]
pub enum Ed25519HighwayHashBasedSignatureError {
    #[error(transparent)]
    Ed25519Error(#[from] ed25519_compact::Error),
}

impl Ed25519HighwayHashBasedSignature {
    // Either we are creating a signature or we provide a signature to verify.
    fn new(
        //master_key: &MasterKey,
        secret_key: ed25519_compact::SecretKey,
        //signature: Option<Signature>,
    ) -> Result<Self, Ed25519HighwayHashBasedSignatureError> {
        //let seed = ed25519_compact::Seed::from_slice(master_key.0.as_bytes()).unwrap(); //[0..32]
        //let ed25519_signing_key_pair = ed25519_compact::KeyPair::from_seed(seed); //from_slice(master_key.as_bytes().take).unwrap();
        let key = highway::Key(HIGHWAY_HASH_KEY);
        let highway_hash = highway::HighwayHasher::new(key);
        let ed25519_noise =
            ed25519_compact::Noise::from_slice(&SHARE_LINK_SIGNATURE_NOISE).unwrap();

        Ok(Ed25519HighwayHashBasedSignature {
            highway_hash,
            secret_key: secret_key,
            ed25519_noise,
            //signature,
        })
    }
}

impl HashBasedSignature for Ed25519HighwayHashBasedSignature {
    type Error = Ed25519HighwayHashBasedSignatureError;

    fn update(&mut self, ciphertext: impl AsRef<[u8]>) {
        self.highway_hash.append(ciphertext.as_ref())
    }

    fn finalize(self) -> Result<Signature, Self::Error> {
        let hash_result = self.highway_hash.finalize256();
        let signature = self
            .secret_key
            .sign(bytemuck::bytes_of(&hash_result), Some(self.ed25519_noise));

        Ok(signature)
    }
}


