pub mod attestation;
pub mod capabilities;
pub mod compliance_report;
pub mod enclave;
pub mod fido2_token;
pub mod handshake;
pub mod hardware_key;
pub mod hsm_pkcs11;
pub mod mtls_pinning;
pub mod p2p_sync;
pub mod plugin_sig;
pub mod privacy;
pub mod rasp;
pub mod ratelimit;
pub mod tpm_sealing;
pub mod validator;
pub mod vault;

#[allow(unused_imports)]
pub use attestation::{AttestationVerification, TpmAttestationEngine, TpmAttestationProof};
pub use capabilities::{CapabilityManifest, SecurityEngine};
#[allow(unused_imports)]
pub use compliance_report::{ComplianceReportEngine, ComplianceReportPayload, SignedComplianceCertificate};
pub use enclave::BiometricEnclave;
pub use fido2_token::Fido2TokenEngine;
pub use handshake::HandshakeSession;
pub use hardware_key::HardwareKeyEngine;
#[allow(unused_imports)]
pub use hsm_pkcs11::{HsmKeyObject, HsmPkcs11Engine, HsmSignatureResult, HsmSlotInfo};
pub use mtls_pinning::TlsPinningEngine;
pub use p2p_sync::P2pSyncEngine;
pub use plugin_sig::SignatureVerifier;
pub use privacy::PrivacyShield;
pub use rasp::RaspEngine;
pub use ratelimit::RateLimiter;
#[allow(unused_imports)]
pub use tpm_sealing::{PcrRegister, SealedDataPackage, TpmEnclaveStatus, TpmSealingEngine};
pub use validator::OriginValidator;
pub use vault::VaultEngine;

pub mod lifecycle;
#[cfg(all(target_os = "linux", feature = "linux-lifecycle"))]
pub mod linux_lifecycle;
#[cfg(test)]
mod regression_tests;
pub mod vault_session;
pub mod windows_lifecycle;
