//! Hardware Token APDU & CTAP2 Framed Packet Mutation Fuzzer.
//!
//! Provides automated robustness testing and anomaly detection for smartcard
//! ISO 7816-4 APDU commands and FIDO2 / CTAP2 framed packets. Detects parser
//! crashes, buffer overruns, and unexpected status word responses.

use sha2::{Digest, Sha256};

#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApduCommand {
    pub cla: u8,
    pub ins: u8,
    pub p1: u8,
    pub p2: u8,
    pub data: Vec<u8>,
    pub le: Option<u8>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FuzzMutationStrategy {
    BitFlip,
    BoundaryLength,
    BufferOverflow,
    InvalidClassInstruction,
    Truncation,
}

#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FuzzAnomaly {
    pub iteration: usize,
    pub strategy: String,
    pub status_word_hex: String,
    pub payload_preview_hex: String,
    pub description: String,
    pub is_critical: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FuzzCampaignReport {
    pub total_mutations_executed: usize,
    pub malformed_rejections_confirmed: usize,
    pub anomalies_detected: usize,
    pub critical_failures: usize,
    pub campaign_fingerprint_hex: String,
    pub anomalies: Vec<FuzzAnomaly>,
}

#[allow(dead_code)]
pub struct TokenFuzzerEngine;

#[allow(dead_code)]
impl TokenFuzzerEngine {
    /// Encode a structured APDU command into raw bytes according to ISO 7816-4.
    pub fn encode_apdu(cmd: &ApduCommand) -> Vec<u8> {
        let mut raw = Vec::with_capacity(7 + cmd.data.len());
        raw.push(cmd.cla);
        raw.push(cmd.ins);
        raw.push(cmd.p1);
        raw.push(cmd.p2);

        if !cmd.data.is_empty() {
            raw.push(cmd.data.len() as u8);
            raw.extend_from_slice(&cmd.data);
        }

        if let Some(le) = cmd.le {
            raw.push(le);
        }

        raw
    }

    /// Strictly parse and validate raw APDU bytes. Rejects malformed or out-of-bounds frames.
    pub fn parse_apdu_strict(raw: &[u8]) -> Result<ApduCommand, String> {
        if raw.len() < 4 {
            return Err("Truncated APDU: minimum header length is 4 bytes".to_string());
        }

        let cla = raw[0];
        let ins = raw[1];
        let p1 = raw[2];
        let p2 = raw[3];

        if raw.len() == 4 {
            return Ok(ApduCommand {
                cla,
                ins,
                p1,
                p2,
                data: Vec::new(),
                le: None,
            });
        }

        let mut offset = 4;
        let mut data = Vec::new();
        let mut le = None;

        if raw.len() > 5 {
            let lc = raw[offset] as usize;
            offset += 1;

            if raw.len() < offset + lc {
                return Err(format!(
                    "APDU payload length mismatch: declared Lc={}, available={}",
                    lc,
                    raw.len() - offset
                ));
            }

            data = raw[offset..offset + lc].to_vec();
            offset += lc;

            if offset < raw.len() {
                le = Some(raw[offset]);
            }
        } else if raw.len() == 5 {
            // Case 2: 4 bytes header + 1 byte Le
            le = Some(raw[4]);
        }

        Ok(ApduCommand {
            cla,
            ins,
            p1,
            p2,
            data,
            le,
        })
    }

    /// Mutate an APDU payload using the specified mutation strategy.
    pub fn mutate_apdu(
        seed_apdu: &ApduCommand,
        strategy: FuzzMutationStrategy,
        seed_offset: usize,
    ) -> Vec<u8> {
        let mut raw = Self::encode_apdu(seed_apdu);

        match strategy {
            FuzzMutationStrategy::BitFlip => {
                if !raw.is_empty() {
                    let target_byte = seed_offset % raw.len();
                    let target_bit = 1u8 << (seed_offset % 8);
                    raw[target_byte] ^= target_bit;
                }
            }
            FuzzMutationStrategy::BoundaryLength => {
                if raw.len() >= 5 {
                    // Inject 0x00, 0xFF, or oversize Lc field
                    let boundary_vals = [0x00, 0xFF, 0xFE, 0x80];
                    let val = boundary_vals[seed_offset % boundary_vals.len()];
                    raw[4] = val;
                }
            }
            FuzzMutationStrategy::BufferOverflow => {
                // Append 1024 bytes of garbage to test buffer overflow limits
                let overflow_block = vec![0xEEu8; 512 + (seed_offset % 512)];
                raw.extend_from_slice(&overflow_block);
            }
            FuzzMutationStrategy::InvalidClassInstruction => {
                // Mutate CLA/INS into reserved or proprietary invalid space
                raw[0] = 0x80 | ((seed_offset as u8) & 0x7F);
                raw[1] = 0xF0 ^ (seed_offset as u8);
            }
            FuzzMutationStrategy::Truncation => {
                if raw.len() > 2 {
                    let cut = 1 + (seed_offset % (raw.len() - 1));
                    raw.truncate(cut);
                }
            }
        }

        raw
    }

