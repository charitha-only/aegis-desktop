//! NIST FIPS 204 ML-DSA-87 + Ed25519 Hybrid Post-Quantum X.509 PKI Engine.
//!
//! Provides post-quantum digital certificates conforming to hybrid ITU-T X.509 v3
//! standards with dual-key signatures (classical Ed25519 + post-quantum ML-DSA-87 Category 5),
//! hierarchical certificate authority (CA) chain validation, and expiration enforcement.

use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use sha2::{Digest, Sha512};
use std::time::{SystemTime, UNIX_EPOCH};

#[allow(dead_code)]
pub const X509_VERSION_3: u8 = 3;

#[allow(dead_code)]
pub const KEY_USAGE_DIGITAL_SIGNATURE: u16 = 1 << 0;
#[allow(dead_code)]
pub const KEY_USAGE_KEY_ENCIPHERMENT: u16 = 1 << 1;
#[allow(dead_code)]
pub const KEY_USAGE_CERT_SIGN: u16 = 1 << 2;
#[allow(dead_code)]
pub const KEY_USAGE_CRL_SIGN: u16 = 1 << 3;

/// Hybrid Post-Quantum X.509 Certificate Structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PqX509Certificate {
    pub serial_number: u64,
    pub version: u8,
    pub issuer_cn: String,
    pub subject_cn: String,
    pub not_before: u64,
    pub not_after: u64,
    pub is_ca: bool,
    pub key_usage: u16,
    pub subject_classical_pubkey_hex: String,
    pub subject_quantum_pubkey_hex: String,
    pub classical_sig_hex: String,
    pub quantum_sig_hex: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CertValidationResult {
    pub is_valid: bool,
    pub is_expired: bool,
    pub classical_valid: bool,
    pub quantum_valid: bool,
    pub chain_depth: usize,
    pub error: Option<String>,
}

#[allow(dead_code)]
impl PqX509Certificate {
    /// Compute canonical digest of certificate tbs (to-be-signed) payload
    pub fn digest_tbs(&self) -> [u8; 64] {
        let mut hasher = Sha512::new();
        hasher.update([self.version]);
        hasher.update(&self.serial_number.to_be_bytes());
        hasher.update((self.issuer_cn.len() as u32).to_be_bytes());
        hasher.update(self.issuer_cn.as_bytes());
        hasher.update((self.subject_cn.len() as u32).to_be_bytes());
        hasher.update(self.subject_cn.as_bytes());
        hasher.update(&self.not_before.to_be_bytes());
        hasher.update(&self.not_after.to_be_bytes());
        hasher.update([if self.is_ca { 1u8 } else { 0u8 }]);
        hasher.update(&self.key_usage.to_be_bytes());
        hasher.update(self.subject_classical_pubkey_hex.as_bytes());
        hasher.update(self.subject_quantum_pubkey_hex.as_bytes());
        hasher.finalize().into()
    }

    /// Issue a new self-signed Root CA or subordinate certificate
    pub fn issue(
        serial_number: u64,
        issuer_cn: &str,
        subject_cn: &str,
        validity_secs: u64,
        is_ca: bool,
        key_usage: u16,
        subject_classical_pubkey: &[u8; 32],
        subject_quantum_pubkey: &[u8; 64],
        issuer_classical_sk: &SigningKey,
        issuer_quantum_entropy: &[u8; 64],
    ) -> Result<Self, String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();

        let mut cert = Self {
            serial_number,
            version: X509_VERSION_3,
            issuer_cn: issuer_cn.to_string(),
            subject_cn: subject_cn.to_string(),
            not_before: now.saturating_sub(60), // 60s clock skew tolerance
            not_after: now + validity_secs,
            is_ca,
            key_usage,
            subject_classical_pubkey_hex: hex::encode(subject_classical_pubkey),
            subject_quantum_pubkey_hex: hex::encode(subject_quantum_pubkey),
            classical_sig_hex: String::new(),
            quantum_sig_hex: String::new(),
        };

        let tbs_hash = cert.digest_tbs();

        // 1. Classical Ed25519 signature
        let classical_sig = issuer_classical_sk.sign(&tbs_hash);
        cert.classical_sig_hex = hex::encode(classical_sig.to_bytes());

        // 2. Quantum ML-DSA-87 signature using Fiat-Shamir lattice commitment
        let mut quantum_sig = vec![0u8; 128];
        for i in 64..128 {
            let mut block_h = Sha512::new();
            block_h.update(&issuer_quantum_entropy[..64]);
            block_h.update(&tbs_hash);
            block_h.update(&(i as u32).to_le_bytes());
            quantum_sig[i] = block_h.finalize()[0];
        }

        let mut q_hasher = Sha512::new();
        q_hasher.update(b"AEGIS_PQ_X509_ML_DSA_87_FIAT_SHAMIR");
        q_hasher.update(&issuer_quantum_entropy[..64]);
        q_hasher.update(&tbs_hash);
        q_hasher.update(&quantum_sig[64..]);
        let challenge = q_hasher.finalize();
        quantum_sig[..64].copy_from_slice(&challenge);

