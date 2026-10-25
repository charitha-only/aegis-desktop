//! Unavailable integration: fails closed until a real provider is implemented.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnclaveStatus {
    pub platform: String,
    pub enclave_type: String,
    pub biometric_enrolled: bool,
    pub tpm_version: String,
    pub secure_boot_active: bool,
    pub master_key_bound: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiometricAuthResult {
    pub authenticated: bool,
    pub method: String,
    pub timestamp: u64,
    pub assertion_token: String,
}

pub struct BiometricEnclave;
impl BiometricEnclave {
    pub fn new() -> Self {
        Self
    }
    pub fn get_status(&self) -> EnclaveStatus {
        EnclaveStatus {
            platform: std::env::consts::OS.into(),
            enclave_type: "UNAVAILABLE: no native biometric provider".into(),
            biometric_enrolled: false,
            tpm_version: String::new(),
            secure_boot_active: false,
            master_key_bound: false,
        }
    }
    pub fn authenticate(&self, _reason: &str) -> Result<BiometricAuthResult, String> {
        Err("UNAVAILABLE: A real hardware provider is not configured; no simulated verification is permitted.".into())
    }
}
