use zeroize::Zeroizing;
// Versioned authenticated encryption. Legacy deterministic envelopes are rejected.
use chacha20poly1305::{
    aead::{Aead, Payload},
    KeyInit, XChaCha20Poly1305, XNonce,
};
pub struct VaultEngine;
impl VaultEngine {
    pub fn encrypt(text: &str, password: &str) -> Result<String, String> {
        if text.len() > 512 * 1024 || password.len() > 1024 {
            return Err("Vault input exceeds size limit".into());
        }
        if password.chars().count() < 12 {
            return Err("Passphrase must be at least 12 characters".into());
        }
        let mut salt = [0u8; 16];
        let mut nonce = [0u8; 24];
        getrandom::getrandom(&mut salt).map_err(|e| e.to_string())?;
        getrandom::getrandom(&mut nonce).map_err(|e| e.to_string())?;
        let mut key = Zeroizing::new([0u8; 32]);
        argon2::Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(19456, 2, 1, Some(32)).map_err(|e| e.to_string())?,
        )
        .hash_password_into(password.as_bytes(), &salt, &mut *key)
        .map_err(|e| e.to_string())?;
        let cipher = XChaCha20Poly1305::new((&*key).into());
        let ciphertext = cipher
            .encrypt(
                XNonce::from_slice(&nonce),
                Payload {
                    msg: text.as_bytes(),
                    aad: b"AEGIS_VAULT_V2",
                },
            )
            .map_err(|_| "Encryption failed")?;
        Ok(format!(
            "AEGIS_VAULT_V2:{}:{}:{}",
            hex::encode(salt),
            hex::encode(nonce),
            hex::encode(ciphertext)
        ))
    }
    pub fn decrypt(envelope: &str, password: &str) -> Result<String, String> {
        if envelope.len() > 2 * 1024 * 1024 + 256 || password.len() > 1024 {
            return Err("Vault envelope exceeds size limit".into());
        }
        let p: Vec<_> = envelope.split(':').collect();
        if p.len() != 4 || p[0] != "AEGIS_VAULT_V2" {
            return Err("Unsupported or legacy vault format; original files are preserved".into());
        }
        let salt = hex::decode(p[1]).map_err(|_| "Invalid salt")?;
        let nonce = hex::decode(p[2]).map_err(|_| "Invalid nonce")?;
        let data = hex::decode(p[3]).map_err(|_| "Invalid ciphertext")?;
        if salt.len() != 16 || nonce.len() != 24 || data.len() > 1024 * 1024 {
            return Err("Invalid envelope lengths".into());
        }
        let mut key = Zeroizing::new([0u8; 32]);
        argon2::Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(19456, 2, 1, Some(32)).map_err(|e| e.to_string())?,
        )
        .hash_password_into(password.as_bytes(), &salt, &mut *key)
        .map_err(|e| e.to_string())?;
        let plain = XChaCha20Poly1305::new((&*key).into())
            .decrypt(
                XNonce::from_slice(&nonce),
                Payload {
                    msg: &data,
                    aad: b"AEGIS_VAULT_V2",
                },
            )
            .map_err(|_| "Decryption failed: wrong passphrase or tampered data")?;
        String::from_utf8(plain).map_err(|_| "Invalid plaintext encoding".into())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authenticated_randomized_roundtrip() {
        let a = VaultEngine::encrypt("secret", "passphrase123").unwrap();
        let b = VaultEngine::encrypt("secret", "passphrase123").unwrap();
        assert_ne!(a, b);
        assert_eq!(VaultEngine::decrypt(&a, "passphrase123").unwrap(), "secret");
        assert!(VaultEngine::decrypt(&a, "wrong").is_err());
        let mut t = a.clone();
        t.push('0');
        assert!(VaultEngine::decrypt(&t, "passphrase123").is_err());
        assert!(VaultEngine::decrypt("AEGIS_VAULT_V1:old:data", "passphrase123").is_err());
    }
}
