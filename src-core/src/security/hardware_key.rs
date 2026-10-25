//! Unavailable integration: fails closed until a real provider is implemented.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareKeyInfo {
    pub key_id: String,
    pub manufacturer: String,
    pub model: String,
    pub protocol: String,
    pub enrolled_at: u64,
    pub touch_required: bool,
    pub pin_verified: bool,
    /// SHA-256 hex of the relying-party ID committed to this key during enrollment
    pub rp_id_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeRequest {
    pub challenge_id: String,
    pub relying_party: String,
    pub challenge_nonce: String,
    pub expires_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResponse {
    pub challenge_id: String,
    pub key_id: String,
    /// The clientDataJSON the authenticator signed (must be provided by caller)
    pub client_data_json: String,
    /// Raw authenticatorData hex (rpIdHash || flags || counter || ...)
    pub authenticator_data: String,
    /// The assertion signature hex provided by the hardware device
    pub signature_hex: String,
    /// True only when full assertion verification passed
    pub verified: bool,
    /// Human-readable verification steps log
    pub verification_log: Vec<String>,
}

pub struct HardwareKeyEngine;
impl HardwareKeyEngine {
    pub fn new() -> Self {
        Self
    }
    pub fn scan_and_detect_keys(&self) -> Vec<HardwareKeyInfo> {
        vec![]
    }
    pub fn list_keys(&self) -> Vec<HardwareKeyInfo> {
        vec![]
    }
    pub fn enroll_key(&self, _id: &str, _pin: &str) -> Result<HardwareKeyInfo, String> {
        Err("UNAVAILABLE: A real hardware provider is not configured; no simulated verification is permitted.".into())
    }
    pub fn set_pin(&self, _old: &str, _new: &str) -> Result<(), String> {
        Err("UNAVAILABLE: A real hardware provider is not configured; no simulated verification is permitted.".into())
    }
    pub fn generate_challenge(&self, _rp: &str) -> Result<ChallengeRequest, String> {
        Err("UNAVAILABLE: A real hardware provider is not configured; no simulated verification is permitted.".into())
    }
    pub fn verify_signature(
        &self,
        _id: &str,
        _pin: &str,
        _cd: &str,
        _ad: &str,
        _sig: &str,
    ) -> Result<ChallengeResponse, String> {
        Err("UNAVAILABLE: A real hardware provider is not configured; no simulated verification is permitted.".into())
    }
}
