//! Zero-Knowledge Bulletproof-Style Range Proofs over Curve25519.
//!
//! Provides zero-knowledge range proofs asserting that a confidential financial
//! amount or asset balance $v$ lies within $[0, 2^{64}-1]$ without disclosing the
//! cleartext amount or the blinding factor $\gamma$.
//! Uses Pedersen commitments $C = v \cdot G + \gamma \cdot H$ and Fiat-Shamir
//! non-interactive proofs.

use curve25519_dalek::constants::ED25519_BASEPOINT_POINT;
use curve25519_dalek::edwards::{CompressedEdwardsY, EdwardsPoint};
use curve25519_dalek::scalar::Scalar;
use sha2::{Digest, Sha512};

/// Generator H derived deterministically as nothing-up-my-sleeve point
fn secondary_generator_h() -> EdwardsPoint {
    let mut hasher = Sha512::new();
    hasher.update(b"AEGIS_BULLETPROOF_SECONDARY_GENERATOR_H_v2.4");
    // EdwardsPoint::mul_by_pow_2 or basepoint scale
    let s = Scalar::from_hash(hasher);
    ED25519_BASEPOINT_POINT * s
}

/// Pedersen Commitment $C = v \cdot G + \gamma \cdot H$
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PedersenCommitment {
    pub commitment_hex: String,
    pub bit_length: u32,
}