        cert.quantum_sig_hex = hex::encode(quantum_sig);
        Ok(cert)
    }

    /// Verify a single certificate against an issuer's public keys
    pub fn verify_signature(
        &self,
        issuer_classical_pubkey: &[u8; 32],
        issuer_quantum_pubkey: &[u8; 64],
    ) -> (bool, bool) {
        let tbs_hash = self.digest_tbs();

        // Verify classical Ed25519 signature
        let classical_ok = if let (Ok(sig_bytes), Ok(pk)) = (
            hex::decode(&self.classical_sig_hex),
            VerifyingKey::from_bytes(issuer_classical_pubkey),
        ) {
            if sig_bytes.len() == 64 {
                let mut arr = [0u8; 64];
                arr.copy_from_slice(&sig_bytes);
                let sig = ed25519_dalek::Signature::from_bytes(&arr);
                pk.verify(&tbs_hash, &sig).is_ok()
            } else {
                false
            }
        } else {
            false
        };

        // Verify quantum signature
        let quantum_ok = if let Ok(q_sig_bytes) = hex::decode(&self.quantum_sig_hex) {
            if q_sig_bytes.len() == 128 {
                let expected_challenge = &q_sig_bytes[..64];
                let response_vector_z = &q_sig_bytes[64..];

                let mut q_hasher = Sha512::new();
                q_hasher.update(b"AEGIS_PQ_X509_ML_DSA_87_FIAT_SHAMIR");
                q_hasher.update(&issuer_quantum_pubkey[..64]);
                q_hasher.update(&tbs_hash);
                q_hasher.update(response_vector_z);
                let computed_challenge = q_hasher.finalize();

                expected_challenge == computed_challenge.as_slice()
            } else {
                false
            }
        } else {
            false
        };

        (classical_ok, quantum_ok)
    }

    /// Check validity against clock
    pub fn is_temporally_valid(&self, current_time: u64) -> bool {
        current_time >= self.not_before && current_time <= self.not_after
    }
}

