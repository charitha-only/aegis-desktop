use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedEnvelope {
    pub cipher_mode: String,
    pub iv_hex: String,
    pub ciphertext_hex: String,
    pub auth_tag_hex: String,
}

pub struct EncryptedStream;
impl EncryptedStream {
    pub fn encrypt_payload(_text: &str, _nonce: &str) -> Result<EncryptedEnvelope, String> {
        Err("UNAVAILABLE: This demo is not an encrypted IPC transport".into())
    }
    pub fn decrypt_payload(_env: &EncryptedEnvelope, _nonce: &str) -> Result<String, String> {
        Err("UNAVAILABLE: encrypted IPC transport is not configured".into())
    }
}
