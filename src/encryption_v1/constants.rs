use once_cell::sync::Lazy;

// This key uses
pub const SERVER_PUBLIC_X25519_KEY: Lazy<x25519_dalek::PublicKey> =
    Lazy::new(|| x25519_dalek::PublicKey::try_from(*b"12345678901234567890123456789012").unwrap());
pub const ENCRYPTION_VERSION: &str = "1.0.0";

// Encrypt this struct with the gcp key and sign it with the clients public ed25519 key.

pub const SHARE_LINK_SIGNATURE_NOISE: [u8; 16] = [
    32, 149, 41, 117, 50, 104, 21, 7, 126, 159, 154, 35, 36, 238, 236, 105,
];

pub const AES_GCM_NONCE: [u8; 12] = [123, 49, 19, 39, 117, 143, 123, 124, 12, 44, 12, 13];

pub const HIGHWAY_HASH_KEY: [u64; 4] = [
    0x0706050403020100,
    0x0F0E0D0C0B0A0908,
    0x1716151413121110,
    0x1F1E1D1C1B1A1918,
];

pub const V1_ENCRYPTION_PASSWORD_SALT: &str = "";

pub const V1_X25519_SIGNATURE_HASH_SALT:&str = "";