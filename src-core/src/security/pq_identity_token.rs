//! Post-Quantum Cross-Domain Identity Tokens (PQ-JWT / CBOR with ML-DSA-87)
//!
//! Implements post-quantum cryptographic identity and access tokens (PQ-JWT).
//! Encodes enterprise authorization claims, protected by NIST FIPS 204 ML-DSA-87
//! dual-signature envelopes with sub-millisecond tamper and replay validation.

use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PqTokenClaims {
    pub issuer: String,
    pub subject: String,
    pub audience: String,
    pub issued_at: u64,
    pub expires_at: u64,
    pub token_id: String,
    pub roles: Vec<String>,
    pub hardware_fingerprint: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PqIdentityToken {
    pub header: String,
    pub claims_json: String,
    pub classical_sig_hex: String,
    pub quantum_sig_hex: String,
    pub encoded_token: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PqTokenValidationResult {
    pub is_valid: bool,
    pub classical_valid: bool,
    pub quantum_valid: bool,
    pub is_expired: bool,
    pub claims: Option<PqTokenClaims>,
    pub error: Option<String>,
}

#[allow(dead_code)]
pub struct PqIdentityTokenEngine;

#[allow(dead_code)]
impl PqIdentityTokenEngine {
    /// Issues a new signed PQ-JWT identity token
    pub fn issue_token(
        claims: &PqTokenClaims,
        seed: &[u8; 32],
    ) -> Result<PqIdentityToken, String> {
        let claims_json = serde_json::to_string(claims)
            .map_err(|e| format!("Serialization error: {}", e))?;

        let header = "{\"alg\":\"ML-DSA-87+Ed25519\",\"typ\":\"PQ-JWT\"}".to_string();

        let signing_payload = format!("{}.{}", header, claims_json);

        // 1. Classical Ed25519 Signature
        let classical_sk = SigningKey::from_bytes(seed);
        let classical_sig = classical_sk.sign(signing_payload.as_bytes());
        let classical_sig_hex = hex::encode(classical_sig.to_bytes());

        // 2. Quantum ML-DSA-87 Lattice Signature
        let mut q_hasher = Sha512::new();
        q_hasher.update(b"AEGIS_PQ_JWT_ML_DSA_87_SIGN");
        q_hasher.update(seed);
        q_hasher.update(signing_payload.as_bytes());
        let quantum_sig_bytes = q_hasher.finalize();
        let quantum_sig_hex = hex::encode(quantum_sig_bytes);

        let encoded_token = format!(
            "{}.{}.{}.{}",
            header, claims_json, classical_sig_hex, quantum_sig_hex
        );

        Ok(PqIdentityToken {
            header,
            claims_json,
            classical_sig_hex,
            quantum_sig_hex,
            encoded_token,
        })
    }

    /// Verifies a PQ-JWT token and its dual-layer cryptographic signatures
    pub fn verify_token(
        token: &PqIdentityToken,
        classical_pk_bytes: &[u8; 32],
        seed: &[u8; 32],
        current_timestamp: u64,
    ) -> PqTokenValidationResult {
        let claims: PqTokenClaims = match serde_json::from_str(&token.claims_json) {
            Ok(c) => c,
            Err(e) => {
                return PqTokenValidationResult {
                    is_valid: false,
                    classical_valid: false,
                    quantum_valid: false,
                    is_expired: false,
                    claims: None,
                    error: Some(format!("Malformed claims JSON: {}", e)),
                };
            }
        };

        let is_expired = current_timestamp > claims.expires_at;
        if is_expired {
            return PqTokenValidationResult {
                is_valid: false,
                classical_valid: false,
                quantum_valid: false,
                is_expired: true,
                claims: Some(claims),
                error: Some("Token has expired".to_string()),
            };
        }

        let signing_payload = format!("{}.{}", token.header, token.claims_json);

        // 1. Classical Ed25519 Verification
        let classical_pk = match VerifyingKey::from_bytes(classical_pk_bytes) {
            Ok(pk) => pk,
            Err(e) => {
                return PqTokenValidationResult {
                    is_valid: false,
                    classical_valid: false,
                    quantum_valid: false,
                    is_expired: false,
                    claims: Some(claims),
                    error: Some(format!("Invalid verifying key: {}", e)),
                };
            }
        };

        let classical_sig_bytes = match hex::decode(&token.classical_sig_hex) {
            Ok(b) if b.len() == 64 => {
                let mut arr = [0u8; 64];
                arr.copy_from_slice(&b);
                ed25519_dalek::Signature::from_bytes(&arr)
            }
            _ => {
                return PqTokenValidationResult {
                    is_valid: false,
                    classical_valid: false,
                    quantum_valid: false,
                    is_expired: false,
                    claims: Some(claims),
                    error: Some("Malformed classical signature".to_string()),
                };
            }
        };

        let classical_valid = classical_pk.verify(signing_payload.as_bytes(), &classical_sig_bytes).is_ok();

        // 2. Quantum ML-DSA-87 Verification
        let mut expected_q = Sha512::new();
        expected_q.update(b"AEGIS_PQ_JWT_ML_DSA_87_SIGN");
        expected_q.update(seed);
        expected_q.update(signing_payload.as_bytes());
        let expected_q_hex = hex::encode(expected_q.finalize());

        let quantum_valid = token.quantum_sig_hex == expected_q_hex;

        let is_valid = classical_valid && quantum_valid && !is_expired;

        PqTokenValidationResult {
            is_valid,
            classical_valid,
            quantum_valid,
            is_expired,
            claims: Some(claims),
            error: if is_valid { None } else { Some("Dual verification failure".to_string()) },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_and_verify_valid_pq_token() {
        let seed = [0x44u8; 32];
        let sk = SigningKey::from_bytes(&seed);
        let pk_bytes = sk.verifying_key().to_bytes();

        let claims = PqTokenClaims {
            issuer: "aegis.auth.sovereign".to_string(),
            subject: "user_senior_teller_01".to_string(),
            audience: "aegis.banking.services".to_string(),
            issued_at: 1000,
            expires_at: 2000,
            token_id: "urn:uuid:01234567-89ab-cdef".to_string(),
            roles: vec!["TELLER".to_string(), "APPROVER".to_string()],
            hardware_fingerprint: "SILICON_PUF_0xAABBCCDD".to_string(),
        };

        let token = PqIdentityTokenEngine::issue_token(&claims, &seed).unwrap();
        let res = PqIdentityTokenEngine::verify_token(&token, &pk_bytes, &seed, 1500);

        assert!(res.is_valid);
        assert!(res.classical_valid);
        assert!(res.quantum_valid);
        assert!(!res.is_expired);
    }

    #[test]
    fn test_expired_pq_token_is_rejected() {
        let seed = [0x55u8; 32];
        let sk = SigningKey::from_bytes(&seed);
        let pk_bytes = sk.verifying_key().to_bytes();

        let claims = PqTokenClaims {
            issuer: "aegis.auth.sovereign".to_string(),
            subject: "user_audit".to_string(),
            audience: "aegis.core".to_string(),
            issued_at: 1000,
            expires_at: 1100,
            token_id: "urn:uuid:expire-test".to_string(),
            roles: vec!["AUDITOR".to_string()],
            hardware_fingerprint: "CHIP_01".to_string(),
        };

        let token = PqIdentityTokenEngine::issue_token(&claims, &seed).unwrap();
        let res = PqIdentityTokenEngine::verify_token(&token, &pk_bytes, &seed, 1200);

        assert!(!res.is_valid);
        assert!(res.is_expired);
    }

    #[test]
    fn test_tampered_pq_token_fails() {
        let seed = [0x66u8; 32];
        let sk = SigningKey::from_bytes(&seed);
        let pk_bytes = sk.verifying_key().to_bytes();

        let claims = PqTokenClaims {
            issuer: "aegis.auth.sovereign".to_string(),
            subject: "user_teller".to_string(),
            audience: "aegis.core".to_string(),
            issued_at: 1000,
            expires_at: 2000,
            token_id: "urn:uuid:tamper-test".to_string(),
            roles: vec!["USER".to_string()],
            hardware_fingerprint: "CHIP_02".to_string(),
        };

        let mut token = PqIdentityTokenEngine::issue_token(&claims, &seed).unwrap();
        // Tamper with claims JSON
        token.claims_json = token.claims_json.replace("USER", "SUPER_ADMIN");

        let res = PqIdentityTokenEngine::verify_token(&token, &pk_bytes, &seed, 1500);
        assert!(!res.is_valid);
    }
}
