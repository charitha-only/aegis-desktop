pub mod app_container;
pub mod attestation;
pub mod canary_guard;
pub mod capabilities;
pub mod compliance_report;
pub mod enclave;
pub mod fido2_token;
pub mod fuzz_harness;
pub mod handshake;
pub mod hardware_key;
pub mod hsm_pkcs11;
pub mod kernel_sandbox;
pub mod macos_seatbelt;
pub mod mem_guard;
pub mod mtls_pinning;
pub mod p2p_sync;
pub mod pentest_suite;
pub mod piv_slots;
pub mod plugin_sig;
pub mod pq_dsa;
pub mod pq_kem;
pub mod privacy;
pub mod rasp;
pub mod ratelimit;
pub mod shamir_escrow;
pub mod sidechannel_auditor;
pub mod siem_telemetry;
pub mod smartcard_pcsc;
pub mod tpm_sealing;
pub mod validator;
pub mod vault;
pub mod zk_proofs;

#[allow(unused_imports)]
pub use app_container::{AppContainerCapability, AppContainerEngine, AppContainerProfile};
#[allow(unused_imports)]
pub use attestation::{AttestationVerification, TpmAttestationEngine, TpmAttestationProof};
#[allow(unused_imports)]
pub use canary_guard::CanaryProtectedBuffer;
pub use capabilities::{CapabilityManifest, SecurityEngine};
#[allow(unused_imports)]
pub use compliance_report::{ComplianceReportEngine, ComplianceReportPayload, SignedComplianceCertificate};
pub use enclave::BiometricEnclave;
pub use fido2_token::Fido2TokenEngine;
#[allow(unused_imports)]
pub use fuzz_harness::{ConstantTimeBenchmarkResult, FuzzHarnessEngine};
pub use handshake::HandshakeSession;
pub use hardware_key::HardwareKeyEngine;
#[allow(unused_imports)]
pub use hsm_pkcs11::{HsmKeyObject, HsmPkcs11Engine, HsmSignatureResult, HsmSlotInfo};
#[allow(unused_imports)]
pub use kernel_sandbox::{KernelSandboxConfig, KernelSandboxEngine, KernelSandboxStatus};
#[allow(unused_imports)]
pub use macos_seatbelt::{MacosSeatbeltEngine, SeatbeltProfile, SeatbeltRuleType};
#[allow(unused_imports)]
pub use mem_guard::{GuardedBuffer, GuardedSecretDescriptor, MemoryGuardEngine, MemoryGuardStatus};
pub use mtls_pinning::TlsPinningEngine;
pub use p2p_sync::P2pSyncEngine;
#[allow(unused_imports)]
pub use pentest_suite::{AttackVectorResult, PenTestEngine, PenTestVector, PenetrationTestReport};
#[allow(unused_imports)]
pub use piv_slots::{PivDeviceState, PivEngine, PivSignResult, PivSlotId, PivSlotInfo};
pub use plugin_sig::SignatureVerifier;
#[allow(unused_imports)]
pub use pq_dsa::{MlDsaEngine, MlDsaKeypairOutput, MlDsaMetadata, MlDsaSignOutput, MlDsaVerifyOutput};
#[allow(unused_imports)]
pub use pq_kem::{PqDecapsulationResult, PqEncapsulationResult, PqKemEngine, PqKemKeypair};
pub use privacy::PrivacyShield;
pub use rasp::RaspEngine;
pub use ratelimit::RateLimiter;
#[allow(unused_imports)]
pub use shamir_escrow::{ShamirEscrowEngine, ShamirShare};
#[allow(unused_imports)]
pub use sidechannel_auditor::{DudectAuditReport, SideChannelAuditor};
#[allow(unused_imports)]
pub use siem_telemetry::{SecurityEventCategory, SiemSecurityEvent, SiemSeverity, SiemTelemetryEngine};
#[allow(unused_imports)]
pub use smartcard_pcsc::{CommandApdu, ResponseApdu, SmartCardHardwareType, SmartCardPcscEngine, SmartCardReaderInfo};
#[allow(unused_imports)]
pub use tpm_sealing::{PcrRegister, SealedDataPackage, TpmEnclaveStatus, TpmSealingEngine};
pub use validator::OriginValidator;
pub use vault::VaultEngine;
#[allow(unused_imports)]
pub use zk_proofs::{ZkProofEngine, ZkProofMetadata, ZkProofVerificationResult, ZkRangeProof};

pub mod lifecycle;
#[cfg(all(target_os = "linux", feature = "linux-lifecycle"))]
pub mod linux_lifecycle;
#[cfg(test)]
mod regression_tests;
pub mod vault_session;
pub mod windows_lifecycle;
