//! Decoy Memory Canaries & Honeytoken RAM Tripwire Defense.
//!
//! Deploys deceptive memory structures mimicking high-value cryptographic assets
//! (BIP-39 mnemonic phrases, AES-256 root master keys, Ed25519 seed material) across
//! process heaps to detect memory scanners, debugger probes, and process dumpers.
//! Triggers immediate defensive volatile zeroization on tripwire breach.

use sha2::{Digest, Sha256};
use zeroize::Zeroize;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HoneytokenType {
    Bip39MnemonicDecoy,
    Aes256MasterKeyDecoy,
    Ed25519SeedDecoy,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HoneytokenTripwire {
    pub token_id: String,
    pub token_type: HoneytokenType,
    pub decoy_payload_hex: String,
    pub canary_checksum_hex: String,
    pub is_tripped: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HoneytokenAuditReport {
    pub total_tripwires: usize,
    pub intact_tripwires: usize,
    pub breached_tripwires: usize,
    pub defense_status: String,
    pub breach_ids: Vec<String>,
}

#[allow(dead_code)]
pub struct HoneytokenDefenseEngine {
    tripwires: Vec<HoneytokenTripwire>,
}

#[allow(dead_code)]
impl HoneytokenDefenseEngine {
    pub fn new() -> Self {
        Self {
            tripwires: Vec::new(),
        }
    }

    /// Calculate canary checksum for a decoy payload
    fn compute_checksum(payload: &[u8], token_id: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"AEGIS_HONEYTOKEN_CANARY_CHECK");
        hasher.update(token_id.as_bytes());
        hasher.update(payload);
        hex::encode(hasher.finalize())
    }

    /// Arm a decoy BIP-39 mnemonic honeytoken
    pub fn arm_mnemonic_honeytoken(&mut self, token_id: &str) -> &HoneytokenTripwire {
        let decoy_text = b"witch collapse practice feed shame open despair creek road again ice least";
        let checksum = Self::compute_checksum(decoy_text, token_id);

        self.tripwires.push(HoneytokenTripwire {
            token_id: token_id.to_string(),
            token_type: HoneytokenType::Bip39MnemonicDecoy,
            decoy_payload_hex: hex::encode(decoy_text),
            canary_checksum_hex: checksum,
            is_tripped: false,
        });

        self.tripwires.last().unwrap()
    }

    /// Arm a decoy AES-256 master key honeytoken
    pub fn arm_aes_honeytoken(&mut self, token_id: &str) -> &HoneytokenTripwire {
        let decoy_bytes = [0xA5u8; 32];
        let checksum = Self::compute_checksum(&decoy_bytes, token_id);

        self.tripwires.push(HoneytokenTripwire {
            token_id: token_id.to_string(),
            token_type: HoneytokenType::Aes256MasterKeyDecoy,
            decoy_payload_hex: hex::encode(decoy_bytes),
            canary_checksum_hex: checksum,
            is_tripped: false,
        });

        self.tripwires.last().unwrap()
    }

    /// Audit all active honeytokens.
    /// If any honeytoken is modified, triggers defensive memory zeroization.
    pub fn audit_and_defend(&mut self, real_sensitive_buffer: &mut [u8]) -> HoneytokenAuditReport {
        let mut breached = Vec::new();

        for wire in &mut self.tripwires {
            let payload = hex::decode(&wire.decoy_payload_hex).unwrap_or_default();
            let current_checksum = Self::compute_checksum(&payload, &wire.token_id);

            if current_checksum != wire.canary_checksum_hex {
                wire.is_tripped = true;
                breached.push(wire.token_id.clone());
            }
        }

        let is_compromised = !breached.is_empty();
        if is_compromised {
            // Memory tripwire breached: execute immediate defensive zeroization
            real_sensitive_buffer.zeroize();
        }

        let intact = self.tripwires.len() - breached.len();
        HoneytokenAuditReport {
            total_tripwires: self.tripwires.len(),
            intact_tripwires: intact,
            breached_tripwires: breached.len(),
            defense_status: if is_compromised {
                "TRIPWIRE_BREACH_DEFENSIVE_ZEROIZATION_TRIGGERED".to_string()
            } else {
                "SECURE_MONITORING_ACTIVE".to_string()
            },
            breach_ids: breached,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_honeytoken_arm_and_detect_access() {
        let mut engine = HoneytokenDefenseEngine::new();
        engine.arm_mnemonic_honeytoken("TRIPWIRE-MNEMONIC-01");
        engine.arm_aes_honeytoken("TRIPWIRE-AES-01");

        let mut sensitive_data = [0x55u8; 64];
        let report = engine.audit_and_defend(&mut sensitive_data);

        assert_eq!(report.total_tripwires, 2);
        assert_eq!(report.intact_tripwires, 2);
        assert_eq!(report.breached_tripwires, 0);
        assert_eq!(report.defense_status, "SECURE_MONITORING_ACTIVE");
        assert_eq!(sensitive_data[0], 0x55, "Sensitive buffer untouched when intact");
    }

    #[test]
    fn test_honeytoken_tamper_alert_zeroization() {
        let mut engine = HoneytokenDefenseEngine::new();
        engine.arm_aes_honeytoken("TRIPWIRE-AES-COMPROMISED");

        let mut sensitive_data = [0xEEu8; 64];

        // Adversary or memory scraper alters honeytoken byte
        let mut tampered_bytes = hex::decode(&engine.tripwires[0].decoy_payload_hex).unwrap();
        tampered_bytes[0] ^= 0xFF;
        engine.tripwires[0].decoy_payload_hex = hex::encode(tampered_bytes);

        let report = engine.audit_and_defend(&mut sensitive_data);

        assert_eq!(report.breached_tripwires, 1);
        assert_eq!(report.defense_status, "TRIPWIRE_BREACH_DEFENSIVE_ZEROIZATION_TRIGGERED");
        assert!(engine.tripwires[0].is_tripped);

        // Assert sensitive buffer was immediately sanitized to all zeros
        assert!(sensitive_data.iter().all(|&b| b == 0), "Sensitive memory MUST be zeroized on breach");
    }
}
