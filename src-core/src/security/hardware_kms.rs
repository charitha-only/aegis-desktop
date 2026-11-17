//! Hardware-Rooted Key Management Service (Hardware KMS) & Envelope Encryption.
//!
//! Enforces hardware envelope encryption where ephemeral Data Encryption Keys (DEKs)
//! are wrapped by silicon Key Encryption Keys (KEKs) bound to Physical Unclonable
//! Functions (PUF) and TPM 2.0 PCR measured boot policies.

use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KmsEnvelope {
    pub key_id: String,
    pub encrypted_dek_hex: String,
    pub dek_nonce_hex: String,
    pub ciphertext_hex: String,
    pub payload_nonce_hex: String,
    pub pcr_policy_digest: String,
    pub created_timestamp_secs: u64,
}

#[allow(dead_code)]
pub struct HardwareKmsEngine {
    kek: [u8; 32],
    pub required_pcr_policy: String,
    pub key_id: String,
}

impl Drop for HardwareKmsEngine {
    fn drop(&mut self) {
        self.kek.zeroize();
    }
}

#[allow(dead_code)]
impl HardwareKmsEngine {
    /// Initialize a new Hardware KMS instance bound to a silicon PUF seed and expected PCR policy.
    pub fn new(
        key_id: &str,
        silicon_puf_seed: &[u8; 32],
        required_pcr_policy: &str,
    ) -> Self {
        let mut kek_hasher = Sha256::new();
        kek_hasher.update(b"AEGIS_HARDWARE_KMS_KEK_ROOT_V2_8_0");
        kek_hasher.update(silicon_puf_seed);
        kek_hasher.update(key_id.as_bytes());
        let kek: [u8; 32] = kek_hasher.finalize().into();

        Self {
            kek,
            required_pcr_policy: required_pcr_policy.to_string(),
            key_id: key_id.to_string(),
        }
    }

    /// Rotate the silicon KEK root.
    pub fn rotate_kek(&mut self, new_silicon_puf_seed: &[u8; 32]) {
        let mut kek_hasher = Sha256::new();
        kek_hasher.update(b"AEGIS_HARDWARE_KMS_KEK_ROOT_V2_8_0");
        kek_hasher.update(new_silicon_puf_seed);
        kek_hasher.update(self.key_id.as_bytes());
        self.kek.zeroize();
        self.kek = kek_hasher.finalize().into();
    }

    /// Perform hardware envelope encryption:
    /// 1. Verifies that the host PCR digest matches policy.
    /// 2. Generates a fresh ephemeral DEK.
    /// 3. Encrypts payload with DEK.
    /// 4. Wraps DEK with silicon KEK (binding PCR policy as AAD).
    pub fn envelope_encrypt(
        &self,
        plaintext: &[u8],
        current_pcr_digest: &str,
        entropy_seed: &[u8; 32],
        timestamp: u64,
    ) -> Result<KmsEnvelope, String> {
        if current_pcr_digest != self.required_pcr_policy {
            return Err(format!(
                "Hardware PCR Policy Violation: Host PCR digest '{}' does not match enrolled policy '{}'",
                current_pcr_digest, self.required_pcr_policy
            ));
        }

        // 1. Derive fresh ephemeral DEK
        let mut dek_hasher = Sha256::new();
        dek_hasher.update(b"AEGIS_EPHEMERAL_DEK");
        dek_hasher.update(entropy_seed);
        dek_hasher.update(&timestamp.to_be_bytes());
        let mut dek: [u8; 32] = dek_hasher.finalize().into();

        // 2. Encrypt plaintext using ephemeral DEK
        let dek_cipher = ChaCha20Poly1305::new_from_slice(&dek)
            .map_err(|e| format!("DEK cipher error: {}", e))?;

        let mut payload_nonce_bytes = [0u8; 12];
        payload_nonce_bytes[0..4].copy_from_slice(&timestamp.to_be_bytes()[4..8]);
        payload_nonce_bytes[4..8].copy_from_slice(&dek[0..4]);
        payload_nonce_bytes[8..12].copy_from_slice(b"PAYL");
        let payload_nonce = Nonce::from_slice(&payload_nonce_bytes);

        let payload_ciphertext = dek_cipher
            .encrypt(payload_nonce, plaintext)
            .map_err(|e| format!("Payload encryption error: {}", e))?;

        // 3. Wrap DEK using silicon KEK with PCR policy digest as AAD
        let kek_cipher = ChaCha20Poly1305::new_from_slice(&self.kek)
            .map_err(|e| format!("KEK cipher error: {}", e))?;

        let mut dek_nonce_bytes = [0u8; 12];
        dek_nonce_bytes[0..4].copy_from_slice(&timestamp.to_be_bytes()[0..4]);
        dek_nonce_bytes[4..8].copy_from_slice(&self.kek[0..4]);
        dek_nonce_bytes[8..12].copy_from_slice(b"WRAP");
        let dek_nonce = Nonce::from_slice(&dek_nonce_bytes);

        let wrap_payload = chacha20poly1305::aead::Payload {
            msg: &dek,
            aad: self.required_pcr_policy.as_bytes(),
        };

        let wrapped_dek = kek_cipher
            .encrypt(dek_nonce, wrap_payload)
            .map_err(|e| format!("DEK wrapping error: {}", e))?;

        // Zeroize plaintext DEK
        dek.zeroize();

        Ok(KmsEnvelope {
            key_id: self.key_id.clone(),
            encrypted_dek_hex: hex::encode(wrapped_dek),
            dek_nonce_hex: hex::encode(dek_nonce_bytes),
            ciphertext_hex: hex::encode(payload_ciphertext),
            payload_nonce_hex: hex::encode(payload_nonce_bytes),
            pcr_policy_digest: self.required_pcr_policy.clone(),
            created_timestamp_secs: timestamp,
        })
    }

