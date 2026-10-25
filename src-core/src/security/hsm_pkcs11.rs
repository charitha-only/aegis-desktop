//! Unavailable integration: fails closed until a real provider is implemented.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HsmSlotInfo {
    pub slot_id: u64,
    pub token_present: bool,
    pub manufacturer_id: String,
    pub model: String,
    pub serial_number: String,
    pub hardware_version: String,
    pub firmware_version: String,
    pub label: String,
    pub is_initialized: bool,
    pub is_user_logged_in: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HsmKeyObject {
    pub key_id: String,
    pub key_type: String, // "ECC_NIST_P256", "RSA_4096", "ED25519"
    pub label: String,
    pub is_extractable: bool, // Always false in Zero-Trust HSM
    pub operations_allowed: Vec<String>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HsmSignatureResult {
    pub key_id: String,
    pub mechanism: String,
    pub payload_hash_hex: String,
    pub signature_hex: String,
    pub token_serial: String,
    pub timestamp: u64,
    pub verified_by_token: bool,
}

pub struct HsmPkcs11Engine;
impl HsmPkcs11Engine {
    pub fn new() -> Self {
        Self
    }
    pub fn list_slots(&self) -> Vec<HsmSlotInfo> {
        vec![]
    }
    pub fn login_token(&self, _id: u64, _pin: &str) -> Result<bool, String> {
        Err("UNAVAILABLE: A real hardware provider is not configured; no simulated verification is permitted.".into())
    }
    pub fn logout_token(&self, _id: u64) -> Result<bool, String> {
        Err("UNAVAILABLE: A real hardware provider is not configured; no simulated verification is permitted.".into())
    }
    pub fn list_keys(&self, _id: u64) -> Result<Vec<HsmKeyObject>, String> {
        Err("UNAVAILABLE: A real hardware provider is not configured; no simulated verification is permitted.".into())
    }
    pub fn sign_payload(
        &self,
        _id: u64,
        _key: &str,
        _payload: &str,
    ) -> Result<HsmSignatureResult, String> {
        Err("UNAVAILABLE: A real hardware provider is not configured; no simulated verification is permitted.".into())
    }
}
