//! Cryptographic Distance Bounding Protocol (Hancke-Kuhn / Brands-Chaum).
//!
//! Provides ultra-low latency challenge-response distance bounding for NFC, BLE,
//! and UWB physical hardware tokens to mathematically prevent radio-frequency
//! relay attacks, wormhole exploits, and man-in-the-middle proxying.

use sha2::{Digest, Sha512};

#[allow(dead_code)]
pub const DEFAULT_MAX_ROUNDTRIP_NANOS: u64 = 66; // ~10m proximity distance threshold
#[allow(dead_code)]
pub const EXCHANGE_ROUND_COUNT: usize = 32;

/// Distance Bounding Session Setup Data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[allow(dead_code)]
pub struct DistanceBoundingSession {
    pub session_id: String,
    pub verifier_nonce_hex: String,
    pub prover_nonce_hex: String,
    pub max_roundtrip_nanos: u64,
    pub round_count: usize,
}

/// Record of an individual rapid round exchange
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RoundExchangeRecord {
    pub round_index: usize,
    pub challenge_bit: u8,
    pub response_bit: u8,
    pub measured_rtt_nanos: u64,
}

/// Verification outcome of distance bounding protocol
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DistanceBoundingResult {
    pub is_verified: bool,
    pub is_relay_attack_detected: bool,
    pub avg_rtt_nanos: f64,
    pub max_rtt_nanos: u64,
    pub estimated_distance_meters: f64,
    pub error_count: usize,
    pub total_rounds: usize,
    pub failure_reason: Option<String>,
}

#[allow(dead_code)]
pub struct DistanceBoundingEngine;

#[allow(dead_code)]
impl DistanceBoundingEngine {
    /// Derive lookup registers R0 and R1 (each 32 bytes = 256 bits) from shared secret and nonces
    pub fn derive_registers(
        shared_key: &[u8; 32],
        verifier_nonce: &[u8; 32],
        prover_nonce: &[u8; 32],
    ) -> (Vec<u8>, Vec<u8>) {
        let mut hasher = Sha512::new();
        hasher.update(b"AEGIS_DISTANCE_BOUNDING_REGISTER_DERIVATION_v2.4");
        hasher.update(shared_key);
        hasher.update(verifier_nonce);
        hasher.update(prover_nonce);
        let digest = hasher.finalize();

        let r0 = digest[0..32].to_vec();
        let r1 = digest[32..64].to_vec();
        (r0, r1)
    }

    /// Extract i-th bit from byte slice
    pub fn get_bit(bytes: &[u8], index: usize) -> u8 {
        let byte_idx = index / 8;
        let bit_idx = index % 8;
        if byte_idx < bytes.len() {
            (bytes[byte_idx] >> (7 - bit_idx)) & 1
        } else {
            0
        }
    }

    /// Simulate or execute the rapid challenge-response exchange rounds
    pub fn execute_exchange_simulation(
        r0: &[u8],
        r1: &[u8],
        challenges: &[u8],
        simulated_latencies: &[u64],
    ) -> Vec<RoundExchangeRecord> {
        let rounds = challenges.len().min(r0.len() * 8);
        let mut records = Vec::with_capacity(rounds);

        for i in 0..rounds {
            let c_bit = challenges[i] & 1;
            let resp_bit = if c_bit == 0 {
                Self::get_bit(r0, i)
            } else {
                Self::get_bit(r1, i)
            };
            let latency = if i < simulated_latencies.len() {
                simulated_latencies[i]
            } else {
                35 // nominal nanoseconds
            };

            records.push(RoundExchangeRecord {
                round_index: i,
                challenge_bit: c_bit,
                response_bit: resp_bit,
                measured_rtt_nanos: latency,
            });
        }

        records
    }

