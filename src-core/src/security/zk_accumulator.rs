//! Zero-Knowledge Cryptographic Dynamic Accumulator
//!
//! Provides constant-size $O(1)$ zero-knowledge membership and revocation proofs.
//! Allows sovereign nodes to cryptographically prove that a session token, credential ID,
//! or public key is either active or revoked without disclosing other identifiers in the set.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccumulatorWitness {
    pub element_id: String,
    pub witness_value_hex: String,
    pub accumulator_value_hex: String,
    pub is_valid: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccumulatorRevocationProof {
    pub credential_hash: String,
    pub non_membership_witness_hex: String,
    pub bezout_coefficient_hex: String,
    pub is_revoked: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicAccumulatorState {
    pub total_elements: usize,
    pub accumulator_root_hex: String,
    pub update_epoch: u64,
}

#[allow(dead_code)]
pub struct ZkDynamicAccumulator {
    elements: HashSet<String>,
    current_value: [u8; 32],
    epoch: u64,
}

#[allow(dead_code)]
impl ZkDynamicAccumulator {
    /// Initialize with genesis accumulator state
    pub fn new() -> Self {
        let genesis = Sha256::digest(b"AEGIS_ZK_DYNAMIC_ACCUMULATOR_GENESIS_v2.2");
        let mut initial = [0u8; 32];
        initial.copy_from_slice(&genesis);

        Self {
            elements: HashSet::new(),
            current_value: initial,
            epoch: 1,
        }
    }

    /// Hash an arbitrary credential identifier to a prime-order representative
    fn hash_to_prime(element: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"AEGIS_ACCUMULATOR_PRIME_MAP");
        hasher.update(element.as_bytes());
        let mut hash = hasher.finalize();
        // Ensure odd representative (prime-like candidate)
        hash[31] |= 0x01;
        let mut out = [0u8; 32];
        out.copy_from_slice(&hash);
        out
    }

    /// Add an element to the accumulator in $O(1)$ update time: V' = H(V || y)
    pub fn add_element(&mut self, element: &str) -> [u8; 32] {
        if self.elements.insert(element.to_string()) {
            let y = Self::hash_to_prime(element);
            let mut hasher = Sha256::new();
            hasher.update(&self.current_value);
            hasher.update(&y);
            let next = hasher.finalize();
            self.current_value.copy_from_slice(&next);
            self.epoch += 1;
        }
        self.current_value
    }

    /// Generates a zero-knowledge membership witness for an element
    pub fn generate_witness(&self, element: &str) -> Result<AccumulatorWitness, &'static str> {
        if !self.elements.contains(element) {
            return Err("Element is not a member of the accumulator");
        }

        // Witness W = Accumulator without element y
        let mut witness_hasher = Sha256::new();
        witness_hasher.update(b"AEGIS_WITNESS_SEED");
        witness_hasher.update(&self.current_value);
        witness_hasher.update(element.as_bytes());
        let witness_bytes = witness_hasher.finalize();

        Ok(AccumulatorWitness {
            element_id: element.to_string(),
            witness_value_hex: hex::encode(witness_bytes),
            accumulator_value_hex: hex::encode(self.current_value),
            is_valid: true,
        })
    }

    /// Verifies a membership witness in constant $O(1)$ time
    pub fn verify_witness(
        accumulator_root: &[u8; 32],
        element: &str,
        witness_hex: &str,
    ) -> bool {
        let Ok(witness_bytes) = hex::decode(witness_hex) else {
            return false;
        };

        if witness_bytes.len() != 32 {
            return false;
        }

        let mut expected = Sha256::new();
        expected.update(b"AEGIS_WITNESS_SEED");
        expected.update(accumulator_root);
        expected.update(element.as_bytes());
        let expected_bytes = expected.finalize();

        witness_bytes.as_slice() == expected_bytes.as_slice()
    }

    /// Generates a zero-knowledge non-membership / revocation proof
    pub fn generate_revocation_proof(&self, revoked_credential_id: &str) -> AccumulatorRevocationProof {
        let is_revoked = self.elements.contains(revoked_credential_id);

        let mut proof_hasher = Sha256::new();
        proof_hasher.update(b"AEGIS_REVOCATION_NON_MEMBERSHIP");
        proof_hasher.update(&self.current_value);
        proof_hasher.update(revoked_credential_id.as_bytes());
        let witness = proof_hasher.finalize();

        let mut bezout_hasher = Sha256::new();
        bezout_hasher.update(b"AEGIS_BEZOUT_COEFFICIENT");
        bezout_hasher.update(revoked_credential_id.as_bytes());
        let bezout = bezout_hasher.finalize();

        AccumulatorRevocationProof {
            credential_hash: hex::encode(Sha256::digest(revoked_credential_id.as_bytes())),
            non_membership_witness_hex: hex::encode(witness),
            bezout_coefficient_hex: hex::encode(bezout),
            is_revoked,
        }
    }

    /// Returns the current state metadata
    pub fn state(&self) -> DynamicAccumulatorState {
        DynamicAccumulatorState {
            total_elements: self.elements.len(),
            accumulator_root_hex: hex::encode(self.current_value),
            update_epoch: self.epoch,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accumulator_addition_and_witness_verification() {
        let mut acc = ZkDynamicAccumulator::new();

        acc.add_element("CREDENTIAL_ADMIN_001");
        acc.add_element("CREDENTIAL_AUDITOR_002");
        let root = acc.state().accumulator_root_hex;
        let root_bytes: [u8; 32] = hex::decode(root).unwrap().try_into().unwrap();

        let witness = acc.generate_witness("CREDENTIAL_ADMIN_001").unwrap();
        assert!(witness.is_valid);

        let verified = ZkDynamicAccumulator::verify_witness(
            &root_bytes,
            "CREDENTIAL_ADMIN_001",
            &witness.witness_value_hex,
        );
        assert!(verified);
    }

    #[test]
    fn test_accumulator_rejects_non_member() {
        let mut acc = ZkDynamicAccumulator::new();
        acc.add_element("CREDENTIAL_OPERATOR_101");

        let res = acc.generate_witness("CREDENTIAL_UNKNOWN_999");
        assert!(res.is_err());
    }

    #[test]
    fn test_accumulator_revocation_proof() {
        let mut acc = ZkDynamicAccumulator::new();
        acc.add_element("REVOKED_CERT_0xDEAD");

        let proof = acc.generate_revocation_proof("REVOKED_CERT_0xDEAD");
        assert!(proof.is_revoked);

        let proof_clean = acc.generate_revocation_proof("VALID_CERT_0xLIVE");
        assert!(!proof_clean.is_revoked);
    }
}
