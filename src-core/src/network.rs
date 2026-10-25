use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRule {
    pub id: String,
    pub domain_pattern: String,
    pub action: String, // "ALLOW" or "BLOCK"
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEvaluation {
    pub target_host: String,
    pub port: u16,
    pub allowed: bool,
    pub matched_rule: Option<String>,
    pub reason: String,
}

/// Dynamic zero-trust network firewall policy engine
/// Restricts outbound connections from UI, WebViews, or plugins
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
            },
            FirewallRule {
                id: "rule-block-analytics".to_string(),
                domain_pattern: "*google-analytics.com".to_string(),
                action: "BLOCK".to_string(),
                description: "Strict air-gap privacy defense".to_string(),
            },
            FirewallRule {
                id: "rule-allow-localhost".to_string(),
                domain_pattern: "localhost".to_string(),
                action: "ALLOW".to_string(),
                description: "Allow internal IPC loopback".to_string(),
            },
            FirewallRule {
                id: "rule-allow-127".to_string(),
                domain_pattern: "127.0.0.1".to_string(),
                action: "ALLOW".to_string(),
                description: "Allow loopback IPv4".to_string(),
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

    pub fn evaluate_connection(&self, host: &str, port: u16) -> NetworkEvaluation {
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
                };
            }
        }

        // 2. Evaluate explicit ALLOW rules
        for rule in rules.iter().filter(|r| r.action == "ALLOW") {
            let pat = rule.domain_pattern.replace("*", "");
            if host.ends_with(&pat) || host == rule.domain_pattern {
                return NetworkEvaluation {
                    target_host: host.to_string(),
                    port,
                    allowed: true,
                    matched_rule: Some(rule.id.clone()),
                    reason: format!("Approved under zero-trust policy '{}'", rule.description),
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
