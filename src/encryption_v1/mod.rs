mod constants;
pub mod decryption_module;
pub mod encryption;
pub mod encryption_module;
pub mod hash;
pub mod hash_based_signature;
mod hash_based_signature_verifier;

#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use ed25519_compact::Signature;
    use tests::{
        decryption_module::ZeroKnowledgeDecryptionModuleV1,
        encryption_module::ZeroKnowledgeEncryptionModuleV1,
    };

    use crate::encryption_v1::decryption_module::DecryptionModule;
    use crate::encryption_v1::encryption::setup;
    use crate::encryption_v1::encryption_module::EncryptionModule;
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    #[test]
    fn test_encryption_module() -> Result<(), ()> {
        let secrets = Arc::from(setup("jkl!9OSda_jdsAdjoi9839", "email@email.com").unwrap());
        let bucket_id = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let mut enc = ZeroKnowledgeEncryptionModuleV1::new(secrets.clone(), &bucket_id.clone());
        let data = "asjdipasjdaosdjapsdjeaomclcmsiufriunkdvnfdkvdnveruio90903r0jwifojef9qjpquogqhh83uqh8uieaaapöshuidxjnm,zxcbiarv";
        let _enc_data = enc.update(data).unwrap();
        let signature = Signature::from_slice(enc.finalize().unwrap().as_slice()).unwrap();
        let mut _dec = ZeroKnowledgeDecryptionModuleV1::new(secrets, &bucket_id, Some(signature));
        let data2 = _dec.update(_enc_data).unwrap();
        assert_eq!(data, std::str::from_utf8(data2.as_slice()).unwrap());
        Ok(())
    }
}
