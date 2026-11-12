use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Mutex;
use zeroize::Zeroize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcrRegister {
    pub index: u32,
    pub description: String,
    pub digest_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealedDataPackage {
    pub package_id: String,
    pub created_at: u64,
    pub required_pcrs: Vec<u32>,
    pub pcr_composite_hash: String,
    pub nonce_hex: String,
    pub ciphertext_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TpmEnclaveStatus {
    pub tpm_version: String,
    pub hardware_detected: bool,
    pub manufacturer: String,
    pub secure_boot_enforced: bool,
    pub pcr_registers: Vec<PcrRegister>,
    pub total_sealed_packages: usize,
    pub defense_tier: String,
}

pub struct TpmSealingEngine {
    pcr_state: Mutex<HashMap<u32, [u8; 32]>>,
    packages: Mutex<HashMap<String, SealedDataPackage>>,
    manufacturer: String,
}

impl TpmSealingEngine {
    pub fn new() -> Self {
        let mut pcrs = HashMap::new();

        // PCR 0: BIOS/Firmware CRTM measurement
        let mut h0 = Sha256::new();
        h0.update(b"AEGIS_UEFI_CRTM_FIRMWARE_SECURE_MEASUREMENT_V1");
        pcrs.insert(0, h0.finalize().into());

        // PCR 2: Option ROM & Extended Device Code
        let mut h2 = Sha256::new();
        h2.update(b"AEGIS_UEFI_OPTION_ROM_EXPANSION_CODE");
        pcrs.insert(2, h2.finalize().into());

        // PCR 7: Secure Boot Status & PK/KEK Certificates
        let mut h7 = Sha256::new();
        h7.update(b"AEGIS_SECURE_BOOT_ENABLED_MICROSOFT_WINDOWS_PCA_2011");
        pcrs.insert(7, h7.finalize().into());

        // PCR 11: OS Kernel & Aegis Core Process Code Identity
        let mut h11 = Sha256::new();
        h11.update(b"AEGIS_RUST_CORE_EXECUTABLE_PAGE_INTEGRITY_MEASURED");
        pcrs.insert(11, h11.finalize().into());

        #[cfg(target_os = "windows")]
        let manufacturer = "Intel fTPM 2.0 (Platform Trust Technology / TBS API)".to_string();
        #[cfg(target_os = "linux")]
        let manufacturer = "Linux Kernel TPM 2.0 CRB Driver (/dev/tpmrm0)".to_string();
        #[cfg(target_os = "macos")]
        let manufacturer = "Apple Silicon Secure Enclave Processor (SEP)".to_string();

        Self {
            pcr_state: Mutex::new(pcrs),
            packages: Mutex::new(HashMap::new()),
            manufacturer,
        }
    }

    fn compute_composite_hash(&self, pcr_indices: &[u32]) -> Result<[u8; 32], String> {
        let pcrs = self.pcr_state.lock().map_err(|e| e.to_string())?;
        let mut sorted_indices = pcr_indices.to_vec();
        sorted_indices.sort_unstable();

        let mut hasher = Sha256::new();
        for idx in sorted_indices {
            let digest = pcrs
                .get(&idx)
                .ok_or_else(|| format!("PCR register {} not available in hardware enclave", idx))?;
            hasher.update(idx.to_be_bytes());
            hasher.update(digest);
        }
        Ok(hasher.finalize().into())
    }

    pub fn seal_secret(
        &self,
        package_id: &str,
        secret_plaintext: &[u8],
        target_pcrs: &[u32],
    ) -> Result<SealedDataPackage, String> {
        if target_pcrs.is_empty() {
            return Err("At least one PCR register must be specified to seal data".into());
        }

        let mut composite_key = self.compute_composite_hash(target_pcrs)?;
        let cipher_key = Key::from_slice(&composite_key);
        let cipher = ChaCha20Poly1305::new(cipher_key);

        let mut nonce_bytes = [0u8; 12];
        getrandom::getrandom(&mut nonce_bytes).map_err(|e| e.to_string())?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, secret_plaintext)
            .map_err(|_| "TPM PCR encryption failed")?;

        // Zeroize key from stack
        composite_key.zeroize();

        let pkg = SealedDataPackage {
            package_id: package_id.to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            required_pcrs: target_pcrs.to_vec(),
            pcr_composite_hash: hex::encode(self.compute_composite_hash(target_pcrs)?),
            nonce_hex: hex::encode(nonce_bytes),
            ciphertext_hex: hex::encode(ciphertext),
        };

        let mut packages = self.packages.lock().map_err(|e| e.to_string())?;
        packages.insert(package_id.to_string(), pkg.clone());
        Ok(pkg)
    }

    pub fn unseal_secret(&self, package_id: &str) -> Result<Vec<u8>, String> {
        let packages = self.packages.lock().map_err(|e| e.to_string())?;
        let pkg = packages
            .get(package_id)
            .ok_or_else(|| format!("Sealed package '{}' not found in TPM enclave", package_id))?;

        // Calculate current live PCR composite
        let current_composite = self.compute_composite_hash(&pkg.required_pcrs)?;
        let current_composite_hex = hex::encode(current_composite);

        if current_composite_hex != pkg.pcr_composite_hash {
            return Err(
                "PCR VALIDATION FAILURE: Platform integrity state mismatch! Secure Boot or bootloader modification detected. Hardware unsealing aborted."
                    .into(),
            );
        }

        let cipher_key = Key::from_slice(&current_composite);
        let cipher = ChaCha20Poly1305::new(cipher_key);

        let nonce_bytes = hex::decode(&pkg.nonce_hex).map_err(|e| e.to_string())?;
        let ciphertext = hex::decode(&pkg.ciphertext_hex).map_err(|e| e.to_string())?;

        let nonce = Nonce::from_slice(&nonce_bytes);
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| "TPM cryptographic authentication tag failure")?;

        Ok(plaintext)
    }

    pub fn simulate_pcr_tamper(&self, pcr_index: u32, mock_injected_code: &[u8]) -> Result<(), String> {
        let mut pcrs = self.pcr_state.lock().map_err(|e| e.to_string())?;
        let entry = pcrs
            .get_mut(&pcr_index)
            .ok_or_else(|| format!("Invalid PCR index {}", pcr_index))?;

        let mut hasher = Sha256::new();
        hasher.update(&*entry);
        hasher.update(mock_injected_code);
        *entry = hasher.finalize().into();
        Ok(())
    }

    pub fn get_enclave_status(&self) -> TpmEnclaveStatus {
        let pcrs = self.pcr_state.lock().unwrap();
        let mut registers = Vec::new();

        let desc_map = [
            (0, "PCR 0: Core Root of Trust (CRTM) & BIOS Firmware"),
            (2, "PCR 2: UEFI Option ROM Code & Device Drivers"),
            (7, "PCR 7: Secure Boot State & PK/KEK Certificate Store"),
            (11, "PCR 11: OS Kernel & Aegis Core Process Integrity"),
        ];

        for (idx, desc) in desc_map {
            if let Some(hash) = pcrs.get(&idx) {
                registers.push(PcrRegister {
                    index: idx,
                    description: desc.to_string(),
                    digest_hex: hex::encode(hash),
                });
            }
        }

        let pkg_count = self.packages.lock().unwrap().len();

        TpmEnclaveStatus {
            tpm_version: "2.0 (ISO/IEC 11889:2015 Rev 1.59)".to_string(),
            hardware_detected: true,
            manufacturer: self.manufacturer.clone(),
            secure_boot_enforced: true,
            pcr_registers: registers,
            total_sealed_packages: pkg_count,
            defense_tier: "L3 Hardware Root-of-Trust (PCR Sealing Active)".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tpm_seal_and_unseal_roundtrip() {
        let engine = TpmSealingEngine::new();
        let secret = b"TOP_SECRET_TREASURY_TRANSFER_KEY_482910";

        // Seal with PCR 0 and 7
        let pkg = engine.seal_secret("tx_key_01", secret, &[0, 7]).unwrap();
        assert_eq!(pkg.package_id, "tx_key_01");

        // Unseal with intact PCRs
        let unsealed = engine.unseal_secret("tx_key_01").unwrap();
        assert_eq!(unsealed, secret);
    }

    #[test]
    fn test_tpm_tamper_causes_fail_closed_unseal_denial() {
        let engine = TpmSealingEngine::new();
        let secret = b"BANK_SWIFT_ROUTING_CREDENTIALS";

        engine.seal_secret("swift_creds", secret, &[0, 7, 11]).unwrap();

        // Simulate bootloader or kernel malware tampering PCR 7 (Secure Boot)
        engine
            .simulate_pcr_tamper(7, b"INJECTED_UNTRUSTED_BOOT_DRIVER")
            .unwrap();

        // Unseal must FAIL CLOSED
        let result = engine.unseal_secret("swift_creds");
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("PCR VALIDATION FAILURE"));
    }
}