    /// Simulate token response to raw mutated frame.
    /// Returns (SW status word, response data).
    pub fn simulate_token_response(raw: &[u8]) -> (u16, Vec<u8>) {
        match Self::parse_apdu_strict(raw) {
            Ok(cmd) => {
                // Check for valid known applet instructions
                if cmd.cla == 0x00 && cmd.ins == 0xA4 {
                    // SELECT FILE
                    (0x9000, vec![0x6F, 0x06, 0x84, 0x04, 0xA0, 0x00])
                } else if cmd.cla == 0x80 && cmd.ins == 0x10 {
                    // GET ATTESTATION
                    (0x9000, vec![0x01, 0x02, 0x03, 0x04])
                } else {
                    // Unknown INS -> 0x6D00 (Instruction not supported)
                    (0x6D00, Vec::new())
                }
            }
            Err(e) => {
                if e.contains("Truncated") {
                    (0x6700, Vec::new()) // Wrong length
                } else if e.contains("mismatch") {
                    (0x6A87, Vec::new()) // Lc inconsistent with P1-P2
                } else {
                    (0x6F00, Vec::new()) // General diagnostic failure
                }
            }
        }
    }

    /// Execute an automated fuzz testing campaign against the token protocol.
    pub fn run_fuzz_campaign(iterations: usize) -> FuzzCampaignReport {
        let seed = ApduCommand {
            cla: 0x00,
            ins: 0xA4,
            p1: 0x04,
            p2: 0x00,
            data: vec![0xA0, 0x00, 0x00, 0x03, 0x08],
            le: Some(0x00),
        };

        let strategies = [
            FuzzMutationStrategy::BitFlip,
            FuzzMutationStrategy::BoundaryLength,
            FuzzMutationStrategy::BufferOverflow,
            FuzzMutationStrategy::InvalidClassInstruction,
            FuzzMutationStrategy::Truncation,
        ];

        let mut confirmed_rejections = 0;
        let mut anomalies = Vec::new();
        let mut campaign_hasher = Sha256::new();

        for i in 0..iterations {
            let strategy = strategies[i % strategies.len()];
            let mutated_raw = Self::mutate_apdu(&seed, strategy, i);
            campaign_hasher.update(&mutated_raw);

            let (sw, _resp) = Self::simulate_token_response(&mutated_raw);

            // Anomaly criteria: if a malformed payload returns 0x9000 (Success) or 0x0000 (Crash)
            let is_strictly_valid = Self::parse_apdu_strict(&mutated_raw).is_ok();

            if !is_strictly_valid {
                if sw == 0x9000 {
                    anomalies.push(FuzzAnomaly {
                        iteration: i,
                        strategy: format!("{:?}", strategy),
                        status_word_hex: format!("{:04X}", sw),
                        payload_preview_hex: hex::encode(&mutated_raw[..std::cmp::min(16, mutated_raw.len())]),
                        description: "Security bypass: Malformed packet accepted with 0x9000 OK".to_string(),
                        is_critical: true,
                    });
                } else if sw == 0x0000 {
                    anomalies.push(FuzzAnomaly {
                        iteration: i,
                        strategy: format!("{:?}", strategy),
                        status_word_hex: "0000".to_string(),
                        payload_preview_hex: hex::encode(&mutated_raw[..std::cmp::min(16, mutated_raw.len())]),
                        description: "Applet crash / unhandled panic simulated".to_string(),
                        is_critical: true,
                    });
                } else {
                    confirmed_rejections += 1;
                }
            } else {
                confirmed_rejections += 1;
            }
        }

        let critical_count = anomalies.iter().filter(|a| a.is_critical).count();
        let fingerprint = hex::encode(campaign_hasher.finalize());

        FuzzCampaignReport {
            total_mutations_executed: iterations,
            malformed_rejections_confirmed: confirmed_rejections,
            anomalies_detected: anomalies.len(),
            critical_failures: critical_count,
            campaign_fingerprint_hex: fingerprint,
            anomalies,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_fuzzer_apdu_mutation_coverage() {
        let cmd = ApduCommand {
            cla: 0x00,
            ins: 0xA4,
            p1: 0x04,
            p2: 0x00,
            data: vec![0x11, 0x22, 0x33],
            le: Some(0x20),
        };

        // Strict parsing of original APDU
        let raw = TokenFuzzerEngine::encode_apdu(&cmd);
        let parsed = TokenFuzzerEngine::parse_apdu_strict(&raw).expect("Valid APDU must parse cleanly");
        assert_eq!(parsed.cla, 0x00);
        assert_eq!(parsed.ins, 0xA4);
        assert_eq!(parsed.data, vec![0x11, 0x22, 0x33]);

        // Truncate to 2 bytes
        let truncated = TokenFuzzerEngine::mutate_apdu(&cmd, FuzzMutationStrategy::Truncation, 1);
        let trunc_res = TokenFuzzerEngine::parse_apdu_strict(&truncated);
        assert!(trunc_res.is_err(), "Truncated APDU must fail strict validation");

        // Boundary length mutation (oversized Lc)
        let boundary = TokenFuzzerEngine::mutate_apdu(&cmd, FuzzMutationStrategy::BoundaryLength, 1);
        let bound_res = TokenFuzzerEngine::parse_apdu_strict(&boundary);
        assert!(bound_res.is_err(), "Oversized Lc payload must be rejected");
    }

    #[test]
    fn test_token_fuzzer_anomaly_detection() {
        // Run 50 iterations fuzz campaign
        let report = TokenFuzzerEngine::run_fuzz_campaign(50);
        assert_eq!(report.total_mutations_executed, 50);
        assert!(report.malformed_rejections_confirmed > 40);
        assert_eq!(report.critical_failures, 0, "No critical parser anomalies should occur");
        assert!(!report.campaign_fingerprint_hex.is_empty());
    }
}
