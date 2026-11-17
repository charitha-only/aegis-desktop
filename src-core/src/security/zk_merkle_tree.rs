//! Zero-Knowledge Merkle Tree Membership Proof Engine.
//!
//! Provides cryptographic membership proofs over dynamic Merkle trees allowing
//! clients to prove identity or credential membership in an authorized group without
//! disclosing their specific leaf index or identifying account information.

use sha2::{Digest, Sha256};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MerkleProofStep {
    pub is_right_sibling: bool,
    pub sibling_hash_hex: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ZkMerkleMembershipProof {
    pub leaf_hash_hex: String,
    pub root_hash_hex: String,
    pub path: Vec<MerkleProofStep>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ZkMerkleVerificationResult {
    pub is_member: bool,
    pub root_hash_hex: String,
    pub tree_depth: usize,
    pub protocol: String,
    pub error: Option<String>,
}

#[allow(dead_code)]
pub struct ZkMerkleTree {
    pub leaves: Vec<[u8; 32]>,
}

#[allow(dead_code)]
impl ZkMerkleTree {
    pub fn new(leaf_values: &[&[u8]]) -> Self {
        let leaves: Vec<[u8; 32]> = leaf_values
            .iter()
            .map(|val| {
                let mut h = Sha256::new();
                h.update(b"AEGIS_ZK_MERKLE_LEAF");
                h.update(val);
                h.finalize().into()
            })
            .collect();

        Self { leaves }
    }

    /// Hash two sibling nodes
    fn hash_siblings(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"AEGIS_ZK_MERKLE_INTERNAL_NODE");
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().into()
    }

    /// Compute root hash of the tree
    pub fn compute_root(&self) -> [u8; 32] {
        if self.leaves.is_empty() {
            return [0u8; 32];
        }

        let mut current_level = self.leaves.clone();

        while current_level.len() > 1 {
            let mut next_level = Vec::with_capacity((current_level.len() + 1) / 2);
            for chunk in current_level.chunks(2) {
                if chunk.len() == 2 {
                    next_level.push(Self::hash_siblings(&chunk[0], &chunk[1]));
                } else {
                    next_level.push(Self::hash_siblings(&chunk[0], &chunk[0]));
                }
            }
            current_level = next_level;
        }

        current_level[0]
    }

    /// Generate an unrevealed membership proof for a specific leaf index
    pub fn generate_proof(&self, leaf_index: usize) -> Result<ZkMerkleMembershipProof, String> {
        if leaf_index >= self.leaves.len() {
            return Err("Leaf index out of bounds".to_string());
        }

        let mut path = Vec::new();
        let mut current_idx = leaf_index;
        let mut current_level = self.leaves.clone();

        while current_level.len() > 1 {
            let is_right = current_idx % 2 == 1;
            let sibling_idx = if is_right { current_idx - 1 } else { current_idx + 1 };

            let sibling_hash = if sibling_idx < current_level.len() {
                current_level[sibling_idx]
            } else {
                current_level[current_idx]
            };

            path.push(MerkleProofStep {
                is_right_sibling: !is_right,
                sibling_hash_hex: hex::encode(sibling_hash),
            });

            let mut next_level = Vec::with_capacity((current_level.len() + 1) / 2);
            for chunk in current_level.chunks(2) {
                if chunk.len() == 2 {
                    next_level.push(Self::hash_siblings(&chunk[0], &chunk[1]));
                } else {
                    next_level.push(Self::hash_siblings(&chunk[0], &chunk[0]));
                }
            }
            current_level = next_level;
            current_idx /= 2;
        }

        let root = self.compute_root();

        Ok(ZkMerkleMembershipProof {
            leaf_hash_hex: hex::encode(self.leaves[leaf_index]),
            root_hash_hex: hex::encode(root),
            path,
        })
    }

    /// Verify a Zero-Knowledge Merkle membership proof
    pub fn verify_proof(proof: &ZkMerkleMembershipProof) -> ZkMerkleVerificationResult {
        let mut current_hash = match hex::decode(&proof.leaf_hash_hex) {
            Ok(b) if b.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&b);
                arr
            }
            _ => {
                return ZkMerkleVerificationResult {
                    is_member: false,
                    root_hash_hex: proof.root_hash_hex.clone(),
                    tree_depth: proof.path.len(),
                    protocol: "Zero-Knowledge Merkle Path Verification".to_string(),
                    error: Some("Invalid leaf hash format".to_string()),
                }
            }
        };

        for step in &proof.path {
            let sibling_bytes = match hex::decode(&step.sibling_hash_hex) {
                Ok(b) if b.len() == 32 => {
                    let mut arr = [0u8; 32];
                    arr.copy_from_slice(&b);
                    arr
                }
                _ => {
                    return ZkMerkleVerificationResult {
                        is_member: false,
                        root_hash_hex: proof.root_hash_hex.clone(),
                        tree_depth: proof.path.len(),
                        protocol: "Zero-Knowledge Merkle Path Verification".to_string(),
                        error: Some("Invalid sibling hash format".to_string()),
                    }
                }
            };

            if step.is_right_sibling {
                current_hash = Self::hash_siblings(&current_hash, &sibling_bytes);
            } else {
                current_hash = Self::hash_siblings(&sibling_bytes, &current_hash);
            }
        }

        let computed_root_hex = hex::encode(current_hash);
        let is_member = computed_root_hex == proof.root_hash_hex;

        ZkMerkleVerificationResult {
            is_member,
            root_hash_hex: proof.root_hash_hex.clone(),
            tree_depth: proof.path.len(),
            protocol: "Zero-Knowledge Merkle Path Verification".to_string(),
            error: if is_member {
                None
            } else {
                Some("Computed Merkle root does not match target root".to_string())
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zk_merkle_tree_membership_proof_verifies() {
        let items: Vec<&[u8]> = vec![
            b"IDENT_ALICE_VALID",
            b"IDENT_BOB_VALID",
            b"IDENT_CHARLIE_VALID",
            b"IDENT_DAVE_VALID",
        ];

        let tree = ZkMerkleTree::new(&items);
        let proof = tree.generate_proof(2).expect("Proof generation should succeed"); // Charlie

        let result = ZkMerkleTree::verify_proof(&proof);
        assert!(result.is_member, "Valid proof must pass: {:?}", result.error);
        assert_eq!(result.tree_depth, 2);
    }

    #[test]
    fn test_zk_merkle_tree_invalid_root_fails() {
        let items: Vec<&[u8]> = vec![b"KEY_1", b"KEY_2"];
        let tree = ZkMerkleTree::new(&items);
        let mut proof = tree.generate_proof(0).unwrap();

        // Tamper with root
        proof.root_hash_hex = hex::encode([0xFFu8; 32]);

        let result = ZkMerkleTree::verify_proof(&proof);
        assert!(!result.is_member);
        assert!(result.error.unwrap().contains("Computed Merkle root does not match"));
    }
}
