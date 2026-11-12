use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::SystemTime;

const MAX_RING_BUFFER_CAPACITY: usize = 120;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketFlowRecord {
    pub id: String,
    pub timestamp: u64,
    pub target_host: String,
    pub port: u16,
    pub protocol: String,
    pub bytes_transferred: usize,
    pub latency_ms: u32,
    pub threat_score: u8, // 0 - 100
    pub decision: String, // "ALLOWED", "BLOCKED", "KILLSWITCH_DROPPED"
    pub matched_rule: String,
    pub geo_region: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficStats {
    pub total_flows_inspected: usize,
    pub total_allowed: usize,
    pub total_blocked: usize,
    pub total_bytes_transferred: u64,
    pub active_threats_neutralized: usize,
    pub killswitch_active: bool,
    pub average_latency_ms: f32,
}

pub struct TrafficMonitorEngine {
    killswitch: Mutex<bool>,
    ring_buffer: Mutex<Vec<PacketFlowRecord>>,
    total_inspected: Mutex<usize>,
    total_allowed: Mutex<usize>,
    total_blocked: Mutex<usize>,
    total_bytes: Mutex<u64>,
    threats_neutralized: Mutex<usize>,
    flow_counter: Mutex<u64>,
}

impl TrafficMonitorEngine {
    pub fn new() -> Self {
        let engine = Self {
            killswitch: Mutex::new(false),
            ring_buffer: Mutex::new(Vec::with_capacity(MAX_RING_BUFFER_CAPACITY)),
            total_inspected: Mutex::new(0),
            total_allowed: Mutex::new(0),
            total_blocked: Mutex::new(0),
            total_bytes: Mutex::new(0),
            threats_neutralized: Mutex::new(0),
            flow_counter: Mutex::new(1000),
        };

        // Seed with realistic baseline enterprise traffic
        engine.seed_baseline_traffic();
        engine
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    fn seed_baseline_traffic(&self) {
        let samples = vec![
            ("vault.internal.corp", 443, "mTLS", 4096, 12, 0, "ALLOWED", "rule-enterprise-vault", "CH-ZURICH"),
            ("127.0.0.1", 48291, "IPC/HTTP", 1280, 2, 0, "ALLOWED", "rule-allow-127", "LOCAL-LOOPBACK"),
            ("api.telemetry-tracker.com", 443, "HTTPS", 512, 45, 88, "BLOCKED", "rule-block-telemetry", "US-EAST"),
            ("analytics.google-analytics.com", 443, "HTTPS", 1024, 60, 75, "BLOCKED", "rule-block-analytics", "US-CENTRAL"),
            ("settlement.ch-clearing.corp", 8443, "mTLS", 8192, 18, 0, "ALLOWED", "rule-enterprise-intranet", "CH-GENEVA"),
            ("threat-feed.malicious-c2.net", 9001, "TCP", 256, 110, 99, "BLOCKED", "zero-trust-deny-default", "RU-MOSCOW"),
        ];

        for (host, port, proto, bytes, lat, threat, decision, rule, geo) in samples {
            self.insert_record(PacketFlowRecord {
                id: format!("FLW-{}", self.next_id()),
                timestamp: Self::current_timestamp().saturating_sub(60),
                target_host: host.to_string(),
                port,
                protocol: proto.to_string(),
                bytes_transferred: bytes,
                latency_ms: lat,
                threat_score: threat,
                decision: decision.to_string(),
                matched_rule: rule.to_string(),
                geo_region: geo.to_string(),
            });
        }
    }

    fn next_id(&self) -> u64 {
        let mut c = self.flow_counter.lock().unwrap();
        *c += 1;
        *c
    }

    fn insert_record(&self, record: PacketFlowRecord) {
        let mut total_inspected = self.total_inspected.lock().unwrap();
        *total_inspected += 1;

        let mut total_bytes = self.total_bytes.lock().unwrap();
        *total_bytes += record.bytes_transferred as u64;

        if record.decision == "ALLOWED" {
            let mut total_allowed = self.total_allowed.lock().unwrap();
            *total_allowed += 1;
        } else {
            let mut total_blocked = self.total_blocked.lock().unwrap();
            *total_blocked += 1;
            if record.threat_score >= 50 {
                let mut threats = self.threats_neutralized.lock().unwrap();
                *threats += 1;
            }
        }

        let mut ring = self.ring_buffer.lock().unwrap();
        if ring.len() >= MAX_RING_BUFFER_CAPACITY {
            ring.remove(0);
        }
        ring.push(record);
    }

    /// Evaluates and logs an outbound flow. If the emergency killswitch is active,
    /// all connections are hard-dropped immediately (fail-closed air gap).
    pub fn inspect_and_log(
        &self,
        host: &str,
        port: u16,
        protocol: &str,
        bytes: usize,
        latency_ms: u32,
        matched_rule: Option<&str>,
        rule_allowed: bool,
    ) -> PacketFlowRecord {
        let is_killed = *self.killswitch.lock().unwrap();

        let (decision, threat_score, region) = if is_killed {
            ("KILLSWITCH_DROPPED", 100, "AIRGAP_ISOLATED")
        } else if !rule_allowed {
            let score = if host.contains("malicious") || host.contains("c2") {
                95
            } else if host.contains("telemetry") || host.contains("analytics") {
                80
            } else {
                55
            };
            ("BLOCKED", score, "EXTERNAL_UNTRUSTED")
        } else {
            ("ALLOWED", 0, "SECURE_ZONE")
        };

        let record = PacketFlowRecord {
            id: format!("FLW-{}", self.next_id()),
            timestamp: Self::current_timestamp(),
            target_host: host.to_string(),
            port,
            protocol: protocol.to_string(),
            bytes_transferred: bytes,
            latency_ms,
            threat_score,
            decision: decision.to_string(),
            matched_rule: matched_rule.unwrap_or("zero-trust-deny").to_string(),
            geo_region: region.to_string(),
        };

        self.insert_record(record.clone());
        record
    }

    /// Toggles the emergency air-gap network kill-switch.
    /// When active, 100% of outbound packets are dropped.
    pub fn toggle_killswitch(&self, enabled: bool) -> bool {
        let mut ks = self.killswitch.lock().unwrap();
        *ks = enabled;
        *ks
    }

    #[allow(dead_code)]
    pub fn is_killswitch_active(&self) -> bool {
        *self.killswitch.lock().unwrap()
    }

    pub fn get_recent_flows(&self, limit: usize) -> Vec<PacketFlowRecord> {
        let ring = self.ring_buffer.lock().unwrap();
        let take_count = if limit == 0 { ring.len() } else { limit.min(ring.len()) };
        ring.iter().rev().take(take_count).cloned().collect()
    }

    pub fn get_stats(&self) -> TrafficStats {
        let ring = self.ring_buffer.lock().unwrap();
        let total_inspected = *self.total_inspected.lock().unwrap();
        let total_allowed = *self.total_allowed.lock().unwrap();
        let total_blocked = *self.total_blocked.lock().unwrap();
        let total_bytes = *self.total_bytes.lock().unwrap();
        let threats = *self.threats_neutralized.lock().unwrap();
        let killswitch_active = *self.killswitch.lock().unwrap();

        let avg_latency = if ring.is_empty() {
            0.0
        } else {
            let sum: u32 = ring.iter().map(|r| r.latency_ms).sum();
            sum as f32 / ring.len() as f32
        };

        TrafficStats {
            total_flows_inspected: total_inspected,
            total_allowed,
            total_blocked,
            total_bytes_transferred: total_bytes,
            active_threats_neutralized: threats,
            killswitch_active,
            average_latency_ms: avg_latency,
        }
    }

    pub fn clear_history(&self) {
        let mut ring = self.ring_buffer.lock().unwrap();
        ring.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traffic_monitor_initial_seed_and_stats() {
        let monitor = TrafficMonitorEngine::new();
        let stats = monitor.get_stats();
        assert!(stats.total_flows_inspected >= 6);
        assert!(stats.total_blocked >= 3);
        assert!(stats.total_allowed >= 3);
        assert!(!stats.killswitch_active);
    }

    #[test]
    fn test_traffic_monitor_inspect_flow() {
        let monitor = TrafficMonitorEngine::new();
        let flow = monitor.inspect_and_log(
            "internal.vault.aegis",
            443,
            "mTLS",
            2048,
            8,
            Some("rule-enterprise-vault"),
            true,
        );

        assert_eq!(flow.decision, "ALLOWED");
        assert_eq!(flow.threat_score, 0);

        let recent = monitor.get_recent_flows(5);
        assert_eq!(recent[0].target_host, "internal.vault.aegis");
    }

    #[test]
    fn test_traffic_monitor_killswitch_drops_all() {
        let monitor = TrafficMonitorEngine::new();
        monitor.toggle_killswitch(true);
        assert!(monitor.is_killswitch_active());

        // Even an allowed rule is dropped by killswitch
        let flow = monitor.inspect_and_log(
            "internal.vault.aegis",
            443,
            "mTLS",
            2048,
            8,
            Some("rule-enterprise-vault"),
            true,
        );

        assert_eq!(flow.decision, "KILLSWITCH_DROPPED");
        assert_eq!(flow.threat_score, 100);

        let stats = monitor.get_stats();
        assert!(stats.killswitch_active);
    }
}
