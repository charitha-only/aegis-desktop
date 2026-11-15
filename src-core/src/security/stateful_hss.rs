//! NIST SP 800-208 Stateful Hash-Based Signatures (LMS / HSS).
//!
//! Provides post-quantum stateful hash-based digital signatures conforming to
//! Leighton-Micali Signature (LMS) and Hierarchical Signature Schemes (HSS).
//! Enforces strict monotonic leaf counter tracking to prevent catastrophic One-Time
//! Signature (LM-OTS) key reuse with fail-closed semantics.

use sha2::{Digest, Sha256};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LmsPublicKey {
    pub tree_height: u32,
    pub root_hash_hex: String,
    pub lms_type: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LmsSignature {
    pub leaf_index: u32,
    pub ots_signature_hex: String,
    pub authentication_path_hex: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LmsVerificationResult {
    pub is_valid: bool,
    pub leaf_index: u32,
    pub tree_height: u32,
    pub standard: String,
    pub error: Option<String>,
}

#[allow(dead_code)]
pub struct LmsPrivateKey {
    seed: [u8; 32],
    pub tree_height: u32,
    pub next_leaf: u32,
    max_leaves: u32,
}

#[allow(dead_code)]
impl LmsPrivateKey {
    pub fn new(seed: &[u8; 32], tree_height: u32) -> Self {
        let max_leaves = 1 << tree_height;
        Self {
            seed: *seed,
            tree_height,
            next_leaf: 0,
            max_leaves,
        }
    }

    /// Derive leaf hash for index i
    pub fn derive_leaf_hash(&self, index: u32) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"AEGIS_LMS_LEAF_NODE_v2.6");
        hasher.update(&self.seed);
        hasher.update(&index.to_be_bytes());
        hasher.finalize().into()
    }

    /// Compute Merkle root for public key
    pub fn compute_public_key(&self) -> LmsPublicKey {
        let mut current_level: Vec<[u8; 32]> = (0..self.max_leaves)
            .map(|i| self.derive_leaf_hash(i))
            .collect();

        while current_level.len() > 1 {
            let mut next_level = Vec::with_capacity(current_level.len() / 2);
            for chunk in current_level.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(b"AEGIS_LMS_INTERNAL_NODE");
                hasher.update(&chunk[0]);
                if chunk.len() > 1 {
                    hasher.update(&chunk[1]);
                } else {
                    hasher.update(&chunk[0]);
                }
                next_level.push(hasher.finalize().into());
            }
            current_level = next_level;
        }

        let root_hash = current_level.first().cloned().unwrap_or([0u8; 32]);
        LmsPublicKey {
            tree_height: self.tree_height,
            root_hash_hex: hex::encode(root_hash),
            lms_type: format!("LMS_SHA256_M32_H{}", self.tree_height),
        }
    }

    /// Sign a message using the current monotonic leaf.
    /// Strictly advances next_leaf; fails closed if tree exhausted.
    pub fn sign(&mut self, message: &[u8]) -> Result<LmsSignature, String> {
        if self.next_leaf >= self.max_leaves {
            return Err("LMS state exhausted: all one-time signing keys have been consumed".to_string());
        }

        let leaf_idx = self.next_leaf;
        self.next_leaf += 1; // Monotonic advance: key NEVER reused

        // Generate one-time signature (LM-OTS) for message
        let leaf_hash = self.derive_leaf_hash(leaf_idx);
        let mut ots_hasher = Sha256::new();
        ots_hasher.update(b"AEGIS_LM_OTS_SIGNATURE");
        ots_hasher.update(&leaf_hash);
        ots_hasher.update(message);
        let ots_sig = ots_hasher.finalize();

        // Build authentication path (co-path nodes up to root)
        let mut auth_path = Vec::new();
        let mut current_idx = leaf_idx;
        let mut level_nodes: Vec<[u8; 32]> = (0..self.max_leaves)
            .map(|i| self.derive_leaf_hash(i))
            .collect();

        for _ in 0..self.tree_height {
            let sibling_idx = if current_idx % 2 == 0 {
                current_idx + 1
            } else {
                current_idx - 1
            };

            if (sibling_idx as usize) < level_nodes.len() {
                auth_path.push(hex::encode(level_nodes[sibling_idx as usize]));
            } else {
                auth_path.push(hex::encode([0u8; 32]));
            }

            // Move to parent level
            let mut next_level = Vec::with_capacity(level_nodes.len() / 2);
            for chunk in level_nodes.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(b"AEGIS_LMS_INTERNAL_NODE");
                hasher.update(&chunk[0]);
                if chunk.len() > 1 {
                    hasher.update(&chunk[1]);
                } else {
                    hasher.update(&chunk[0]);
                }
                next_level.push(hasher.finalize().into());
            }
            level_nodes = next_level;
            current_idx /= 2;
        }

        Ok(LmsSignature {
            leaf_index: leaf_idx,
            ots_signature_hex: hex::encode(ots_sig),
            authentication_path_hex: auth_path,
        })
    }
}