    /// Verify distance bounding records against expected registers and timing thresholds
    pub fn verify(
        r0: &[u8],
        r1: &[u8],
        records: &[RoundExchangeRecord],
        max_rtt_nanos: u64,
        max_error_tolerance: usize,
    ) -> DistanceBoundingResult {
        if records.is_empty() {
            return DistanceBoundingResult {
                is_verified: false,
                is_relay_attack_detected: false,
                avg_rtt_nanos: 0.0,
                max_rtt_nanos: 0,
                estimated_distance_meters: 0.0,
                error_count: 0,
                total_rounds: 0,
                failure_reason: Some("No exchange records provided".to_string()),
            };
        }

        let mut error_count = 0;
        let mut max_observed_rtt = 0u64;
        let mut sum_rtt = 0u64;
        let mut relay_detected = false;

        for r in records {
            max_observed_rtt = max_observed_rtt.max(r.measured_rtt_nanos);
            sum_rtt += r.measured_rtt_nanos;

            // Check timing threshold
            if r.measured_rtt_nanos > max_rtt_nanos {
                relay_detected = true;
            }

            // Verify bit correctness
            let expected_bit = if r.challenge_bit == 0 {
                Self::get_bit(r0, r.round_index)
            } else {
                Self::get_bit(r1, r.round_index)
            };

            if r.response_bit != expected_bit {
                error_count += 1;
            }
        }

        let avg_rtt = sum_rtt as f64 / records.len() as f64;
        // Distance bound d = (c * t) / 2 where c ~ 0.3 meters per nanosecond
        let estimated_distance = (avg_rtt * 0.299792) / 2.0;

        let is_verified = !relay_detected && error_count <= max_error_tolerance;
        let failure_reason = if relay_detected {
            Some(format!(
                "Relay attack detected: max RTT {} ns exceeded limit of {} ns",
                max_observed_rtt, max_rtt_nanos
            ))
        } else if error_count > max_error_tolerance {
            Some(format!(
                "Bit error threshold exceeded: {} errors in {} rounds",
                error_count,
                records.len()
            ))
        } else {
            None
        };

        DistanceBoundingResult {
            is_verified,
            is_relay_attack_detected: relay_detected,
            avg_rtt_nanos: avg_rtt,
            max_rtt_nanos: max_observed_rtt,
            estimated_distance_meters: estimated_distance,
            error_count,
            total_rounds: records.len(),
            failure_reason,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_bounding_legitimate_proximity_verifies() {
        let key = [0x42u8; 32];
        let v_nonce = [0x11u8; 32];
        let p_nonce = [0x22u8; 32];

        let (r0, r1) = DistanceBoundingEngine::derive_registers(&key, &v_nonce, &p_nonce);
        let challenges = vec![0, 1, 1, 0, 1, 0, 0, 1, 1, 1, 0, 0, 1, 0, 1, 0];
        let latencies = vec![25, 28, 30, 22, 26, 29, 27, 24, 25, 29, 31, 26, 28, 25, 27, 26];

        let records = DistanceBoundingEngine::execute_exchange_simulation(&r0, &r1, &challenges, &latencies);
        let result = DistanceBoundingEngine::verify(&r0, &r1, &records, 66, 0);

        assert!(result.is_verified, "Legitimate proximity must verify");
        assert!(!result.is_relay_attack_detected);
        assert_eq!(result.error_count, 0);
        assert!(result.avg_rtt_nanos < 35.0);
        assert!(result.estimated_distance_meters < 5.0);
    }

    #[test]
    fn test_distance_bounding_relay_attack_timeout_fails() {
        let key = [0x88u8; 32];
        let v_nonce = [0x33u8; 32];
        let p_nonce = [0x44u8; 32];

        let (r0, r1) = DistanceBoundingEngine::derive_registers(&key, &v_nonce, &p_nonce);
        let challenges = vec![0, 1, 0, 1];
        // Adversary wormhole relay latency: 250 ns (~37 meters distance)
        let relay_latencies = vec![30, 250, 28, 32];

        let records = DistanceBoundingEngine::execute_exchange_simulation(&r0, &r1, &challenges, &relay_latencies);
        let result = DistanceBoundingEngine::verify(&r0, &r1, &records, 66, 0);

        assert!(!result.is_verified, "Relayed exchange must fail verification");
        assert!(result.is_relay_attack_detected);
        assert!(result.failure_reason.unwrap().contains("Relay attack detected"));
    }
}
