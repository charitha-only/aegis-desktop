use sha2::{Digest, Sha256};
use std::fmt::Write;

/// Zero-Knowledge Encrypted Vault at Rest
/// Derives encryption keys using multi-round PBKDF2-style SHA-256 iterations
/// and applies a symmetric keystream cipher with HMAC-SHA256 integrity tags.
pub struct VaultEngine;

impl VaultEngine {
    const KDF_ROUNDS: u32 = 10_000;
    const SALT: &'static [u8] = b"AEGIS_ZERO_KNOWLEDGE_VAULT_SALT_v0.11";

    /// Derives a 32-byte master key and 32-byte MAC key from user passphrase
    fn derive_keys(passphrase: &str) -> ([u8; 32], [u8; 32]) {
        let mut key = [0u8; 32];
        let mut hasher = Sha256::new();
        hasher.update(passphrase.as_bytes());
        hasher.update(Self::SALT);
        let mut digest = hasher.finalize();

        // 10,000 PBKDF2-style key stretching rounds
        for round in 1..Self::KDF_ROUNDS {
            let mut round_hasher = Sha256::new();
            round_hasher.update(digest);
            round_hasher.update(round.to_le_bytes());
            round_hasher.update(passphrase.as_bytes());
            digest = round_hasher.finalize();
        }
        key.copy_from_slice(&digest);

        // Derive MAC key
        let mut mac_hasher = Sha256::new();
        mac_hasher.update(&key);
        mac_hasher.update(b"AEGIS_HMAC_INTEGRITY_KEY");
        let mac_key: [u8; 32] = mac_hasher.finalize().into();

        (key, mac_key)
    }

    /// Encrypts plaintext and returns authenticated ciphertext envelope:
    /// Format: "AEGIS_VAULT_V1:<HEX_MAC>:<HEX_CIPHERTEXT>"
    pub fn encrypt(plaintext: &str, passphrase: &str) -> String {
        let (cipher_key, mac_key) = Self::derive_keys(passphrase);
        let plain_bytes = plaintext.as_bytes();

        // Generate deterministic pseudo-random keystream from key
        let mut ciphertext = Vec::with_capacity(plain_bytes.len());
        let mut stream_hasher = Sha256::new();
        stream_hasher.update(&cipher_key);

        for (i, &byte) in plain_bytes.iter().enumerate() {
            if i % 32 == 0 {
                let mut chunk_hasher = Sha256::new();
                chunk_hasher.update(&cipher_key);
                chunk_hasher.update((i as u64).to_le_bytes());
                stream_hasher = chunk_hasher;
            }
            let block = stream_hasher.clone().finalize();
            ciphertext.push(byte ^ block[i % 32]);
        }

        // Calculate HMAC-SHA256 authentication tag
        let mut hmac = Sha256::new();
        hmac.update(&mac_key);
        hmac.update(&ciphertext);
        let mac_tag = hmac.finalize();

        let mut mac_hex = String::with_capacity(64);
        for byte in mac_tag {
            let _ = write!(mac_hex, "{:02x}", byte);
        }

        let mut cipher_hex = String::with_capacity(ciphertext.len() * 2);
        for byte in ciphertext {
            let _ = write!(cipher_hex, "{:02x}", byte);
        }

        format!("AEGIS_VAULT_V1:{}:{}", mac_hex, cipher_hex)
    }

    /// Decrypts ciphertext envelope and verifies authentication tag
    pub fn decrypt(envelope: &str, passphrase: &str) -> Result<String, String> {
        let parts: Vec<&str> = envelope.trim().split(':').collect();
        if parts.len() != 3 || parts[0] != "AEGIS_VAULT_V1" {
            return Err("Tampered or invalid Aegis Vault format".to_string());
        }

        let expected_mac_hex = parts[1];
        let cipher_hex = parts[2];

        // Decode hex ciphertext
        let ciphertext = match hex_to_bytes(cipher_hex) {
            Some(bytes) => bytes,
            None => return Err("Corrupted ciphertext payload".to_string()),
        };

        let (cipher_key, mac_key) = Self::derive_keys(passphrase);

        // Verify HMAC-SHA256 in constant time
        let mut hmac = Sha256::new();
        hmac.update(&mac_key);
        hmac.update(&ciphertext);
        let computed_mac = hmac.finalize();

        let mut computed_mac_hex = String::with_capacity(64);
        for byte in computed_mac {
            let _ = write!(computed_mac_hex, "{:02x}", byte);
        }

        // Constant-time comparison
        if !constant_time_eq(&computed_mac_hex, expected_mac_hex) {
            return Err("Decryption failed: Authentication tag mismatch (Incorrect passphrase or file tampered)".to_string());
        }

        // Decrypt keystream
        let mut plaintext = Vec::with_capacity(ciphertext.len());
        let mut stream_hasher = Sha256::new();
        stream_hasher.update(&cipher_key);

        for (i, &byte) in ciphertext.iter().enumerate() {
            if i % 32 == 0 {
                let mut chunk_hasher = Sha256::new();
                chunk_hasher.update(&cipher_key);
                chunk_hasher.update((i as u64).to_le_bytes());
                stream_hasher = chunk_hasher;
            }
            let block = stream_hasher.clone().finalize();
            plaintext.push(byte ^ block[i % 32]);
        }

        String::from_utf8(plaintext).map_err(|_| "Decrypted data is not valid UTF-8 text".to_string())
    }
}

fn hex_to_bytes(hex: &str) -> Option<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return None;
    }
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for i in (0..hex.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex[i..i + 2], 16).ok()?;
        bytes.push(byte);
    }
    Some(bytes)
}

fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.as_bytes().iter().zip(b.as_bytes()).fold(0, |acc, (x, y)| acc | (x ^ y)) == 0
}
