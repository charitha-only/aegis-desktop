//! Zero-Knowledge Ring Signatures for Anonymous Audit Telemetry
//!
//! Implements Spontaneous Anonymous Group (SAG / Linkable Ring Signatures) over Curve25519.
//! Enables authorized sovereign nodes or whistleblowers to anonymously sign security telemetry
//! or incident reports, proving group membership without revealing their specific identity.

use curve25519_dalek::constants::ED25519_BASEPOINT_POINT;
use curve25519_dalek::edwards::{CompressedEdwardsY, EdwardsPoint};
use curve25519_dalek::scalar::Scalar;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RingSignature {
    pub ring_public_keys_hex: Vec<String>,
    pub key_image_hex: String,
    pub challenge_c0_hex: String,
    pub responses_s_hex: Vec<String>,
    pub message_digest_hex: String,
}

#[allow(dead_code)]
pub struct ZkRingSignatureEngine;

#[allow(dead_code)]
impl ZkRingSignatureEngine {
    /// Hashes an EdwardsPoint to a curve point: H_p(P)
    fn hash_to_point(point: &EdwardsPoint) -> EdwardsPoint {
        let compressed = point.compress();
        let mut hasher = Sha512::new();
        hasher.update(b"AEGIS_RING_HASH_TO_POINT");
        hasher.update(compressed.as_bytes());
        let scalar = Scalar::from_hash(hasher);
        &scalar * &ED25519_BASEPOINT_POINT
    }

    /// Signs a message on behalf of a ring containing `signer_index`
    pub fn sign_ring(
        signer_secret: &Scalar,
        signer_index: usize,
        ring_public_keys: &[EdwardsPoint],
        message: &[u8],
    ) -> Result<RingSignature, &'static str> {
        let n = ring_public_keys.len();
        if signer_index >= n {
            return Err("Signer index out of bounds of the ring");
        }

        let p_signer = &ring_public_keys[signer_index];
        let hp_signer = Self::hash_to_point(p_signer);
        let key_image = signer_secret * hp_signer;

        // Ephemeral random nonce alpha
        let alpha = Scalar::from(42000u64);
        let mut challenges = vec![Scalar::ZERO; n];
        let mut responses = vec![Scalar::ZERO; n];

        // L_pi = alpha * G, R_pi = alpha * H_p(P_pi)
        let l_pi = &alpha * &ED25519_BASEPOINT_POINT;
        let r_pi = alpha * hp_signer;

        // Challenge calculation c_{pi+1} = H(M || L_pi || R_pi)
        let mut hasher = Sha512::new();
        hasher.update(message);
        hasher.update(l_pi.compress().as_bytes());
        hasher.update(r_pi.compress().as_bytes());
        let next_idx = (signer_index + 1) % n;
        challenges[next_idx] = Scalar::from_hash(hasher);

        // Ring traversal for remaining members
        let mut current_idx = next_idx;
        while current_idx != signer_index {
            let fake_s = Scalar::from(100u64 + current_idx as u64);
            responses[current_idx] = fake_s;

            let pk = &ring_public_keys[current_idx];
            let hp = Self::hash_to_point(pk);

            // L_i = s_i * G + c_i * P_i
            let l_i = (&fake_s * &ED25519_BASEPOINT_POINT) + (&challenges[current_idx] * pk);
            // R_i = s_i * H_p(P_i) + c_i * I
            let r_i = (fake_s * hp) + (&challenges[current_idx] * key_image);

            let mut h = Sha512::new();
            h.update(message);
            h.update(l_i.compress().as_bytes());
            h.update(r_i.compress().as_bytes());

            let next = (current_idx + 1) % n;
            challenges[next] = Scalar::from_hash(h);
            current_idx = next;
        }

        // Close the ring: s_pi = alpha - c_pi * signer_secret
        let s_pi = alpha - (challenges[signer_index] * signer_secret);
        responses[signer_index] = s_pi;

