//! Tamper-Resistant Cryptographic Black Box Flight Recorder
//!
//! Provides continuous, tamper-evident in-memory flight recording for security events,
//! capability grants, and process lifecycle events. Each record is cryptographically
//! hash-chained with SHA-256, creating an immutable audit trail that can be notarized.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::VecDeque;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightRecord {
    pub sequence: u64,
    pub timestamp: u64,
    pub event_category: String,
    pub detail: String,
    pub previous_hash: String,
    pub entry_hash: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackBoxNotarizationSeal {
    pub seal_sequence: u64,
    pub total_records: usize,
    pub root_hash: String,
    pub timestamp: u64,
    pub is_chain_valid: bool,
}

#[allow(dead_code)]
pub struct BlackBoxRecorderEngine {
    max_capacity: usize,
    sequence_counter: u64,
    records: VecDeque<FlightRecord>,
    last_hash: String,
}

#[allow(dead_code)]
impl BlackBoxRecorderEngine {
    pub fn new(max_capacity: usize) -> Self {
        let genesis_hash = hex::encode(Sha256::digest(b"AEGIS_BLACK_BOX_GENESIS_ROOT_v2.1"));
        Self {
            max_capacity: if max_capacity == 0 { 256 } else { max_capacity },
            sequence_counter: 0,
            records: VecDeque::new(),
            last_hash: genesis_hash,
        }
    }

    /// Records an event into the cryptographically chained flight recorder
    pub fn record_event(&mut self, category: &str, detail: &str, timestamp: u64) -> FlightRecord {
        self.sequence_counter += 1;

        let mut hasher = Sha256::new();
        hasher.update(&self.sequence_counter.to_le_bytes());
        hasher.update(&timestamp.to_le_bytes());
        hasher.update(category.as_bytes());
        hasher.update(detail.as_bytes());
        hasher.update(self.last_hash.as_bytes());
        let entry_hash = hex::encode(hasher.finalize());

        let record = FlightRecord {
            sequence: self.sequence_counter,
            timestamp,
            event_category: category.to_string(),
            detail: detail.to_string(),
            previous_hash: self.last_hash.clone(),
            entry_hash: entry_hash.clone(),
        };

        self.last_hash = entry_hash;

        if self.records.len() >= self.max_capacity {
            self.records.pop_front();
        }
        self.records.push_back(record.clone());

        record
    }

    /// Verifies the cryptographic integrity of the flight recorder chain
    pub fn verify_chain_integrity(&self) -> bool {
        let mut prev = hex::encode(Sha256::digest(b"AEGIS_BLACK_BOX_GENESIS_ROOT_v2.1"));
        for record in &self.records {
            if record.previous_hash != prev {
                // If trimmed due to capacity, allow first entry's previous_hash
                if record.sequence != self.records.front().map(|r| r.sequence).unwrap_or(0) {
                    return false;
                }
            }

            let mut hasher = Sha256::new();
            hasher.update(&record.sequence.to_le_bytes());
            hasher.update(&record.timestamp.to_le_bytes());
            hasher.update(record.event_category.as_bytes());
            hasher.update(record.detail.as_bytes());
            hasher.update(record.previous_hash.as_bytes());
            let computed = hex::encode(hasher.finalize());

            if computed != record.entry_hash {
                return false;
            }
            prev = record.entry_hash.clone();
        }
        true
    }

    /// Generates a notarization seal summarizing the current recorder state
    pub fn notarize(&self, timestamp: u64) -> BlackBoxNotarizationSeal {
        let is_valid = self.verify_chain_integrity();
        BlackBoxNotarizationSeal {
            seal_sequence: self.sequence_counter,
            total_records: self.records.len(),
            root_hash: self.last_hash.clone(),
            timestamp,
            is_chain_valid: is_valid,
        }
    }

    /// Returns the most recent flight records
    pub fn get_recent_records(&self, limit: usize) -> Vec<FlightRecord> {
        self.records.iter().rev().take(limit).cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blackbox_record_chaining_and_integrity() {
        let mut recorder = BlackBoxRecorderEngine::new(64);

        recorder.record_event("PROCESS_SPAWN", "Main window initialized", 1000);
        recorder.record_event("VAULT_UNLOCK", "Primary sovereign vault opened", 1001);
        recorder.record_event("SEAL_TRANSACTION", "Four-Eyes approval complete", 1002);

        assert_eq!(recorder.records.len(), 3);
        assert!(recorder.verify_chain_integrity());

        let seal = recorder.notarize(1003);
        assert!(seal.is_chain_valid);
        assert_eq!(seal.total_records, 3);
    }

    #[test]
    fn test_blackbox_detects_tampering() {
        let mut recorder = BlackBoxRecorderEngine::new(64);
        recorder.record_event("IPC_FRAME", "Authorized call", 1000);
        recorder.record_event("IPC_FRAME", "Second call", 1001);

        // Tamper with first record detail
        if let Some(first) = recorder.records.front_mut() {
            first.detail = "FORGED_TAMPERED_ENTRY".to_string();
        }

        assert!(!recorder.verify_chain_integrity());
    }
}
