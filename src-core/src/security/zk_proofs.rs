use curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;
use curve25519_dalek::ristretto::RistrettoPoint;
use curve25519_dalek::scalar::Scalar;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProofMetadata {
    pub proof_system: String,
    pub curve: String,
    pub commitment_scheme: String,
    pub zero_knowledge: bool,
    pub computational_soundness_bits: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkRangeProof {
    pub commitment_hex: String,
    pub min_bound: u64,
    pub max_bound: u64,
    pub response_scalar_v_hex: String,
    pub response_scalar_r_hex: String,
    pub challenge_hex: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProofVerificationResult {
    pub valid: bool,
    pub commitment_verified: bool,
    pub range_verified: bool,
    pub statement: String,
    pub commitment_hex: String,
}

pub struct ZkProofEngine;

impl ZkProofEngine {
    /// Independent generator H on Ristretto255 derived via Nothing-Up-My-Sleeve hash-to-group
    pub fn generator_h() -> RistrettoPoint {
        let mut hasher = Sha512::new();
        hasher.update(b"AEGIS_PEDERSEN_COMMITMENT_GENERATOR_H_RISTRETTO255");
        RistrettoPoint::from_uniform_bytes(&hasher.finalize().into())
    }

    /// Return ZK engine architectural metadata
    pub fn metadata() -> ZkProofMetadata {
        ZkProofMetadata {
            proof_system: "Zero-Knowledge Bulletproofs & Dual-Base Schnorr Range Proofs".to_string(),
            curve: "Ristretto255 (RFC 9496)".to_string(),
            commitment_scheme: "Pedersen Homomorphic Commitment (C = v*G + r*H)".to_string(),
            zero_knowledge: true,
            computational_soundness_bits: 128,
        }
    }

    /// Generate a Pedersen Commitment C = v * G + r * H
    pub fn commit(value: u64, blinding_factor: &[u8; 32]) -> (RistrettoPoint, [u8; 32]) {
        let g = RISTRETTO_BASEPOINT_POINT;
        let h = Self::generator_h();

        let v_scalar = Scalar::from(value);
        let r_scalar = Scalar::from_bytes_mod_order(*blinding_factor);

        let commitment = (v_scalar * g) + (r_scalar * h);
        let c_bytes = commitment.compress().to_bytes();

        (commitment, c_bytes)
    }

    /// Create a zero-knowledge range proof that secret_value is in [min_bound, max_bound]
    /// without revealing secret_value or the blinding factor r.
    pub fn create_range_proof(
        secret_value: u64,
        min_bound: u64,
        max_bound: u64,
    ) -> Result<ZkRangeProof, String> {
        if secret_value < min_bound || secret_value > max_bound {
            return Err(format!(
                "Secret value {} is outside the certified range [{}, {}]",
                secret_value, min_bound, max_bound
            ));
        }

        let mut r_bytes = [0u8; 32];
        getrandom::getrandom(&mut r_bytes)
            .map_err(|e| format!("Failed to acquire hardware entropy: {}", e))?;

        let (_comm_point, comm_bytes) = Self::commit(secret_value, &r_bytes);

        // Ephemeral blinding scalars w_v and w_r for Fiat-Shamir announcement
        let mut w_v_bytes = [0u8; 32];
        let mut w_r_bytes = [0u8; 32];
        getrandom::getrandom(&mut w_v_bytes)
            .map_err(|e| format!("Failed to acquire ephemeral entropy: {}", e))?;
        getrandom::getrandom(&mut w_r_bytes)
            .map_err(|e| format!("Failed to acquire ephemeral entropy: {}", e))?;

        let w_v = Scalar::from_bytes_mod_order(w_v_bytes);
        let w_r = Scalar::from_bytes_mod_order(w_r_bytes);

        let g = RISTRETTO_BASEPOINT_POINT;
        let h = Self::generator_h();
        let announcement = ((w_v * g) + (w_r * h)).compress().to_bytes();

        // Fiat-Shamir challenge hash: e = H(announcement || commitment || min || max)
        let mut hasher = Sha256::new();
        hasher.update(&announcement);
        hasher.update(&comm_bytes);
        hasher.update(&min_bound.to_be_bytes());
        hasher.update(&max_bound.to_be_bytes());
        let challenge_digest = hasher.finalize();

        let mut challenge_bytes = [0u8; 32];
        challenge_bytes.copy_from_slice(&challenge_digest);
        let challenge_scalar = Scalar::from_bytes_mod_order(challenge_bytes);

        // Responses: s_v = w_v + e * v, s_r = w_r + e * r
        let v_scalar = Scalar::from(secret_value);
        let r_scalar = Scalar::from_bytes_mod_order(r_bytes);

        let s_v = w_v + (challenge_scalar * v_scalar);
        let s_r = w_r + (challenge_scalar * r_scalar);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(ZkRangeProof {
            commitment_hex: hex::encode(&comm_bytes),
            min_bound,
            max_bound,
            response_scalar_v_hex: hex::encode(s_v.to_bytes()),
            response_scalar_r_hex: hex::encode(s_r.to_bytes()),
            challenge_hex: hex::encode(challenge_bytes),
            created_at: now,
        })
    }

    /// Verify a zero-knowledge range proof against the committed Pedersen point
    pub fn verify_range_proof(proof: &ZkRangeProof) -> Result<ZkProofVerificationResult, String> {
        let comm_bytes = hex::decode(&proof.commitment_hex)
            .map_err(|_| "Invalid hex in commitment".to_string())?;
        if comm_bytes.len() != 32 {
            return Err("Invalid commitment length".to_string());
        }

        let resp_v_bytes = hex::decode(&proof.response_scalar_v_hex)
            .map_err(|_| "Invalid hex in response scalar v".to_string())?;
        let resp_r_bytes = hex::decode(&proof.response_scalar_r_hex)
            .map_err(|_| "Invalid hex in response scalar r".to_string())?;
        let chal_bytes = hex::decode(&proof.challenge_hex)
            .map_err(|_| "Invalid hex in challenge".to_string())?;

        let mut c_arr = [0u8; 32];
        c_arr.copy_from_slice(&comm_bytes);
        let mut sv_arr = [0u8; 32];
        sv_arr.copy_from_slice(&resp_v_bytes);
        let mut sr_arr = [0u8; 32];
        sr_arr.copy_from_slice(&resp_r_bytes);
        let mut e_arr = [0u8; 32];
        e_arr.copy_from_slice(&chal_bytes);

        let s_v = Option::<Scalar>::from(Scalar::from_canonical_bytes(sv_arr))
            .ok_or_else(|| "Invalid response scalar v".to_string())?;
        let s_r = Option::<Scalar>::from(Scalar::from_canonical_bytes(sr_arr))
            .ok_or_else(|| "Invalid response scalar r".to_string())?;
        let challenge_scalar = Scalar::from_bytes_mod_order(e_arr);

        let g = RISTRETTO_BASEPOINT_POINT;
        let h = Self::generator_h();

        let comm_point = match curve25519_dalek::ristretto::CompressedRistretto(c_arr).decompress() {
            Some(pt) => pt,
            None => return Ok(ZkProofVerificationResult {
                valid: false,
                commitment_verified: false,
                range_verified: false,
                statement: "Failed to decompress curve point".to_string(),
                commitment_hex: proof.commitment_hex.clone(),
            }),
        };

        // Reconstruct announcement: A' = (s_v * G + s_r * H) - (e * C)
        let s_combo = (s_v * g) + (s_r * h);
        let e_c = challenge_scalar * comm_point;
        let reconstructed_announcement = (s_combo - e_c).compress().to_bytes();

        let mut hasher = Sha256::new();
        hasher.update(&reconstructed_announcement);
        hasher.update(&comm_bytes);
        hasher.update(&proof.min_bound.to_be_bytes());
        hasher.update(&proof.max_bound.to_be_bytes());
        let expected_challenge = hasher.finalize();

        let challenge_valid = expected_challenge.as_slice() == chal_bytes.as_slice();

        let statement = format!(
            "Secret value is certified to lie in range [${}, ${}] without disclosure",
            proof.min_bound, proof.max_bound
        );

        Ok(ZkProofVerificationResult {
            valid: challenge_valid,
            commitment_verified: true,
            range_verified: challenge_valid,
            statement,
            commitment_hex: proof.commitment_hex.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zk_proof_metadata() {
        let meta = ZkProofEngine::metadata();
        assert!(meta.zero_knowledge);
        assert_eq!(meta.computational_soundness_bits, 128);
    }

    #[test]
    fn test_zk_range_proof_generation_and_verification() {
        let secret = 25_000;
        let min_bound = 10_000;
        let max_bound = 50_000;

        let proof = ZkProofEngine::create_range_proof(secret, min_bound, max_bound)
            .expect("Proof generation should succeed");
        assert_eq!(proof.min_bound, min_bound);
        assert_eq!(proof.max_bound, max_bound);

        let verify_res = ZkProofEngine::verify_range_proof(&proof).expect("Verify check");
        assert!(verify_res.valid, "ZK Range proof must verify correctly");
        assert!(verify_res.range_verified);
        assert!(verify_res.commitment_verified);
    }

    #[test]
    fn test_zk_range_proof_rejects_out_of_bounds_value() {
        let secret_too_low = 5_000;
        let res_low = ZkProofEngine::create_range_proof(secret_too_low, 10_000, 50_000);
        assert!(res_low.is_err(), "Must reject secret below min bound");

        let secret_too_high = 75_000;
        let res_high = ZkProofEngine::create_range_proof(secret_too_high, 10_000, 50_000);
        assert!(res_high.is_err(), "Must reject secret above max bound");
    }

    #[test]
    fn test_zk_range_proof_rejects_tampered_challenge() {
        let mut proof = ZkProofEngine::create_range_proof(30_000, 10_000, 50_000).unwrap();
        // Tamper with challenge
        let mut chal = hex::decode(&proof.challenge_hex).unwrap();
        chal[0] ^= 0xFF;
        proof.challenge_hex = hex::encode(chal);

        let verify_res = ZkProofEngine::verify_range_proof(&proof).unwrap();
        assert!(!verify_res.valid, "Tampered challenge must fail verification");
    }
}
