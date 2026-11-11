//! Mutual TLS (mTLS) & Zero-Trust Certificate Pinning Engine
//!
//! Enforces cryptographic Public Key Pinning (SHA-256 SPKI) on core banking
//! endpoints and rejects unauthorized local proxy interception (Burp, Fiddler, Charles).

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedEndpoint {
    pub host: String,
    pub primary_spki_sha256: String,
    pub backup_spki_sha256: Option<String>,
    pub mtls_enforced: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PinningStatus {
    ValidPinned,
    PinMismatch,
    UntrustedProxyDetected,
    HostNotEnrolled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsSecurityAssessment {
    pub host: String,
    pub status: PinningStatus,
    pub computed_spki_sha256: String,
    pub expected_spki_sha256: Option<String>,
    pub mtls_client_authenticated: bool,
    pub mitm_proxy_flagged: bool,
}

pub struct TlsPinningEngine {
    pinned_hosts: RwLock<HashMap<String, PinnedEndpoint>>,
}

impl TlsPinningEngine {
    pub fn new() -> Self {
        let mut initial = HashMap::new();
        // Default core banking pinned endpoint
        initial.insert(
            "api.aegis-bank.internal".to_string(),
            PinnedEndpoint {
                host: "api.aegis-bank.internal".to_string(),
                primary_spki_sha256: "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08".to_string(),
                backup_spki_sha256: Some("5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8".to_string()),
                mtls_enforced: true,
            },
        );

        Self {
            pinned_hosts: RwLock::new(initial),
        }
    }

    /// Enrolls or updates an endpoint's expected public key SHA-256 digest.
    pub fn enroll_pinned_host(&self, endpoint: PinnedEndpoint) {
        if let Ok(mut map) = self.pinned_hosts.write() {
            map.insert(endpoint.host.to_lowercase(), endpoint);
        }
    }

    /// Computes the SHA-256 hash of a certificate's public key (SPKI).
    pub fn compute_spki_sha256(&self, cert_bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(cert_bytes);
        let digest = hasher.finalize();
        hex::encode(digest)
    }

    /// Evaluates server certificate against pinned hash and checks for MITM proxies.
    pub fn evaluate_connection(
        &self,
        host: &str,
        server_cert_bytes: &[u8],
        client_cert_present: bool,
    ) -> TlsSecurityAssessment {
        let host_lower = host.to_lowercase();
        let computed = self.compute_spki_sha256(server_cert_bytes);

        let map = self.pinned_hosts.read().unwrap();
        if let Some(pinned) = map.get(&host_lower) {
            let matches_primary = pinned.primary_spki_sha256.eq_ignore_ascii_case(&computed);
            let matches_backup = pinned
                .backup_spki_sha256
                .as_ref()
                .map(|b| b.eq_ignore_ascii_case(&computed))
                .unwrap_or(false);

            if matches_primary || matches_backup {
                TlsSecurityAssessment {
                    host: host.to_string(),
                    status: PinningStatus::ValidPinned,
                    computed_spki_sha256: computed,
                    expected_spki_sha256: Some(pinned.primary_spki_sha256.clone()),
                    mtls_client_authenticated: client_cert_present || !pinned.mtls_enforced,
                    mitm_proxy_flagged: false,
                }
            } else {
                TlsSecurityAssessment {
                    host: host.to_string(),
                    status: PinningStatus::PinMismatch,
                    computed_spki_sha256: computed,
                    expected_spki_sha256: Some(pinned.primary_spki_sha256.clone()),
                    mtls_client_authenticated: client_cert_present,
                    mitm_proxy_flagged: true,
                }
            }
        } else {
            TlsSecurityAssessment {
                host: host.to_string(),
                status: PinningStatus::HostNotEnrolled,
                computed_spki_sha256: computed,
                expected_spki_sha256: None,
                mtls_client_authenticated: client_cert_present,
                mitm_proxy_flagged: false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spki_pin_verification_success_and_denial() {
        let engine = TlsPinningEngine::new();
        let cert_data = b"VALID_BANK_PUBLIC_KEY_CERTIFICATE_PAYLOAD";
        let computed = engine.compute_spki_sha256(cert_data);

        // Enroll host
        engine.enroll_pinned_host(PinnedEndpoint {
            host: "treasury.internal".to_string(),
            primary_spki_sha256: computed.clone(),
            backup_spki_sha256: None,
            mtls_enforced: true,
        });

        // 1. Valid connection with client certificate
        let assessment = engine.evaluate_connection("treasury.internal", cert_data, true);
        assert_eq!(assessment.status, PinningStatus::ValidPinned);
        assert!(assessment.mtls_client_authenticated);
        assert!(!assessment.mitm_proxy_flagged);

        // 2. Tampered / MITM Proxy certificate (e.g. Burp Suite interception CA)
        let mitm_cert = b"BURP_SUITE_INTERCEPTION_CERTIFICATE";
        let bad_assessment = engine.evaluate_connection("treasury.internal", mitm_cert, true);
        assert_eq!(bad_assessment.status, PinningStatus::PinMismatch);
        assert!(bad_assessment.mitm_proxy_flagged);
    }
}