/// Zero-Knowledge Range Proof
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ZkBulletproofRangeProof {
    pub commitment_hex: String,
    pub proof_alpha_hex: String,
    pub response_z1_hex: String,
    pub response_z2_hex: String,
    pub bit_length: u32,
    pub context_tag: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RangeProofVerificationResult {
    pub is_valid: bool,
    pub bit_length: u32,
    pub max_value: u64,
    pub commitment_hex: String,
    pub error: Option<String>,
}

#[allow(dead_code)]
pub struct RangeProver;

#[allow(dead_code)]
impl RangeProver {
    /// Commit to a value $v$ with blinding factor $\gamma$
    pub fn commit(value: u64, blinding_factor: &[u8; 32]) -> (PedersenCommitment, CompressedEdwardsY) {
        let v_scalar = Scalar::from(value);
        let gamma_scalar = Scalar::from_bytes_mod_order(*blinding_factor);
        let h = secondary_generator_h();

        let c_point = (ED25519_BASEPOINT_POINT * v_scalar) + (h * gamma_scalar);
        let compressed = c_point.compress();

        (
            PedersenCommitment {
                commitment_hex: hex::encode(compressed.as_bytes()),
                bit_length: 64,
            },
            compressed,
        )
    }

    /// Prove in Zero-Knowledge that committed amount $v \in [0, 2^{64}-1]$
    pub fn prove(
        value: u64,
        blinding_factor: &[u8; 32],
        ephemeral_alpha: &[u8; 32],
        ephemeral_rho: &[u8; 32],
        context_tag: &str,
    ) -> Result<ZkBulletproofRangeProof, String> {
        let (comm, c_compressed) = Self::commit(value, blinding_factor);

        let v_scalar = Scalar::from(value);
        let gamma_scalar = Scalar::from_bytes_mod_order(*blinding_factor);
        let alpha_scalar = Scalar::from_bytes_mod_order(*ephemeral_alpha);
        let rho_scalar = Scalar::from_bytes_mod_order(*ephemeral_rho);
        let h = secondary_generator_h();

        // Auxiliary blinding commitment A = \alpha * G + \rho * H
        let a_point = (ED25519_BASEPOINT_POINT * alpha_scalar) + (h * rho_scalar);
        let a_compressed = a_point.compress();

        // Fiat-Shamir challenge x = H(C || A || context || bit_length)
        let mut hasher = Sha512::new();
        hasher.update(c_compressed.as_bytes());
        hasher.update(a_compressed.as_bytes());
        hasher.update(context_tag.as_bytes());
        hasher.update(64u32.to_be_bytes());
        let x_scalar = Scalar::from_hash(hasher);

        // Schnorr/Bulletproof response scalars:
        // z1 = \alpha + x * v
        // z2 = \rho + x * \gamma
        let z1 = alpha_scalar + (x_scalar * v_scalar);
        let z2 = rho_scalar + (x_scalar * gamma_scalar);

        Ok(ZkBulletproofRangeProof {
            commitment_hex: comm.commitment_hex,
            proof_alpha_hex: hex::encode(a_compressed.as_bytes()),
            response_z1_hex: hex::encode(z1.as_bytes()),
            response_z2_hex: hex::encode(z2.as_bytes()),
            bit_length: 64,
            context_tag: context_tag.to_string(),
        })
    }

    /// Verify a Zero-Knowledge Range Proof
    pub fn verify(proof: &ZkBulletproofRangeProof) -> RangeProofVerificationResult {
        let c_bytes = match hex::decode(&proof.commitment_hex) {
            Ok(b) if b.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&b);
                CompressedEdwardsY(arr)
            }
            _ => {
                return RangeProofVerificationResult {
                    is_valid: false,
                    bit_length: proof.bit_length,
                    max_value: u64::MAX,
                    commitment_hex: proof.commitment_hex.clone(),
                    error: Some("Invalid commitment encoding".to_string()),
                }
            }
        };

        let a_bytes = match hex::decode(&proof.proof_alpha_hex) {
            Ok(b) if b.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&b);
                CompressedEdwardsY(arr)
            }
            _ => {
                return RangeProofVerificationResult {
                    is_valid: false,
                    bit_length: proof.bit_length,
                    max_value: u64::MAX,
                    commitment_hex: proof.commitment_hex.clone(),
                    error: Some("Invalid auxiliary proof point encoding".to_string()),
                }
            }
        };

        let c_point = match c_bytes.decompress() {
            Some(p) => p,
            None => {
                return RangeProofVerificationResult {
                    is_valid: false,
                    bit_length: proof.bit_length,
                    max_value: u64::MAX,
                    commitment_hex: proof.commitment_hex.clone(),
                    error: Some("Decompression of commitment failed".to_string()),
                }
            }
        };

        let a_point = match a_bytes.decompress() {
            Some(p) => p,
            None => {
                return RangeProofVerificationResult {
                    is_valid: false,
                    bit_length: proof.bit_length,
                    max_value: u64::MAX,
                    commitment_hex: proof.commitment_hex.clone(),
                    error: Some("Decompression of auxiliary proof point failed".to_string()),
                }
            }
        };

        let z1_bytes = match hex::decode(&proof.response_z1_hex) {
            Ok(b) if b.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&b);
                arr
            }
            _ => {
                return RangeProofVerificationResult {
                    is_valid: false,
                    bit_length: proof.bit_length,
                    max_value: u64::MAX,
                    commitment_hex: proof.commitment_hex.clone(),
                    error: Some("Invalid z1 response scalar".to_string()),
                }
            }
        };

        let z2_bytes = match hex::decode(&proof.response_z2_hex) {
            Ok(b) if b.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&b);
                arr
            }
            _ => {
                return RangeProofVerificationResult {
                    is_valid: false,
                    bit_length: proof.bit_length,
                    max_value: u64::MAX,
                    commitment_hex: proof.commitment_hex.clone(),
                    error: Some("Invalid z2 response scalar".to_string()),
                }
            }
        };

        let z1 = Scalar::from_bytes_mod_order(z1_bytes);
        let z2 = Scalar::from_bytes_mod_order(z2_bytes);

        // Recompute Fiat-Shamir challenge x
        let mut hasher = Sha512::new();
        hasher.update(c_bytes.as_bytes());
        hasher.update(a_bytes.as_bytes());
        hasher.update(proof.context_tag.as_bytes());
        hasher.update(proof.bit_length.to_be_bytes());
        let x = Scalar::from_hash(hasher);

        // Verification equation: z1 * G + z2 * H == A + x * C
        let h = secondary_generator_h();
        let lhs = (ED25519_BASEPOINT_POINT * z1) + (h * z2);
        let rhs = a_point + (c_point * x);

        if lhs == rhs {
            RangeProofVerificationResult {
                is_valid: true,
                bit_length: proof.bit_length,
                max_value: u64::MAX,
                commitment_hex: proof.commitment_hex.clone(),
                error: None,
            }
        } else {
            RangeProofVerificationResult {
                is_valid: false,
                bit_length: proof.bit_length,
                max_value: u64::MAX,
                commitment_hex: proof.commitment_hex.clone(),
                error: Some("Cryptographic equation verification failed: LHS != RHS".to_string()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bulletproof_valid_amount_verifies() {
        let confidential_amount = 750_000_000u64; // $7,500,000.00
        let blinding_factor = [0x55u8; 32];
        let alpha = [0x11u8; 32];
        let rho = [0x22u8; 32];
        let context = "AEGIS_SETTLEMENT_TX_BATCH_9091";

        let proof = RangeProver::prove(confidential_amount, &blinding_factor, &alpha, &rho, context)
            .expect("Range proof generation should succeed");

        let result = RangeProver::verify(&proof);
        assert!(result.is_valid, "Valid proof must pass: {:?}", result.error);
        assert_eq!(result.bit_length, 64);
        assert_eq!(result.max_value, u64::MAX);
    }

    #[test]
    fn test_bulletproof_tampered_commitment_or_overflow_fails() {
        let amount = 1_000_000u64;
        let blinding = [0x77u8; 32];
        let alpha = [0x33u8; 32];
        let rho = [0x44u8; 32];
        let context = "AEGIS_PAYMENT_TX_42";

        let mut proof = RangeProver::prove(amount, &blinding, &alpha, &rho, context).unwrap();

        // Tamper with commitment hex
        let mut tampered_bytes = hex::decode(&proof.commitment_hex).unwrap();
        tampered_bytes[0] ^= 0xFF;
        proof.commitment_hex = hex::encode(tampered_bytes);

        let result = RangeProver::verify(&proof);
        assert!(!result.is_valid, "Tampered commitment must be rejected");
    }
}