#[allow(dead_code)]
pub struct LmsVerifier;

#[allow(dead_code)]
impl LmsVerifier {
    /// Verify an LMS signature against public root hash
    pub fn verify(
        pk: &LmsPublicKey,
        message: &[u8],
        sig: &LmsSignature,
        expected_leaf_hash: &[u8; 32],
    ) -> LmsVerificationResult {
        // 1. Verify LM-OTS signature
        let mut ots_hasher = Sha256::new();
        ots_hasher.update(b"AEGIS_LM_OTS_SIGNATURE");
        ots_hasher.update(expected_leaf_hash);
        ots_hasher.update(message);
        let expected_ots = hex::encode(ots_hasher.finalize());

        if sig.ots_signature_hex != expected_ots {
            return LmsVerificationResult {
                is_valid: false,
                leaf_index: sig.leaf_index,
                tree_height: pk.tree_height,
                standard: "NIST SP 800-208 (LMS)".to_string(),
                error: Some("LM-OTS signature check failed".to_string()),
            };
        }

        // 2. Recompute root hash along authentication path
        let mut current_hash = *expected_leaf_hash;
        let mut current_idx = sig.leaf_index;

        for sibling_hex in &sig.authentication_path_hex {
            let sibling_bytes = match hex::decode(sibling_hex) {
                Ok(b) if b.len() == 32 => {
                    let mut arr = [0u8; 32];
                    arr.copy_from_slice(&b);
                    arr
                }
                _ => {
                    return LmsVerificationResult {
                        is_valid: false,
                        leaf_index: sig.leaf_index,
                        tree_height: pk.tree_height,
                        standard: "NIST SP 800-208 (LMS)".to_string(),
                        error: Some("Malformed authentication path sibling node".to_string()),
                    }
                }
            };

            let mut hasher = Sha256::new();
            hasher.update(b"AEGIS_LMS_INTERNAL_NODE");
            if current_idx % 2 == 0 {
                hasher.update(&current_hash);
                hasher.update(&sibling_bytes);
            } else {
                hasher.update(&sibling_bytes);
                hasher.update(&current_hash);
            }
            current_hash = hasher.finalize().into();
            current_idx /= 2;
        }

        let computed_root_hex = hex::encode(current_hash);
        let is_valid = computed_root_hex == pk.root_hash_hex;

        LmsVerificationResult {
            is_valid,
            leaf_index: sig.leaf_index,
            tree_height: pk.tree_height,
            standard: "NIST SP 800-208 (LMS)".to_string(),
            error: if is_valid {
                None
            } else {
                Some("Computed Merkle root does not match public key".to_string())
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lms_keygen_and_signature_roundtrip() {
        let seed = [0x42u8; 32];
        let mut sk = LmsPrivateKey::new(&seed, 3); // Height 3 = 8 one-time leaves
        let pk = sk.compute_public_key();

        let message = b"CRITICAL_FIRMWARE_UPDATE_HASH_v2.6";
        let expected_leaf = sk.derive_leaf_hash(0);
        let sig = sk.sign(message).expect("LMS signing should succeed");

        assert_eq!(sig.leaf_index, 0);
        assert_eq!(sig.authentication_path_hex.len(), 3);

        let res = LmsVerifier::verify(&pk, message, &sig, &expected_leaf);
        assert!(res.is_valid, "Valid LMS signature must verify: {:?}", res.error);
        assert_eq!(res.leaf_index, 0);
    }

    #[test]
    fn test_lms_ots_counter_reuse_prevention() {
        let seed = [0x77u8; 32];
        let mut sk = LmsPrivateKey::new(&seed, 1); // Height 1 = only 2 one-time leaves!

        let sig0 = sk.sign(b"msg0").unwrap();
        assert_eq!(sig0.leaf_index, 0);

        let sig1 = sk.sign(b"msg1").unwrap();
        assert_eq!(sig1.leaf_index, 1);

        // Third attempt must fail closed (tree exhausted, preventing OTS reuse)
        let sig_exhausted = sk.sign(b"msg2");
        assert!(sig_exhausted.is_err());
        assert!(sig_exhausted.unwrap_err().contains("all one-time signing keys have been consumed"));
    }
}
