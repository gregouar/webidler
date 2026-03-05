use hmac::{Hmac, Mac};
use sha2::Sha256;

pub type HmacKey = [u8; 32];
pub type HmacSignature = [u8; 32];

pub fn load_hmac_key(setting_name: &str) -> HmacKey {
    let hex = std::env::var(setting_name).unwrap_or_else(|_| panic!("{setting_name} must be set"));

    let bytes = hex::decode(hex).unwrap_or_else(|_| panic!("{setting_name} must be valid hex"));

    if bytes.len() != 32 {
        panic!("{setting_name} must be 32 bytes (64 hex chars)");
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);

    key
}

pub fn compute_hmac(payload: &[u8], key: &HmacKey) -> [u8; 32] {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).unwrap();
    mac.update(payload);
    let result = mac.finalize();
    let bytes = result.into_bytes();

    let mut sig = [0u8; 32];
    sig.copy_from_slice(&bytes);
    sig
}

pub fn verify_hmac(payload: &[u8], signature: &[u8; 32], key: &HmacKey) -> bool {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).unwrap();
    mac.update(payload);
    mac.verify_slice(signature).is_ok()
}