    /// Perform hardware envelope decryption:
    /// 1. Verifies that the host PCR digest matches envelope and hardware policy.
    /// 2. Unwraps ephemeral DEK with silicon KEK.
    /// 3. Decrypts payload with DEK.
    /// 4. Zeroizes DEK immediately.
    pub fn envelope_decrypt(
        &self,
        envelope: &KmsEnvelope,
        current_pcr_digest: &str,
    ) -> Result<Vec<u8>, String> {
        if envelope.key_id != self.key_id {
            return Err("KMS Key ID mismatch".to_string());
        }
        if current_pcr_digest != self.required_pcr_policy || envelope.pcr_policy_digest != self.required_pcr_policy {
            return Err(format!(
                "Hardware PCR Policy Violation: Current PCR digest '{}' does not match policy '{}'",
                current_pcr_digest, self.required_pcr_policy
            ));
        }

        // 1. Unwrap DEK with KEK
        let kek_cipher = ChaCha20Poly1305::new_from_slice(&self.kek)
            .map_err(|e| format!("KEK cipher error: {}", e))?;

        let dek_nonce_bytes = hex::decode(&envelope.dek_nonce_hex)
            .map_err(|e| format!("Invalid dek nonce hex: {}", e))?;
        if dek_nonce_bytes.len() != 12 {
            return Err("Invalid DEK nonce length".to_string());
        }
        let dek_nonce = Nonce::from_slice(&dek_nonce_bytes);

        let wrapped_dek_bytes = hex::decode(&envelope.encrypted_dek_hex)
            .map_err(|e| format!("Invalid wrapped dek hex: {}", e))?;

        let unwrap_payload = chacha20poly1305::aead::Payload {
            msg: &wrapped_dek_bytes,
            aad: self.required_pcr_policy.as_bytes(),
        };

        let mut unwrapped_dek = kek_cipher
            .decrypt(dek_nonce, unwrap_payload)
            .map_err(|_| "DEK unwrap failed: KEK mismatch or PCR policy AAD tampered".to_string())?;

        if unwrapped_dek.len() != 32 {
            unwrapped_dek.zeroize();
            return Err("Unwrapped DEK length invalid".to_string());
        }

        // 2. Decrypt ciphertext using DEK
        let dek_cipher = ChaCha20Poly1305::new_from_slice(&unwrapped_dek)
            .map_err(|e| format!("DEK cipher init error: {}", e))?;

        let payload_nonce_bytes = hex::decode(&envelope.payload_nonce_hex)
            .map_err(|e| format!("Invalid payload nonce hex: {}", e))?;
        if payload_nonce_bytes.len() != 12 {
            unwrapped_dek.zeroize();
            return Err("Invalid payload nonce length".to_string());
        }
        let payload_nonce = Nonce::from_slice(&payload_nonce_bytes);

        let ciphertext_bytes = hex::decode(&envelope.ciphertext_hex)
            .map_err(|e| format!("Invalid ciphertext hex: {}", e))?;

        let decrypted_plaintext = dek_cipher
            .decrypt(payload_nonce, ciphertext_bytes.as_slice())
            .map_err(|_| "Payload decryption failed: ciphertext corrupted or tampered".to_string());

        // Zeroize unwrapped DEK
        unwrapped_dek.zeroize();

        decrypted_plaintext
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_kms_envelope_encrypt_decrypt_roundtrip() {
        let puf_seed = [0x77u8; 32];
        let pcr_policy = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let kms = HardwareKmsEngine::new("root-kek-vault-01", &puf_seed, pcr_policy);

        let secret_data = b"TOP_SECRET_HSM_RECOVERY_PHRASE_CORP_2026";
        let entropy = [0x55u8; 32];
        let timestamp = 1794500000;

        let envelope = kms
            .envelope_encrypt(secret_data, pcr_policy, &entropy, timestamp)
            .expect("Envelope encryption must succeed under valid PCR policy");

        assert_eq!(envelope.key_id, "root-kek-vault-01");
        assert_eq!(envelope.pcr_policy_digest, pcr_policy);

        let decrypted = kms
            .envelope_decrypt(&envelope, pcr_policy)
            .expect("Envelope decryption must succeed under matching PCR policy");

        assert_eq!(decrypted, secret_data);
    }

    #[test]
    fn test_hardware_kms_pcr_policy_mismatch_fails() {
        let puf_seed = [0x88u8; 32];
        let pcr_policy_enrolled = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let kms = HardwareKmsEngine::new("root-kek-vault-02", &puf_seed, pcr_policy_enrolled);

        let secret_data = b"CONFIDENTIAL_BANK_VAULT_PASSCODE";
        let entropy = [0x33u8; 32];
        let timestamp = 1794500100;

        // Attempt encryption with tampered PCR measurement (compromised bootloader)
        let compromised_pcr = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let enc_res = kms.envelope_encrypt(secret_data, compromised_pcr, &entropy, timestamp);
        assert!(enc_res.is_err(), "Encryption must fail if PCR policy does not match enrolled state");

        // Encrypt with valid policy
        let envelope = kms
            .envelope_encrypt(secret_data, pcr_policy_enrolled, &entropy, timestamp)
            .unwrap();

        // Attempt decryption under altered PCR state
        let dec_res = kms.envelope_decrypt(&envelope, compromised_pcr);
        assert!(dec_res.is_err(), "Decryption must fail closed if host PCR measurement has changed");
        assert!(dec_res.unwrap_err().contains("Hardware PCR Policy Violation"));
    }
}
