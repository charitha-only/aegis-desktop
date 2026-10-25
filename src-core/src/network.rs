use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Mutex;
use crate::security::attestation::{TpmAttestationEngine, TpmAttestationProof};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRule {
    pub id: String,
    pub domain_pattern: String,
    pub action: String, // "ALLOW" or "BLOCK"
    pub description: String,
    #[serde(default)]
    pub requires_tpm_attestation: bool,
    #[serde(default)]
    pub min_tpm_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEvaluation {
    pub target_host: String,
    pub port: u16,
    pub allowed: bool,
    pub matched_rule: Option<String>,
    pub reason: String,
    #[serde(default)]
    pub attestation_token: Option<String>,
    #[serde(default)]
    pub tpm_verified: bool,
}

/// Dynamic zero-trust network firewall policy engine
/// Restricts outbound connections from UI, WebViews, or plugins,
/// enforcing TPM 2.0 remote attestation proofs on enterprise boundaries.
pub struct NetworkPolicyEngine {
    rules: Mutex<Vec<FirewallRule>>,
    allowed_domains: Mutex<HashSet<String>>,
}

impl NetworkPolicyEngine {
    pub fn new() -> Self {
        let default_rules = vec![
            FirewallRule {
                id: "rule-block-telemetry".to_string(),
                domain_pattern: "*.telemetry.com".to_string(),
                action: "BLOCK".to_string(),
                description: "Block all third-party telemetry harvesting".to_string(),
                requires_tpm_attestation: false,
                min_tpm_version: String::new(),
            },
            FirewallRule {
                id: "rule-block-analytics".to_string(),
                domain_pattern: "*google-analytics.com".to_string(),
                action: "BLOCK".to_string(),
                description: "Strict air-gap privacy defense".to_string(),
                requires_tpm_attestation: false,
                min_tpm_version: String::new(),
            },
            FirewallRule {
                id: "rule-enterprise-intranet".to_string(),
                domain_pattern: "*.corp.internal".to_string(),
                action: "ALLOW".to_string(),
                description: "Enterprise Zero-Trust Intranet Boundary".to_string(),
                requires_tpm_attestation: true,
                min_tpm_version: "2.0".to_string(),
            },
            FirewallRule {
                id: "rule-enterprise-vault".to_string(),
                domain_pattern: "*.enterprise-vault.net".to_string(),
                action: "ALLOW".to_string(),
                description: "Enterprise Confidential Vault Endpoint".to_string(),
                requires_tpm_attestation: true,
                min_tpm_version: "2.0".to_string(),
            },
            FirewallRule {
                id: "rule-allow-localhost".to_string(),
                domain_pattern: "localhost".to_string(),
                action: "ALLOW".to_string(),
                description: "Allow internal IPC loopback".to_string(),
                requires_tpm_attestation: false,
                min_tpm_version: String::new(),
            },
            FirewallRule {
                id: "rule-allow-127".to_string(),
                domain_pattern: "127.0.0.1".to_string(),
                action: "ALLOW".to_string(),
                description: "Allow loopback IPv4".to_string(),
                requires_tpm_attestation: false,
                min_tpm_version: String::new(),
            },
        ];

        let mut allowed = HashSet::new();
        allowed.insert("localhost".to_string());
        allowed.insert("127.0.0.1".to_string());

        Self {
            rules: Mutex::new(default_rules),
            allowed_domains: Mutex::new(allowed),
        }
    }

    /// Evaluate connection with standard zero-trust rules
    pub fn evaluate_connection(&self, host: &str, port: u16) -> NetworkEvaluation {
        self.evaluate_connection_internal(host, port, None, None, "")
    }

    /// Evaluate connection with enterprise TPM 2.0 remote attestation proof
    pub fn evaluate_connection_with_attestation(
        &self,
        host: &str,
        port: u16,
        attestation_proof: Option<&TpmAttestationProof>,
        attestation_engine: &TpmAttestationEngine,
        expected_nonce: &str,
    ) -> NetworkEvaluation {
        self.evaluate_connection_internal(
            host,
            port,
            attestation_proof,
            Some(attestation_engine),
            expected_nonce,
        )
    }

    fn evaluate_connection_internal(
        &self,
        host: &str,
        port: u16,
        attestation_proof: Option<&TpmAttestationProof>,
        attestation_engine: Option<&TpmAttestationEngine>,
        expected_nonce: &str,
    ) -> NetworkEvaluation {
        let rules = self.rules.lock().unwrap();

        // 1. Evaluate explicit BLOCK rules
        for rule in rules.iter().filter(|r| r.action == "BLOCK") {
            let pat = rule.domain_pattern.replace("*", "");
            if host.ends_with(&pat) || host == rule.domain_pattern {
                return NetworkEvaluation {
                    target_host: host.to_string(),
                    port,
                    allowed: false,
                    matched_rule: Some(rule.id.clone()),
                    reason: format!("Violates policy '{}': Outbound telemetry and tracking strictly prohibited", rule.description),
                    attestation_token: None,
                    tpm_verified: false,
                };
            }
        }

        // 2. Evaluate explicit ALLOW rules
        for rule in rules.iter().filter(|r| r.action == "ALLOW") {
            let pat = rule.domain_pattern.replace("*", "");
            if host.ends_with(&pat) || host == rule.domain_pattern {
                // If rule requires enterprise TPM remote attestation:
                if rule.requires_tpm_attestation {
                    match (attestation_proof, attestation_engine) {
                        (Some(proof), Some(engine)) => {
                            match engine.verify_quote(proof, expected_nonce) {
                                Ok(ver) => {
                                    return NetworkEvaluation {
                                        target_host: host.to_string(),
                                        port,
                                        allowed: true,
                                        matched_rule: Some(rule.id.clone()),
                                        reason: format!(
                                            "Approved: Enterprise TPM 2.0 hardware remote attestation verified (PCR0, PCR7 SecureBoot, PCR10 IMA) for policy '{}'",
                                            rule.description
                                        ),
                                        attestation_token: Some(ver.verification_token),
                                        tpm_verified: true,
                                    };
                                }
                                Err(err) => {
                                    return NetworkEvaluation {
                                        target_host: host.to_string(),
                                        port,
                                        allowed: false,
                                        matched_rule: Some(rule.id.clone()),
                                        reason: format!("POLICY_DENY_ATTESTATION_FAILED: {}", err),
                                        attestation_token: None,
                                        tpm_verified: false,
                                    };
                                }
                            }
                        }
                        _ => {
                            return NetworkEvaluation {
                                target_host: host.to_string(),
                                port,
                                allowed: false,
                                matched_rule: Some(rule.id.clone()),
                                reason: format!(
                                    "POLICY_DENY_ATTESTATION_REQUIRED: Enterprise domain '{}' requires valid TPM 2.0 remote attestation proof under policy '{}'.",
                                    host, rule.description
                                ),
                                attestation_token: None,
                                tpm_verified: false,
                            };
                        }
                    }
                }

                return NetworkEvaluation {
                    target_host: host.to_string(),
                    port,
                    allowed: true,
                    matched_rule: Some(rule.id.clone()),
                    reason: format!("Approved under zero-trust policy '{}'", rule.description),
                    attestation_token: None,
                    tpm_verified: false,
                };
            }
        }

        // 3. Default Zero-Trust Stance: Deny-by-default for unknown outbound
        NetworkEvaluation {
            target_host: host.to_string(),
            port,
            allowed: false,
            matched_rule: None,
            reason: "Default Zero-Trust Stance: Deny-by-default outbound network policy".to_string(),
            attestation_token: None,
            tpm_verified: false,
        }
    }

    pub fn list_rules(&self) -> Vec<FirewallRule> {
        self.rules.lock().unwrap().clone()
    }

    pub fn add_rule(&self, rule: FirewallRule) {
        let mut rules = self.rules.lock().unwrap();
        rules.push(rule);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enterprise_zero_trust_denies_without_attestation() {
        let engine = NetworkPolicyEngine::new();
        let eval = engine.evaluate_connection("gateway.corp.internal", 443);
        assert!(!eval.allowed);
        assert!(eval.reason.contains("POLICY_DENY_ATTESTATION_REQUIRED"));
    }

    #[test]
    fn test_enterprise_zero_trust_allows_with_valid_tpm_attestation() {
        let net_engine = NetworkPolicyEngine::new();
        let tpm_engine = TpmAttestationEngine::new();
        let nonce = "zt_session_challenge_5510";

        let proof = tpm_engine.generate_quote(nonce, &[0, 7, 10, 11]).expect("quote");
        let eval = net_engine.evaluate_connection_with_attestation(
            "gateway.corp.internal",
            443,
            Some(&proof),
            &tpm_engine,
            nonce,
        );

        assert!(eval.allowed);
        assert!(eval.tpm_verified);
        assert!(eval.attestation_token.is_some());
        assert!(eval.attestation_token.unwrap().starts_with("zt_attest_"));
    }

    #[test]
    fn test_enterprise_zero_trust_blocks_with_invalid_nonce_replay() {
        let net_engine = NetworkPolicyEngine::new();
        let tpm_engine = TpmAttestationEngine::new();

        let proof = tpm_engine.generate_quote("original_challenge", &[0, 7, 10]).expect("quote");
        let eval = net_engine.evaluate_connection_with_attestation(
            "gateway.corp.internal",
            443,
            Some(&proof),
            &tpm_engine,
            "forged_or_different_nonce",
        );

        assert!(!eval.allowed);
        assert!(!eval.tpm_verified);
        assert!(eval.reason.contains("POLICY_DENY_ATTESTATION_FAILED"));
    }
}
