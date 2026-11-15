//! Chaumian / Schnorr Blind Signatures on Curve25519.
//!
//! Provides cryptographic blinding allowing clients to obtain valid digital signatures
//! from an authority without revealing the token payload, serial number, or payee details.
//! Used for untraceable sovereign e-cash tokens, private access credentials, and anonymous votes.

use curve25519_dalek::constants::ED25519_BASEPOINT_POINT;
use curve25519_dalek::edwards::{CompressedEdwardsY, EdwardsPoint};
use curve25519_dalek::scalar::Scalar;
use sha2::{Digest, Sha512};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[allow(dead_code)]
pub struct BlindedTokenRequest {
    pub blinded_challenge_hex: String,
    pub signer_public_key_hex: String,
    pub signer_nonce_commitment_hex: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UnblindedToken {
    pub message: String,
    pub challenge_c_prime_hex: String,
    pub response_s_prime_hex: String,
    pub signer_public_key_hex: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlindSignatureVerificationResult {
    pub is_valid: bool,
    pub message: String,
    pub signer_public_key_hex: String,
    pub protocol: String,
    pub error: Option<String>,
}

#[allow(dead_code)]
pub struct BlindSignatureAuthority {
    secret_key: Scalar,
    pub public_key: EdwardsPoint,
}

#[allow(dead_code)]
impl BlindSignatureAuthority {
    pub fn new(seed: &[u8; 32]) -> Self {
        let secret_key = Scalar::from_bytes_mod_order(*seed);
        let public_key = ED25519_BASEPOINT_POINT * secret_key;
        Self {
            secret_key,
            public_key,
        }
    }

    /// Signer step 1: Generate an ephemeral nonce commitment R = k * G
    pub fn generate_nonce_commitment(&self, ephemeral_k_seed: &[u8; 32]) -> (Scalar, CompressedEdwardsY) {
        let k = Scalar::from_bytes_mod_order(*ephemeral_k_seed);
        let r_point = ED25519_BASEPOINT_POINT * k;
        (k, r_point.compress())
    }

    /// Signer step 2: Sign the client's blinded challenge c: s = k - c * x (mod l)
    pub fn sign_blinded(&self, k: &Scalar, blinded_c: &Scalar) -> Scalar {
        k - (blinded_c * self.secret_key)
    }
}

#[allow(dead_code)]
pub struct BlindSignatureClient;

#[allow(dead_code)]
impl BlindSignatureClient {
    /// Client step 1: Blind the message with secret factors alpha and beta
    /// R' = R + alpha * G + beta * Y
    /// c' = H(Y || R' || message)
    /// c = c' - beta (mod l)
    pub fn blind_message(
        signer_pk: &EdwardsPoint,
        r_point: &EdwardsPoint,
        message: &str,
        alpha_seed: &[u8; 32],
        beta_seed: &[u8; 32],
    ) -> (Scalar, Scalar, Scalar, CompressedEdwardsY) {
        let alpha = Scalar::from_bytes_mod_order(*alpha_seed);
        let beta = Scalar::from_bytes_mod_order(*beta_seed);

        let r_prime = r_point + (ED25519_BASEPOINT_POINT * alpha) + (signer_pk * beta);
        let r_prime_compressed = r_prime.compress();

        let mut hasher = Sha512::new();
        hasher.update(b"AEGIS_SCHNORR_BLIND_SIGNATURE_CHALLENGE_v2.5");
        hasher.update(signer_pk.compress().as_bytes());
        hasher.update(r_prime_compressed.as_bytes());
        hasher.update(message.as_bytes());
        let c_prime = Scalar::from_hash(hasher);

        let blinded_c = c_prime - beta;

        (blinded_c, c_prime, alpha, r_prime_compressed)
    }

    /// Client step 2: Unblind the signer's response: s' = s + alpha (mod l)
    pub fn unblind_response(s: &Scalar, alpha: &Scalar) -> Scalar {
        s + alpha
    }

    /// Verification: Assert R'' = s' * G + c' * Y produces c' == H(Y || R'' || message)
    pub fn verify(token: &UnblindedToken) -> BlindSignatureVerificationResult {
        let pk_bytes = match hex::decode(&token.signer_public_key_hex) {
            Ok(b) if b.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&b);
                CompressedEdwardsY(arr)
            }
            _ => {
                return BlindSignatureVerificationResult {
                    is_valid: false,
                    message: token.message.clone(),
                    signer_public_key_hex: token.signer_public_key_hex.clone(),
                    protocol: "Curve25519 Schnorr Blind Signature".to_string(),
                    error: Some("Invalid signer public key encoding".to_string()),
                }
            }
        };

        let signer_pk = match pk_bytes.decompress() {
            Some(p) => p,
            None => {
                return BlindSignatureVerificationResult {
                    is_valid: false,
                    message: token.message.clone(),
                    signer_public_key_hex: token.signer_public_key_hex.clone(),
                    protocol: "Curve25519 Schnorr Blind Signature".to_string(),
                    error: Some("Public key point decompression failed".to_string()),
                }
            }
        };

        let c_prime_bytes = match hex::decode(&token.challenge_c_prime_hex) {
            Ok(b) if b.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&b);
                arr
            }
            _ => {
                return BlindSignatureVerificationResult {
                    is_valid: false,
                    message: token.message.clone(),
                    signer_public_key_hex: token.signer_public_key_hex.clone(),
                    protocol: "Curve25519 Schnorr Blind Signature".to_string(),
                    error: Some("Invalid c_prime scalar encoding".to_string()),
                }
            }
        };

        let s_prime_bytes = match hex::decode(&token.response_s_prime_hex) {
            Ok(b) if b.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&b);
                arr
            }
            _ => {
                return BlindSignatureVerificationResult {
                    is_valid: false,
                    message: token.message.clone(),
                    signer_public_key_hex: token.signer_public_key_hex.clone(),
                    protocol: "Curve25519 Schnorr Blind Signature".to_string(),
                    error: Some("Invalid s_prime scalar encoding".to_string()),
                }
            }
        };

        let c_prime = Scalar::from_bytes_mod_order(c_prime_bytes);
        let s_prime = Scalar::from_bytes_mod_order(s_prime_bytes);

        // Recompute reconstructed R' = s' * G + c' * Y
        let r_reconstructed = (ED25519_BASEPOINT_POINT * s_prime) + (signer_pk * c_prime);
        let r_reconstructed_compressed = r_reconstructed.compress();

        let mut hasher = Sha512::new();
        hasher.update(b"AEGIS_SCHNORR_BLIND_SIGNATURE_CHALLENGE_v2.5");
        hasher.update(pk_bytes.as_bytes());
        hasher.update(r_reconstructed_compressed.as_bytes());
        hasher.update(token.message.as_bytes());
        let expected_c_prime = Scalar::from_hash(hasher);

        let is_valid = c_prime == expected_c_prime;

        BlindSignatureVerificationResult {
            is_valid,
            message: token.message.clone(),
            signer_public_key_hex: token.signer_public_key_hex.clone(),
            protocol: "Curve25519 Schnorr Blind Signature".to_string(),
            error: if is_valid {
                None
            } else {
                Some("Cryptographic signature equation verification failed".to_string())
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blind_signature_issue_unblind_and_verify() {
        let authority_seed = [0x77u8; 32];
        let authority = BlindSignatureAuthority::new(&authority_seed);
        let k_seed = [0x88u8; 32];
        let (k, r_compressed) = authority.generate_nonce_commitment(&k_seed);
        let r_point = r_compressed.decompress().unwrap();

        let message = "AEGIS_ANONYMOUS_SETTLEMENT_COUPON_SERIAL_#99281";
        let alpha_seed = [0x11u8; 32];
        let beta_seed = [0x22u8; 32];

        // 1. Client blinds
        let (blinded_c, c_prime, alpha, _) = BlindSignatureClient::blind_message(
            &authority.public_key,
            &r_point,
            message,
            &alpha_seed,
            &beta_seed,
        );

        // 2. Signer signs blinded challenge without knowing message or alpha/beta
        let s = authority.sign_blinded(&k, &blinded_c);

        // 3. Client unblinds response
        let s_prime = BlindSignatureClient::unblind_response(&s, &alpha);

        let token = UnblindedToken {
            message: message.to_string(),
            challenge_c_prime_hex: hex::encode(c_prime.as_bytes()),
            response_s_prime_hex: hex::encode(s_prime.as_bytes()),
            signer_public_key_hex: hex::encode(authority.public_key.compress().as_bytes()),
        };

        // 4. Any third party verifies unblinded token
        let result = BlindSignatureClient::verify(&token);
        assert!(result.is_valid, "Valid blind signature must verify: {:?}", result.error);
        assert_eq!(result.message, message);
    }

    #[test]
    fn test_blind_signature_tampered_token_fails() {
        let authority = BlindSignatureAuthority::new(&[0x12u8; 32]);
        let (k, r_comp) = authority.generate_nonce_commitment(&[0x34u8; 32]);
        let r_point = r_comp.decompress().unwrap();

        let msg = "PAYMENT_AMOUNT_10_CREDITS";
        let (blinded_c, c_prime, alpha, _) = BlindSignatureClient::blind_message(
            &authority.public_key,
            &r_point,
            msg,
            &[0x55u8; 32],
            &[0x66u8; 32],
        );

        let s = authority.sign_blinded(&k, &blinded_c);
        let s_prime = BlindSignatureClient::unblind_response(&s, &alpha);

        // Adversary alters message to 1000 credits
        let tampered_token = UnblindedToken {
            message: "PAYMENT_AMOUNT_1000_CREDITS".to_string(),
            challenge_c_prime_hex: hex::encode(c_prime.as_bytes()),
            response_s_prime_hex: hex::encode(s_prime.as_bytes()),
            signer_public_key_hex: hex::encode(authority.public_key.compress().as_bytes()),
        };

        let result = BlindSignatureClient::verify(&tampered_token);
        assert!(!result.is_valid, "Tampered payload must fail verification");
    }
}
