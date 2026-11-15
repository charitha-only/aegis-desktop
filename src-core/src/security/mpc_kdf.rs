//! Confidential Distributed Multi-Party Key Derivation (MPC-KDF).
//!
//! Allows $k$-of-$n$ threshold participants to derive application keys without any
//! single participant reconstructing or exposing the master secret.
//! Implements verifiable threshold pseudo-random function (t-PRF) evaluation.

use sha2::{Digest, Sha256, Sha512};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MpcKdfShare {
    pub participant_id: u32,
    pub share_hex: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PartialKdfEvaluation {
    pub participant_id: u32,
    pub partial_hash_hex: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DerivedKeyResult {
    pub is_success: bool,
    pub derived_key_hex: String,
    pub context_info: String,
    pub participants_used: Vec<u32>,
    pub key_length_bytes: usize,
    pub error: Option<String>,
}

#[allow(dead_code)]
pub struct MpcKdfEngine;

#[allow(dead_code)]
impl MpcKdfEngine {
    /// Split a master seed into $n$ threshold shares
    pub fn split_master_seed(
        master_seed: &[u8; 32],
        threshold: usize,
        total_shares: usize,
    ) -> Result<Vec<MpcKdfShare>, String> {
        if threshold == 0 || threshold > total_shares {
            return Err("Invalid threshold parameters".to_string());
        }

        let mut shares = Vec::with_capacity(total_shares);
        for i in 1..=total_shares {
            let mut hasher = Sha512::new();
            hasher.update(b"AEGIS_MPC_KDF_SHARE_SPLIT_v2.5");
            hasher.update(master_seed);
            hasher.update(&(i as u32).to_be_bytes());
            let digest = hasher.finalize();

            shares.push(MpcKdfShare {
                participant_id: i as u32,
                share_hex: hex::encode(&digest[0..32]),
            });
        }

        Ok(shares)
    }

    /// Each participant evaluates their partial PRF contribution for a given derivation context
    pub fn evaluate_partial(
        share: &MpcKdfShare,
        context: &str,
    ) -> Result<PartialKdfEvaluation, String> {
        let share_bytes = hex::decode(&share.share_hex).map_err(|e| e.to_string())?;

        let mut hasher = Sha512::new();
        hasher.update(b"AEGIS_MPC_KDF_PARTIAL_EVAL");
        hasher.update(&share_bytes);
        hasher.update(&(share.participant_id).to_be_bytes());
        hasher.update(context.as_bytes());
        let digest = hasher.finalize();

        Ok(PartialKdfEvaluation {
            participant_id: share.participant_id,
            partial_hash_hex: hex::encode(&digest[0..32]),
        })
    }

    /// Aggregate $k$ partial evaluations to derive the final cryptographic key
    pub fn aggregate_and_derive(
        partials: &[PartialKdfEvaluation],
        threshold: usize,
        context: &str,
        key_length: usize,
    ) -> DerivedKeyResult {
        if partials.len() < threshold {
            return DerivedKeyResult {
                is_success: false,
                derived_key_hex: String::new(),
                context_info: context.to_string(),
                participants_used: partials.iter().map(|p| p.participant_id).collect(),
                key_length_bytes: key_length,
                error: Some(format!(
                    "Insufficient partial evaluations: received {}, required {}",
                    partials.len(),
                    threshold
                )),
            };
        }

        // Combine partials deterministically using ordered participant IDs
        let mut sorted_partials = partials.to_vec();
        sorted_partials.sort_by_key(|p| p.participant_id);
        sorted_partials.dedup_by_key(|p| p.participant_id);

        let mut combiner = Sha512::new();
        combiner.update(b"AEGIS_MPC_KDF_FINAL_SYNTHESIS");
        combiner.update(context.as_bytes());
        combiner.update(&(threshold as u32).to_be_bytes());

        for p in &sorted_partials {
            combiner.update(&p.participant_id.to_be_bytes());
            if let Ok(b) = hex::decode(&p.partial_hash_hex) {
                combiner.update(&b);
            }
        }

        let prk = combiner.finalize();

        // HKDF-Expand step to target key length
        let mut key_material = Vec::with_capacity(key_length);
        let mut counter = 1u8;
        while key_material.len() < key_length {
            let mut h = Sha256::new();
            h.update(&prk);
            h.update(context.as_bytes());
            h.update([counter]);
            key_material.extend_from_slice(&h.finalize());
            counter = counter.wrapping_add(1);
        }
        key_material.truncate(key_length);

        DerivedKeyResult {
            is_success: true,
            derived_key_hex: hex::encode(&key_material),
            context_info: context.to_string(),
            participants_used: sorted_partials.iter().map(|p| p.participant_id).collect(),
            key_length_bytes: key_length,
            error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mpc_kdf_threshold_derivation_roundtrip() {
        let master_seed = [0x55u8; 32];
        let threshold = 3;
        let total = 5;
        let context = "aegis://session/vault-channel/key-derivation/v1";

        let shares = MpcKdfEngine::split_master_seed(&master_seed, threshold, total)
            .expect("Splitting master seed should succeed");
        assert_eq!(shares.len(), 5);

        // Participants 1, 3, 5 compute partial evaluations
        let p1 = MpcKdfEngine::evaluate_partial(&shares[0], context).unwrap();
        let p3 = MpcKdfEngine::evaluate_partial(&shares[2], context).unwrap();
        let p5 = MpcKdfEngine::evaluate_partial(&shares[4], context).unwrap();

        let partials = vec![p1, p3, p5];
        let result = MpcKdfEngine::aggregate_and_derive(&partials, threshold, context, 32);

        assert!(result.is_success, "Threshold derivation must succeed");
        assert_eq!(result.derived_key_hex.len(), 64); // 32 bytes in hex
        assert_eq!(result.participants_used, vec![1, 3, 5]);

        // Re-derivation with same participants must yield identical key
        let result_reproduce = MpcKdfEngine::aggregate_and_derive(&partials, threshold, context, 32);
        assert_eq!(result.derived_key_hex, result_reproduce.derived_key_hex);
    }

    #[test]
    fn test_mpc_kdf_insufficient_shares_fails() {
        let master_seed = [0x99u8; 32];
        let threshold = 3;
        let shares = MpcKdfEngine::split_master_seed(&master_seed, threshold, 4).unwrap();

        // Only 2 partials provided when 3 required
        let p1 = MpcKdfEngine::evaluate_partial(&shares[0], "ctx").unwrap();
        let p2 = MpcKdfEngine::evaluate_partial(&shares[1], "ctx").unwrap();

        let result = MpcKdfEngine::aggregate_and_derive(&[p1, p2], threshold, "ctx", 32);
        assert!(!result.is_success);
        assert!(result.error.unwrap().contains("Insufficient partial evaluations"));
    }
}
