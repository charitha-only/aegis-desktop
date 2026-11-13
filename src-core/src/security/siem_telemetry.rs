//! Enterprise SIEM, Syslog (RFC 5424) & ETW Security Telemetry Exporter
//!
//! Emits standardized security events formatted in Common Event Format (CEF),
//! RFC 5424 Syslog, and Event Tracing for Windows (ETW) schema. Integrates directly
//! with enterprise SIEM platforms (Splunk, Elastic, Datadog, Sentinel).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Standard SIEM Severity Levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SiemSeverity {
    Informational = 1,
    Notice = 3,
    Warning = 5,
    Error = 7,
    Critical = 9,
    Emergency = 10,
}

/// Security Incident Classification Categories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityEventCategory {
    TamperAttempt,
    RateLimitExceeded,
    UnauthorizedOrigin,
    StepUpChallengeFailed,
    CanaryCorrupted,
    FourEyesViolation,
    PostQuantumKeyMismatch,
    KernelSandboxViolation,
}

impl SecurityEventCategory {
    pub fn event_code(&self) -> &'static str {
        match self {
            Self::TamperAttempt => "SEC-001-TAMPER",
            Self::RateLimitExceeded => "SEC-002-RATELIMIT",
            Self::UnauthorizedOrigin => "SEC-003-ORIGIN",
            Self::StepUpChallengeFailed => "SEC-004-STEPUP",
            Self::CanaryCorrupted => "SEC-005-CANARY",
            Self::FourEyesViolation => "SEC-006-FOUREYES",
            Self::PostQuantumKeyMismatch => "SEC-007-PQKEY",
            Self::KernelSandboxViolation => "SEC-008-SANDBOX",
        }
    }
}

/// Structured Security Telemetry Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiemSecurityEvent {
    pub event_id: String,
    pub timestamp_epoch_millis: u64,
    pub category: SecurityEventCategory,
    pub severity: SiemSeverity,
    pub source_ip: String,
    pub principal_identity: String,
    pub description: String,
    pub forensic_evidence_hash: String,
}

impl SiemSecurityEvent {
    /// Format event into Common Event Format (CEF:0|Vendor|Product|Version|SignatureID|Name|Severity|Extension)
    pub fn to_cef_format(&self) -> String {
        format!(
            "CEF:0|AegisDesktop|AegisCore|1.5.0|{}|{}|{}|src={} suser={} msg={} cs1Label=ForensicHash cs1={}",
            self.category.event_code(),
            format!("{:?}", self.category),
            self.severity as u8,
            self.source_ip,
            self.principal_identity,
            self.description.replace('|', "\\|"),
            self.forensic_evidence_hash
        )
    }

    /// Format event into RFC 5424 Syslog line
    pub fn to_rfc5424_syslog(&self) -> String {
        let prival = 16 * 8 + (self.severity as u8); // Local0 facility
        format!(
            "<{}>1 {} aegis-desktop aegis-core - {} [aegis@5424 category=\"{:?}\" forensicHash=\"{}\"] {}",
            prival,
            self.timestamp_epoch_millis,
            self.category.event_code(),
            self.category,
            self.forensic_evidence_hash,
            self.description
        )
    }
}

/// Enterprise SIEM Telemetry Buffer Engine
pub struct SiemTelemetryEngine {
    ring_buffer: VecDeque<SiemSecurityEvent>,
    max_capacity: usize,
    total_events_emitted: u64,
}

impl SiemTelemetryEngine {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            ring_buffer: VecDeque::with_capacity(max_capacity),
            max_capacity,
            total_events_emitted: 0,
        }
    }

    /// Record a high-assurance security incident event
    pub fn record_incident(
        &mut self,
        category: SecurityEventCategory,
        severity: SiemSeverity,
        source_ip: &str,
        principal: &str,
        description: &str,
        evidence_hash: &str,
        timestamp: u64,
    ) -> SiemSecurityEvent {
        self.total_events_emitted += 1;
        let event_id = format!("EVT-{}-{:08X}", timestamp, self.total_events_emitted);

        let event = SiemSecurityEvent {
            event_id,
            timestamp_epoch_millis: timestamp,
            category,
            severity,
            source_ip: source_ip.to_string(),
            principal_identity: principal.to_string(),
            description: description.to_string(),
            forensic_evidence_hash: evidence_hash.to_string(),
        };

        if self.ring_buffer.len() >= self.max_capacity {
            self.ring_buffer.pop_front();
        }
        self.ring_buffer.push_back(event.clone());

        event
    }

    /// Export buffered events as batch CEF lines
    pub fn export_cef_batch(&self) -> Vec<String> {
        self.ring_buffer.iter().map(|e| e.to_cef_format()).collect()
    }

    /// Retrieve total count of emitted incidents
    pub fn get_total_emitted_count(&self) -> u64 {
        self.total_events_emitted
    }

    /// Query latest alerts filtered by severity
    pub fn query_by_min_severity(&self, min_severity: SiemSeverity) -> Vec<SiemSecurityEvent> {
        self.ring_buffer
            .iter()
            .filter(|e| (e.severity as u8) >= (min_severity as u8))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cef_formatting() {
        let event = SiemSecurityEvent {
            event_id: "EVT-001".to_string(),
            timestamp_epoch_millis: 1700000000000,
            category: SecurityEventCategory::CanaryCorrupted,
            severity: SiemSeverity::Critical,
            source_ip: "127.0.0.1".to_string(),
            principal_identity: "teller-01".to_string(),
            description: "Heap canary corruption detected on protected buffer".to_string(),
            forensic_evidence_hash: "A1B2C3D4E5F6".to_string(),
        };

        let cef = event.to_cef_format();
        assert!(cef.starts_with("CEF:0|AegisDesktop|AegisCore|1.5.0|SEC-005-CANARY|CanaryCorrupted|9|"));
        assert!(cef.contains("src=127.0.0.1"));
        assert!(cef.contains("cs1=A1B2C3D4E5F6"));
    }

    #[test]
    fn test_siem_ring_buffer_and_filtering() {
        let mut engine = SiemTelemetryEngine::new(3);

        engine.record_incident(
            SecurityEventCategory::RateLimitExceeded,
            SiemSeverity::Warning,
            "127.0.0.1",
            "guest",
            "Rate limit breached",
            "HASH1",
            100,
        );

        engine.record_incident(
            SecurityEventCategory::FourEyesViolation,
            SiemSeverity::Critical,
            "127.0.0.1",
            "maker-01",
            "Maker attempted to self-approve transaction",
            "HASH2",
            200,
        );

        let criticals = engine.query_by_min_severity(SiemSeverity::Critical);
        assert_eq!(criticals.len(), 1);
        assert_eq!(criticals[0].category, SecurityEventCategory::FourEyesViolation);

        let cef_lines = engine.export_cef_batch();
        assert_eq!(cef_lines.len(), 2);
    }
}

