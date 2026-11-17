//! Multi-Party Threshold BLS / Aggregated Signatures Engine.
//!
//! Provides non-interactive threshold signature aggregation over Curve25519.
//! Enables t-of-n participants to contribute partial signatures that can be aggregated
//! into a single compact signature verifiable in O(1) time against the group public key.

use curve25519_dalek::constants::ED25519_BASEPOINT_POINT;
use curve25519_dalek::edwards::{CompressedEdwardsY, EdwardsPoint};
use curve25519_dalek::scalar::Scalar;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use zeroize::Zeroize;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlsKeyShare {
    pub participant_id: u32,
    pub threshold: u32,
    pub total_signers: u32,
    pub public_key_hex: String,
    pub group_public_key_hex: String,
    #[serde(skip_serializing)]
    pub secret_scalar_hex: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlsPartialSignature {
    pub participant_id: u32,
    pub signature_scalar_hex: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlsAggregatedSignature {
    pub aggregated_signature_hex: String,
    pub participant_ids: Vec<u32>,
    pub message_hash_hex: String,
    pub group_public_key_hex: String,
}

#[allow(dead_code)]
pub struct ThresholdBlsEngine;

#[allow(dead_code)]
impl ThresholdBlsEngine {
    /// Hash an arbitrary message bytes to a non-zero Scalar in the Curve25519 field.
    pub fn hash_message_to_scalar(message: &[u8]) -> Scalar {
        let mut hasher = Sha512::new();
        hasher.update(b"AEGIS_THRESHOLD_BLS_CURVE25519_V2_8_0");
        hasher.update(message);
        Scalar::from_hash(hasher)
    }

    /// Generate $t$-of-$n$ Shamir secret key shares and corresponding public keys.
    pub fn generate_key_shares(
        threshold: u32,
        total_signers: u32,
        master_seed: &[u8; 32],
    ) -> Result<Vec<BlsKeyShare>, String> {
        if threshold == 0 || threshold > total_signers {
            return Err("Invalid threshold: threshold must be >= 1 and <= total_signers".to_string());
        }

        // Derive master secret scalar
        let mut master_hasher = Sha512::new();
        master_hasher.update(b"AEGIS_BLS_MASTER_KEY");
        master_hasher.update(master_seed);
        let mut master_secret = Scalar::from_hash(master_hasher);
        let group_pubkey = (ED25519_BASEPOINT_POINT * master_secret).compress();
        let group_pubkey_hex = hex::encode(group_pubkey.as_bytes());

        // Derive polynomial coefficients: f(x) = master_secret + a_1*x + ... + a_{t-1}*x^{t-1}
        let mut coefficients = Vec::with_capacity(threshold as usize);
        coefficients.push(master_secret);

        for idx in 1..threshold {
            let mut coeff_hasher = Sha512::new();
            coeff_hasher.update(b"AEGIS_BLS_POLY_COEFF");
            coeff_hasher.update(master_seed);
            coeff_hasher.update(&idx.to_be_bytes());
            coefficients.push(Scalar::from_hash(coeff_hasher));
        }

        let mut shares = Vec::with_capacity(total_signers as usize);
        for id in 1..=total_signers {
            let x = Scalar::from(id);
            let mut eval = Scalar::ZERO;
            let mut x_pow = Scalar::ONE;

            for coeff in &coefficients {
                eval += coeff * x_pow;
                x_pow *= x;
            }

            let member_pubkey = (ED25519_BASEPOINT_POINT * eval).compress();
            let mut secret_bytes = eval.to_bytes();
            let secret_hex = hex::encode(secret_bytes);
            secret_bytes.zeroize();

            shares.push(BlsKeyShare {
                participant_id: id,
                threshold,
                total_signers,
                public_key_hex: hex::encode(member_pubkey.as_bytes()),
                group_public_key_hex: group_pubkey_hex.clone(),
                secret_scalar_hex: secret_hex,
            });
        }

        master_secret.zeroize();
        Ok(shares)
    }

    /// Sign a message partial using a participant's key share.
    pub fn sign_partial(
        share: &BlsKeyShare,
        message: &[u8],
    ) -> Result<BlsPartialSignature, String> {
        let secret_bytes = hex::decode(&share.secret_scalar_hex)
            .map_err(|e| format!("Invalid secret scalar hex: {}", e))?;
        if secret_bytes.len() != 32 {
            return Err("Invalid secret scalar length".to_string());
        }

        let mut secret_arr = [0u8; 32];
        secret_arr.copy_from_slice(&secret_bytes);
        let secret_scalar = Scalar::from_bytes_mod_order(secret_arr);
        secret_arr.zeroize();

        let h_m = Self::hash_message_to_scalar(message);
        let partial_signature = secret_scalar * h_m;

        Ok(BlsPartialSignature {
            participant_id: share.participant_id,
            signature_scalar_hex: hex::encode(partial_signature.to_bytes()),
        })
    }

    /// Verify a partial signature against the member's public key share.
    pub fn verify_partial(
        pubkey_hex: &str,
        message: &[u8],
        partial: &BlsPartialSignature,
    ) -> bool {
        let pubkey_bytes = match hex::decode(pubkey_hex) {
            Ok(b) if b.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&b);
                CompressedEdwardsY(arr)
            }
            _ => return false,
        };

        let member_pubkey = match pubkey_bytes.decompress() {
            Some(p) => p,
            None => return false,
        };

        let sig_bytes = match hex::decode(&partial.signature_scalar_hex) {
            Ok(b) if b.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&b);
                arr
            }
            _ => return false,
        };

        let sig_scalar = Scalar::from_bytes_mod_order(sig_bytes);
        let h_m = Self::hash_message_to_scalar(message);

        // Verification equation: sig_scalar * G == h_m * P_member
        let lhs = ED25519_BASEPOINT_POINT * sig_scalar;
        let rhs = member_pubkey * h_m;

        lhs == rhs
    }

    /// Aggregate $t$ partial signatures using Lagrange coefficients.
    pub fn aggregate_signatures(
        threshold: u32,
        group_pubkey_hex: &str,
        message: &[u8],
        partials: &[BlsPartialSignature],
    ) -> Result<BlsAggregatedSignature, String> {
        if (partials.len() as u32) < threshold {
            return Err(format!(
                "Insufficient partial signatures: got {}, threshold requires {}",
                partials.len(),
                threshold
            ));
        }

        let selected = &partials[0..(threshold as usize)];
        let participant_ids: Vec<u32> = selected.iter().map(|p| p.participant_id).collect();

        // Check for duplicates
        for i in 0..participant_ids.len() {
            for j in (i + 1)..participant_ids.len() {
                if participant_ids[i] == participant_ids[j] {
                    return Err(format!("Duplicate participant ID detected: {}", participant_ids[i]));
                }
            }
        }

        let mut aggregated_sig = Scalar::ZERO;

        // Lagrange interpolation at x = 0: \lambda_i = \prod_{j \neq i} \frac{x_j}{x_j - x_i}
        for (i, p) in selected.iter().enumerate() {
            let xi = Scalar::from(p.participant_id);
            let mut num = Scalar::ONE;
            let mut den = Scalar::ONE;

            for (j, other) in selected.iter().enumerate() {
                if i != j {
                    let xj = Scalar::from(other.participant_id);
                    num *= xj;
                    den *= xj - xi;
                }
            }

            let den_inv = den.invert();
            let lambda_i = num * den_inv;

            let sig_bytes = hex::decode(&p.signature_scalar_hex)
                .map_err(|e| format!("Invalid partial sig hex: {}", e))?;
            if sig_bytes.len() != 32 {
                return Err("Invalid partial signature byte length".to_string());
            }

            let mut arr = [0u8; 32];
            arr.copy_from_slice(&sig_bytes);
            let partial_scalar = Scalar::from_bytes_mod_order(arr);

            aggregated_sig += lambda_i * partial_scalar;
        }

        let msg_hash = Self::hash_message_to_scalar(message);

        Ok(BlsAggregatedSignature {
            aggregated_signature_hex: hex::encode(aggregated_sig.to_bytes()),
            participant_ids,
            message_hash_hex: hex::encode(msg_hash.to_bytes()),
            group_public_key_hex: group_pubkey_hex.to_string(),
        })
    }

    /// Verify an aggregated signature against the group public key in O(1) time.
    pub fn verify_aggregated(
        agg: &BlsAggregatedSignature,
        message: &[u8],
    ) -> Result<bool, String> {
        let group_pubkey_bytes = hex::decode(&agg.group_public_key_hex)
            .map_err(|e| format!("Invalid group public key hex: {}", e))?;
        if group_pubkey_bytes.len() != 32 {
            return Err("Invalid group public key length".to_string());
        }

        let mut arr = [0u8; 32];
        arr.copy_from_slice(&group_pubkey_bytes);
        let group_compressed = CompressedEdwardsY(arr);
        let group_point: EdwardsPoint = group_compressed
            .decompress()
            .ok_or_else(|| "Failed to decompress group public key point".to_string())?;

        let sig_bytes = hex::decode(&agg.aggregated_signature_hex)
            .map_err(|e| format!("Invalid aggregated signature hex: {}", e))?;
        if sig_bytes.len() != 32 {
            return Err("Invalid aggregated signature length".to_string());
        }

        let mut sig_arr = [0u8; 32];
        sig_arr.copy_from_slice(&sig_bytes);
        let agg_sig_scalar = Scalar::from_bytes_mod_order(sig_arr);

        let h_m = Self::hash_message_to_scalar(message);

        // Verification equation: agg_sig_scalar * G == h_m * P_group
        let lhs = ED25519_BASEPOINT_POINT * agg_sig_scalar;
        let rhs = group_point * h_m;

        Ok(lhs == rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threshold_signature_aggregation_and_verify() {
        let seed = [0x5Au8; 32];
        let threshold = 3;
        let total = 5;

        let shares = ThresholdBlsEngine::generate_key_shares(threshold, total, &seed)
            .expect("Key share generation must succeed");
        assert_eq!(shares.len(), 5);

        let message = b"CRITICAL_AUTHORIZATION_TRANSFER_USD_2500000";

        // Partial signatures from signers 1, 3, 5
        let sig1 = ThresholdBlsEngine::sign_partial(&shares[0], message).unwrap();
        let sig3 = ThresholdBlsEngine::sign_partial(&shares[2], message).unwrap();
        let sig5 = ThresholdBlsEngine::sign_partial(&shares[4], message).unwrap();

        // Individual partial verifications
        assert!(ThresholdBlsEngine::verify_partial(&shares[0].public_key_hex, message, &sig1));
        assert!(ThresholdBlsEngine::verify_partial(&shares[2].public_key_hex, message, &sig3));
        assert!(ThresholdBlsEngine::verify_partial(&shares[4].public_key_hex, message, &sig5));

        // Aggregate 3 of 5
        let partials = vec![sig1, sig3, sig5];
        let agg_sig = ThresholdBlsEngine::aggregate_signatures(
            threshold,
            &shares[0].group_public_key_hex,
            message,
            &partials,
        )
        .expect("Aggregation must succeed");

        assert_eq!(agg_sig.participant_ids, vec![1, 3, 5]);

        let is_valid = ThresholdBlsEngine::verify_aggregated(&agg_sig, message)
            .expect("Verification must not error");
        assert!(is_valid, "Aggregated threshold signature must verify against group pubkey");

        // Verify with different message must fail
        let tampered_msg = b"CRITICAL_AUTHORIZATION_TRANSFER_USD_9999999";
        let is_valid_tampered = ThresholdBlsEngine::verify_aggregated(&agg_sig, tampered_msg)
            .expect("Verification must not error");
        assert!(!is_valid_tampered, "Tampered message must fail verification");
    }

    #[test]
    fn test_threshold_signature_tampered_member_fails() {
        let seed = [0x7Eu8; 32];
        let threshold = 2;
        let total = 3;

        let shares = ThresholdBlsEngine::generate_key_shares(threshold, total, &seed)
            .expect("Key shares must generate");

        let message = b"SWIFT_ISO20022_SETTLEMENT_BATCH_901";

        let sig1 = ThresholdBlsEngine::sign_partial(&shares[0], message).unwrap();
        let mut sig2 = ThresholdBlsEngine::sign_partial(&shares[1], message).unwrap();

        // Corrupt signature 2
        sig2.signature_scalar_hex = hex::encode([0xFFu8; 32]);

        assert!(!ThresholdBlsEngine::verify_partial(&shares[1].public_key_hex, message, &sig2));

        let partials = vec![sig1, sig2];
        let agg_sig = ThresholdBlsEngine::aggregate_signatures(
            threshold,
            &shares[0].group_public_key_hex,
            message,
            &partials,
        )
        .expect("Aggregation step proceeds");

        let is_valid = ThresholdBlsEngine::verify_aggregated(&agg_sig, message)
            .expect("Verification check executes");
        assert!(!is_valid, "Aggregated signature with corrupted share must fail verification");
    }
}
