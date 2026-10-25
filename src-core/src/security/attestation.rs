//! Unavailable integration: fails closed until a real provider is implemented.
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcrMeasurement {
    pub pcr_index: u32,
    pub description: String,
    pub digest_hex: String,
    pub baseline_digest: String,
    pub matches_baseline: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TpmAttestationProof {
    pub quote_id: String,
    pub pcr_digest: String,
    pub pcr_values: HashMap<u32, String>,
    pub nonce: String,
    pub ak_public_key: String,
    pub signature_hex: String,
    pub tpm_family: String,
    pub secure_boot_confirmed: bool,
    pub created_at: u64,
    pub validity_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationVerification {
    pub verified: bool,
    pub posture_status: String,
    pub secure_boot_active: bool,
    pub ima_hash_verified: bool,
    pub ak_identity: String,
    pub verification_token: String,
    pub audit_log: Vec<String>,
}

pub struct TpmAttestationEngine;
impl TpmAttestationEngine {
    pub fn new() -> Self {
        Self
    }
    pub fn get_measurements(&self) -> Vec<PcrMeasurement> {
        vec![]
    }
    pub fn generate_quote(
        &self,
        _nonce: &str,
        _pcrs: &[u32],
    ) -> Result<TpmAttestationProof, String> {
        Err("UNAVAILABLE: A real hardware provider is not configured; no simulated verification is permitted.".into())
    }
    pub fn verify_quote(
        &self,
        _proof: &TpmAttestationProof,
        _nonce: &str,
    ) -> Result<AttestationVerification, String> {
        Err("UNAVAILABLE: A real hardware provider is not configured; no simulated verification is permitted.".into())
    }
}