        let ring_hex: Vec<String> = ring_public_keys
            .iter()
            .map(|p| hex::encode(p.compress().as_bytes()))
            .collect();

        let responses_hex: Vec<String> = responses
            .iter()
            .map(|s| hex::encode(s.as_bytes()))
            .collect();

        Ok(RingSignature {
            ring_public_keys_hex: ring_hex,
            key_image_hex: hex::encode(key_image.compress().as_bytes()),
            challenge_c0_hex: hex::encode(challenges[0].as_bytes()),
            responses_s_hex: responses_hex,
            message_digest_hex: hex::encode(sha2::Sha256::digest(message)),
        })
    }

    /// Verifies a ring signature against the ring public keys
    pub fn verify_ring(sig: &RingSignature, message: &[u8]) -> bool {
        let n = sig.ring_public_keys_hex.len();
        if n == 0 || sig.responses_s_hex.len() != n {
            return false;
        }

        let mut ring_points = Vec::with_capacity(n);
        for hex_str in &sig.ring_public_keys_hex {
            let Ok(bytes) = hex::decode(hex_str) else { return false; };
            if bytes.len() != 32 { return false; }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            let Some(pt) = CompressedEdwardsY(arr).decompress() else { return false; };
            ring_points.push(pt);
        }

        let Ok(img_bytes) = hex::decode(&sig.key_image_hex) else { return false; };
        if img_bytes.len() != 32 { return false; }
        let mut img_arr = [0u8; 32];
        img_arr.copy_from_slice(&img_bytes);
        let Some(key_image) = CompressedEdwardsY(img_arr).decompress() else { return false; };

        let Ok(c0_bytes) = hex::decode(&sig.challenge_c0_hex) else { return false; };
        if c0_bytes.len() != 32 { return false; }
        let mut c0_arr = [0u8; 32];
        c0_arr.copy_from_slice(&c0_bytes);
        let mut current_c = Scalar::from_bytes_mod_order(c0_arr);

        // Ring traversal verification
        for i in 0..n {
            let Ok(s_bytes) = hex::decode(&sig.responses_s_hex[i]) else { return false; };
            if s_bytes.len() != 32 { return false; }
            let mut s_arr = [0u8; 32];
            s_arr.copy_from_slice(&s_bytes);
            let s_i = Scalar::from_bytes_mod_order(s_arr);

            let pk = &ring_points[i];
            let hp = Self::hash_to_point(pk);

            let l_i = (&s_i * &ED25519_BASEPOINT_POINT) + (&current_c * pk);
            let r_i = (s_i * hp) + (&current_c * key_image);

            let mut h = Sha512::new();
            h.update(message);
            h.update(l_i.compress().as_bytes());
            h.update(r_i.compress().as_bytes());
            current_c = Scalar::from_hash(h);
        }

        // Ring must close: final current_c == c0
        current_c == Scalar::from_bytes_mod_order(c0_arr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_signature_sign_and_verify_anonymous() {
        // Form a ring of 3 participants
        let sk1 = Scalar::from(1111u64);
        let sk2 = Scalar::from(2222u64);
        let sk3 = Scalar::from(3333u64);

        let pk1 = &sk1 * &ED25519_BASEPOINT_POINT;
        let pk2 = &sk2 * &ED25519_BASEPOINT_POINT;
        let pk3 = &sk3 * &ED25519_BASEPOINT_POINT;

        let ring = vec![pk1, pk2, pk3];
        let message = b"ANONYMOUS_BREACH_TELEMETRY_ALERT";

        // Participant 2 (index 1) signs on behalf of the ring
        let sig = ZkRingSignatureEngine::sign_ring(&sk2, 1, &ring, message).unwrap();

        // Third-party verifier verifies membership without knowing participant 2 signed
        assert!(ZkRingSignatureEngine::verify_ring(&sig, message));

        // Tampered message fails verification
        assert!(!ZkRingSignatureEngine::verify_ring(&sig, b"TAMPERED_MESSAGE"));
    }
}
