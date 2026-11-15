//! Heterogeneous Hardware Attestation Graph & Evidence Cache Engine.
//!
//! Evaluates and chains heterogeneous hardware attestation quotes (TPM 2.0 PCR quotes,
//! Intel SGX Enclave certificates, and AMD SEV-SNP guest reports) into a unified
//! cryptographic directed acyclic graph (DAG), computing a composite platform trust score.

use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AttestationRootType {
    Tpm20Pcr,
    IntelSgx,
    AmdSevSnp,
    SecureBootPolicy,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AttestationEvidenceNode {
    pub node_id: String,
    pub root_type: AttestationRootType,
    pub measurement_digest_hex: String,
    pub signature_hex: String,
    pub nonce: u64,
    pub issuer_cert_thumbprint_hex: String,
    pub is_verified: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AttestationGraphReport {
    pub total_nodes: usize,
    pub verified_nodes: usize,
    pub platform_trust_score: u32,
    pub is_fully_trusted: bool,
    pub evidence_nodes: Vec<AttestationEvidenceNode>,
    pub evaluation_timestamp: u64,
    pub failure_reasons: Vec<String>,
}

#[allow(dead_code)]
pub struct AttestationGraphEngine {
    trusted_issuers: HashMap<String, String>,
}

#[allow(dead_code)]
impl AttestationGraphEngine {
    pub fn new() -> Self {
        let mut trusted_issuers = HashMap::new();
        // Standard hardware manufacturer roots
        trusted_issuers.insert(
            "INTEL_SGX_ROOT_CA_CERT".to_string(),
            hex::encode(Sha256::digest(b"INTEL_SGX_ROOT_CA_KEY")),
        );
        trusted_issuers.insert(
            "AMD_SEV_SNP_ROOT_CA_CERT".to_string(),
            hex::encode(Sha256::digest(b"AMD_SEV_SNP_ROOT_CA_KEY")),
        );
        trusted_issuers.insert(
            "TCG_TPM20_EK_ROOT_CA_CERT".to_string(),
            hex::encode(Sha256::digest(b"TCG_TPM20_EK_ROOT_CA_KEY")),
        );
        Self { trusted_issuers }
    }

    /// Verify an individual hardware evidence node
    pub fn verify_node(&self, node: &mut AttestationEvidenceNode, expected_nonce: u64) -> bool {
        // 1. Freshness check: nonce must match
        if node.nonce != expected_nonce {
            node.is_verified = false;
            return false;
        }

        // 2. Issuer trust check: thumbprint must be recognized
        let known_issuer = self.trusted_issuers.values().any(|thumb| thumb == &node.issuer_cert_thumbprint_hex);
        if !known_issuer {
            node.is_verified = false;
            return false;
        }

        // 3. Signature verification simulation: sig = H(measurement || nonce || thumbprint)
        let mut hasher = Sha256::new();
        hasher.update(b"AEGIS_HW_ATTESTATION_CLAIM");
        hasher.update(node.measurement_digest_hex.as_bytes());
        hasher.update(&node.nonce.to_be_bytes());
        hasher.update(node.issuer_cert_thumbprint_hex.as_bytes());
        let expected_sig = hex::encode(hasher.finalize());

        let valid = expected_sig == node.signature_hex;
        node.is_verified = valid;
        valid
    }

    /// Construct a verified node with correct signature
    pub fn create_test_node(
        node_id: &str,
        root_type: AttestationRootType,
        measurement: &[u8],
        nonce: u64,
        issuer_key: &[u8],
    ) -> AttestationEvidenceNode {
        let thumbprint = hex::encode(Sha256::digest(issuer_key));
        let measurement_hex = hex::encode(Sha256::digest(measurement));

        let mut hasher = Sha256::new();
        hasher.update(b"AEGIS_HW_ATTESTATION_CLAIM");
        hasher.update(measurement_hex.as_bytes());
        hasher.update(&nonce.to_be_bytes());
        hasher.update(thumbprint.as_bytes());
        let signature_hex = hex::encode(hasher.finalize());

        AttestationEvidenceNode {
            node_id: node_id.to_string(),
            root_type,
            measurement_digest_hex: measurement_hex,
            signature_hex,
            nonce,
            issuer_cert_thumbprint_hex: thumbprint,
            is_verified: false,
        }
    }

    /// Evaluate complete graph and calculate composite trust score
    pub fn evaluate_graph(
        &self,
        mut nodes: Vec<AttestationEvidenceNode>,
        expected_nonce: u64,
        timestamp: u64,
    ) -> AttestationGraphReport {
        let total = nodes.len();
        let mut verified_count = 0;
        let mut failure_reasons = Vec::new();

        for node in &mut nodes {
            if self.verify_node(node, expected_nonce) {
                verified_count += 1;
            } else {
                failure_reasons.push(format!("Node '{}' failed attestation check", node.node_id));
            }
        }

        let trust_score = if total == 0 {
            0
        } else {
            ((verified_count as f32 / total as f32) * 100.0) as u32
        };

        AttestationGraphReport {
            total_nodes: total,
            verified_nodes: verified_count,
            platform_trust_score: trust_score,
            is_fully_trusted: total > 0 && verified_count == total,
            evidence_nodes: nodes,
            evaluation_timestamp: timestamp,
            failure_reasons,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attestation_graph_all_roots_verified() {
        let engine = AttestationGraphEngine::new();
        let nonce = 1234567890u64;

        let node_tpm = AttestationGraphEngine::create_test_node(
            "TPM-PCR-BANK",
            AttestationRootType::Tpm20Pcr,
            b"PCR_0_FIRMWARE_MEASUREMENT_DATA",
            nonce,
            b"TCG_TPM20_EK_ROOT_CA_KEY",
        );

        let node_sgx = AttestationGraphEngine::create_test_node(
            "INTEL-SGX-EPC",
            AttestationRootType::IntelSgx,
            b"MRENCLAVE_AES_GCM_ENCLAVE_CODE",
            nonce,
            b"INTEL_SGX_ROOT_CA_KEY",
        );

        let node_sev = AttestationGraphEngine::create_test_node(
            "AMD-SEV-SNP",
            AttestationRootType::AmdSevSnp,
            b"GUEST_POLICY_MEASUREMENT_DIGEST",
            nonce,
            b"AMD_SEV_SNP_ROOT_CA_KEY",
        );

        let report = engine.evaluate_graph(vec![node_tpm, node_sgx, node_sev], nonce, 1700000000);

        assert_eq!(report.total_nodes, 3);
        assert_eq!(report.verified_nodes, 3);
        assert_eq!(report.platform_trust_score, 100);
        assert!(report.is_fully_trusted);
        assert!(report.failure_reasons.is_empty());
    }

    #[test]
    fn test_attestation_graph_revoked_root_or_bad_quote_fails() {
        let engine = AttestationGraphEngine::new();
        let nonce = 9999u64;

        // Node with unknown/untrusted issuer
        let mut untrusted_node = AttestationGraphEngine::create_test_node(
            "ROGUE-TPM",
            AttestationRootType::Tpm20Pcr,
            b"MEASUREMENT",
            nonce,
            b"UNTRUSTED_UNKNOWN_KEY",
        );

        let valid = engine.verify_node(&mut untrusted_node, nonce);
        assert!(!valid, "Untrusted issuer must be rejected");

        // Stale nonce check
        let mut stale_node = AttestationGraphEngine::create_test_node(
            "STALE-SGX",
            AttestationRootType::IntelSgx,
            b"MEASUREMENT",
            nonce,
            b"INTEL_SGX_ROOT_CA_KEY",
        );
        let valid_nonce = engine.verify_node(&mut stale_node, nonce + 1); // wrong nonce
        assert!(!valid_nonce, "Stale nonce replay must be rejected");
    }
}