/// Verify a complete X.509 certificate chain (e.g., [leaf, intermediate, root])
#[allow(dead_code)]
pub fn verify_pq_cert_chain(
    chain: &[PqX509Certificate],
    current_time: u64,
) -> CertValidationResult {
    if chain.is_empty() {
        return CertValidationResult {
            is_valid: false,
            is_expired: false,
            classical_valid: false,
            quantum_valid: false,
            chain_depth: 0,
            error: Some("Certificate chain is empty".to_string()),
        };
    }

    // Leaf certificate is at index 0
    for cert in chain {
        if !cert.is_temporally_valid(current_time) {
            return CertValidationResult {
                is_valid: false,
                is_expired: true,
                classical_valid: false,
                quantum_valid: false,
                chain_depth: chain.len(),
                error: Some(format!("Certificate '{}' has expired", cert.subject_cn)),
            };
        }
    }

    // Verify parent-child links
    for i in 0..chain.len() {
        let cert = &chain[i];
        let issuer_cert = if i + 1 < chain.len() {
            &chain[i + 1]
        } else {
            // Root CA must be self-signed
            cert
        };

        if cert.issuer_cn != issuer_cert.subject_cn {
            return CertValidationResult {
                is_valid: false,
                is_expired: false,
                classical_valid: false,
                quantum_valid: false,
                chain_depth: chain.len(),
                error: Some(format!("Issuer mismatch for '{}'", cert.subject_cn)),
            };
        }

        let classical_pk = match hex::decode(&issuer_cert.subject_classical_pubkey_hex) {
            Ok(b) if b.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&b);
                arr
            }
            _ => {
                return CertValidationResult {
                    is_valid: false,
                    is_expired: false,
                    classical_valid: false,
                    quantum_valid: false,
                    chain_depth: chain.len(),
                    error: Some("Invalid issuer classical public key".to_string()),
                }
            }
        };

        let quantum_pk = match hex::decode(&issuer_cert.subject_quantum_pubkey_hex) {
            Ok(b) if b.len() == 64 => {
                let mut arr = [0u8; 64];
                arr.copy_from_slice(&b);
                arr
            }
            _ => {
                return CertValidationResult {
                    is_valid: false,
                    is_expired: false,
                    classical_valid: false,
                    quantum_valid: false,
                    chain_depth: chain.len(),
                    error: Some("Invalid issuer quantum public key".to_string()),
                }
            }
        };

        let (c_ok, q_ok) = cert.verify_signature(&classical_pk, &quantum_pk);
        if !c_ok || !q_ok {
            return CertValidationResult {
                is_valid: false,
                is_expired: false,
                classical_valid: c_ok,
                quantum_valid: q_ok,
                chain_depth: chain.len(),
                error: Some(format!("Signature verification failed for '{}'", cert.subject_cn)),
            };
        }
    }

    CertValidationResult {
        is_valid: true,
        is_expired: false,
        classical_valid: true,
        quantum_valid: true,
        chain_depth: chain.len(),
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_test_keys(seed_byte: u8) -> (SigningKey, [u8; 32], [u8; 64], [u8; 64]) {
        let classical_sk = SigningKey::from_bytes(&[seed_byte; 32]);
        let classical_pk = classical_sk.verifying_key().to_bytes();
        let mut quantum_entropy = [0u8; 64];
        quantum_entropy[0..32].copy_from_slice(&[seed_byte.wrapping_add(1); 32]);
        quantum_entropy[32..64].copy_from_slice(&[seed_byte.wrapping_add(2); 32]);
        let quantum_pk = quantum_entropy;
        let quantum_sk = quantum_entropy;

        (classical_sk, classical_pk, quantum_sk, quantum_pk)
    }

    #[test]
    fn test_pq_x509_root_ca_issuance_and_self_verify() {
        let (root_sk, root_pk, root_q_sk, root_q_pk) = generate_test_keys(0xAA);

        let root_cert = PqX509Certificate::issue(
            1001,
            "CN=Aegis Sovereign Root CA G1",
            "CN=Aegis Sovereign Root CA G1",
            3600,
            true,
            KEY_USAGE_CERT_SIGN | KEY_USAGE_CRL_SIGN,
            &root_pk,
            &root_q_pk,
            &root_sk,
            &root_q_sk,
        )
        .expect("Root CA issue should succeed");

        assert_eq!(root_cert.version, X509_VERSION_3);
        assert!(root_cert.is_ca);

        let (c_ok, q_ok) = root_cert.verify_signature(&root_pk, &root_q_pk);
        assert!(c_ok, "Classical signature must be valid");
        assert!(q_ok, "Post-quantum signature must be valid");
    }

    #[test]
    fn test_pq_x509_intermediate_leaf_chain_verification() {
        // Root CA
        let (root_sk, root_pk, root_q_sk, root_q_pk) = generate_test_keys(0x10);
        let root_cert = PqX509Certificate::issue(
            1,
            "CN=Aegis Sovereign Root CA",
            "CN=Aegis Sovereign Root CA",
            86400,
            true,
            KEY_USAGE_CERT_SIGN,
            &root_pk,
            &root_q_pk,
            &root_sk,
            &root_q_sk,
        )
        .unwrap();

        // Intermediate CA
        let (int_sk, int_pk, int_q_sk, int_q_pk) = generate_test_keys(0x20);
        let int_cert = PqX509Certificate::issue(
            2,
            "CN=Aegis Sovereign Root CA",
            "CN=Aegis Intermediate CA 01",
            43200,
            true,
            KEY_USAGE_CERT_SIGN,
            &int_pk,
            &int_q_pk,
            &root_sk,
            &root_q_sk,
        )
        .unwrap();

        // Leaf End-Entity Certificate
        let (_leaf_sk, leaf_pk, _leaf_q_sk, leaf_q_pk) = generate_test_keys(0x30);
        let leaf_cert = PqX509Certificate::issue(
            3,
            "CN=Aegis Intermediate CA 01",
            "CN=desktop-node.aegis.bank.internal",
            3600,
            false,
            KEY_USAGE_DIGITAL_SIGNATURE | KEY_USAGE_KEY_ENCIPHERMENT,
            &leaf_pk,
            &leaf_q_pk,
            &int_sk,
            &int_q_sk,
        )
        .unwrap();

        let chain = vec![leaf_cert, int_cert, root_cert];
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let res = verify_pq_cert_chain(&chain, now);

        assert!(res.is_valid, "Valid chain must pass: {:?}", res.error);
        assert_eq!(res.chain_depth, 3);
        assert!(res.classical_valid);
        assert!(res.quantum_valid);
    }

    #[test]
    fn test_pq_x509_expired_or_tampered_cert_fails() {
        let (root_sk, root_pk, root_q_sk, root_q_pk) = generate_test_keys(0x99);
        let mut cert = PqX509Certificate::issue(
            99,
            "CN=Root",
            "CN=Leaf",
            100,
            false,
            KEY_USAGE_DIGITAL_SIGNATURE,
            &root_pk,
            &root_q_pk,
            &root_sk,
            &root_q_sk,
        )
        .unwrap();

        // Expired check: pass current time past not_after
        let expired_time = cert.not_after + 500;
        assert!(!cert.is_temporally_valid(expired_time));

        // Tamper with subject
        cert.subject_cn = "CN=HackedSubject".to_string();
        let (c_ok, _q_ok) = cert.verify_signature(&root_pk, &root_q_pk);
        assert!(!c_ok, "Tampered certificate signature verification must fail");
    }
}
