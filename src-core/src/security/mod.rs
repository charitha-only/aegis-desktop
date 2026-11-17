pub mod anti_forensics;
pub mod app_container;
pub mod attestation;
pub mod blackbox_recorder;
pub mod canary_guard;
pub mod capabilities;
pub mod compliance_report;
pub mod confidential_computing;
pub mod ctap2_resident;
pub mod differential_privacy;
pub mod ebpf_microsegmentation;
pub mod enclave;
pub mod fido2_token;
pub mod fuzz_harness;
pub mod handshake;
pub mod hardware_hkdf;
pub mod hardware_key;
pub mod hsm_pkcs11;
pub mod kernel_sandbox;
pub mod macos_seatbelt;
pub mod mem_guard;
pub mod merkle_mountain_range;
pub mod micro_enclave;
pub mod mtls_pinning;
pub mod p2p_sync;
pub mod panic_sentinel;
pub mod pentest_suite;
pub mod piv_slots;
pub mod pkcs11_v3;
pub mod platform_security_audit;
pub mod plugin_sig;
pub mod pq_dsa;
pub mod pq_dsa87;
pub mod pq_identity_token;
pub mod pq_kem;
pub mod pq_kem1024;
pub mod pq_tls_handshake;
pub mod privacy;
pub mod rasp;
pub mod ratelimit;
pub mod seccomp_bpf;
pub mod secure_element_bus;
pub mod shamir_escrow;
pub mod sidechannel_auditor;
pub mod siem_telemetry;
pub mod silicon_puf;
pub mod smartcard_pcsc;
pub mod social_recovery;
pub mod threshold_mpc;
pub mod time_lock_vdf;
pub mod tpm_sealing;
pub mod validator;
pub mod vault;
pub mod wycheproof_vectors;
pub mod zk_accumulator;
pub mod zk_proofs;
pub mod zk_ring_signature;
pub mod pq_x509;
pub mod zk_range_proof;
pub mod proximity_bounding;
pub mod stride_threat_model;
pub mod mpc_kdf;
pub mod blind_signatures;
pub mod attestation_graph;
pub mod binary_hash_chain;
pub mod stateful_hss;
pub mod ram_honeytokens;
pub mod noise_pipe;
pub mod timelock_puzzle;
pub mod homomorphic_vault;
pub mod random_beacon;
pub mod cfi_flattening;
pub mod zk_merkle_tree;
pub mod threshold_bls;
pub mod pq_ratchet;
pub mod token_fuzzer;
pub mod hardware_kms;

#[allow(unused_imports)]
pub use anti_forensics::{AntiForensicsEngine, AntiForensicsReceipt, SanitizationStandard};
#[allow(unused_imports)]
pub use app_container::{AppContainerCapability, AppContainerEngine, AppContainerProfile};
#[allow(unused_imports)]
pub use attestation::{AttestationVerification, TpmAttestationEngine, TpmAttestationProof};
#[allow(unused_imports)]
pub use blackbox_recorder::{BlackBoxNotarizationSeal, BlackBoxRecorderEngine, FlightRecord};
#[allow(unused_imports)]
pub use canary_guard::CanaryProtectedBuffer;
pub use capabilities::{CapabilityManifest, SecurityEngine};
#[allow(unused_imports)]
pub use compliance_report::{ComplianceReportEngine, ComplianceReportPayload, SignedComplianceCertificate};
#[allow(unused_imports)]
pub use confidential_computing::{AttestationVerificationResult, ConfidentialComputingEngine, EnclaveMeasurementQuote};
#[allow(unused_imports)]
pub use ctap2_resident::{Ctap2DeviceStatus, Ctap2PinUvAuthToken, Ctap2ResidentEngine, ResidentCredential};
#[allow(unused_imports)]
pub use differential_privacy::{DifferentialPrivacyEngine, PerturbedMetricReport, PrivacyBudgetStatus};
#[allow(unused_imports)]
pub use ebpf_microsegmentation::{EbpfMicrosegmentationEngine, EbpfMicrosegmentationReport, SocketConnectionAttempt};
pub use enclave::BiometricEnclave;
pub use fido2_token::Fido2TokenEngine;
#[allow(unused_imports)]
pub use fuzz_harness::{ConstantTimeBenchmarkResult, FuzzHarnessEngine};
pub use handshake::HandshakeSession;
#[allow(unused_imports)]
pub use hardware_hkdf::{HardwareDerivedKey, HardwareHkdfEngine, KeyDomain};
pub use hardware_key::HardwareKeyEngine;
#[allow(unused_imports)]
pub use hsm_pkcs11::{HsmKeyObject, HsmPkcs11Engine, HsmSignatureResult, HsmSlotInfo};
#[allow(unused_imports)]
pub use kernel_sandbox::{KernelSandboxConfig, KernelSandboxEngine, KernelSandboxStatus};
#[allow(unused_imports)]
pub use macos_seatbelt::{MacosSeatbeltEngine, SeatbeltProfile, SeatbeltRuleType};
#[allow(unused_imports)]
pub use mem_guard::{GuardedBuffer, GuardedSecretDescriptor, MemoryGuardEngine, MemoryGuardStatus};
#[allow(unused_imports)]
pub use merkle_mountain_range::{MerkleMountainRange, MmrHash, MmrInclusionProof};
#[allow(unused_imports)]
pub use micro_enclave::{EnclaveCapability, EnclaveStatus, MicroEnclaveInstance, DEFAULT_ENCLAVE_FUEL_LIMIT, MAX_ENCLAVE_MEMORY_PAGES};
pub use mtls_pinning::TlsPinningEngine;
pub use p2p_sync::P2pSyncEngine;
#[allow(unused_imports)]
pub use panic_sentinel::{PanicSentinel, PanicSentinelStatus};
#[allow(unused_imports)]
pub use pentest_suite::{AttackVectorResult, PenTestEngine, PenTestVector, PenetrationTestReport};
#[allow(unused_imports)]
pub use piv_slots::{PivDeviceState, PivEngine, PivSignResult, PivSlotId, PivSlotInfo};
#[allow(unused_imports)]
pub use pkcs11_v3::{HsmSiliconKey, Pkcs11Mechanism, Pkcs11Session, Pkcs11TokenInfo, Pkcs11V3Engine};
#[allow(unused_imports)]
pub use platform_security_audit::{HostOsFamily, HostSecurityPosture, PlatformSecurityAuditor};
pub use plugin_sig::SignatureVerifier;
#[allow(unused_imports)]
pub use pq_dsa::{MlDsaEngine, MlDsaKeypairOutput, MlDsaMetadata, MlDsaSignOutput, MlDsaVerifyOutput};
#[allow(unused_imports)]
pub use pq_dsa87::{MlDsa87Engine, MlDsa87KeypairOutput, MlDsa87Metadata, MlDsa87SignOutput, MlDsa87VerifyOutput};
#[allow(unused_imports)]
pub use pq_identity_token::{PqIdentityToken, PqIdentityTokenEngine, PqTokenClaims, PqTokenValidationResult};
#[allow(unused_imports)]
pub use pq_kem::{PqDecapsulationResult, PqEncapsulationResult, PqKemEngine, PqKemKeypair};
#[allow(unused_imports)]
pub use pq_kem1024::{Hybrid1024Ciphertext, Hybrid1024Kem, Hybrid1024PublicKey, Hybrid1024SecretKey, MlKem1024Engine};
#[allow(unused_imports)]
pub use pq_tls_handshake::{HybridClientHello, HybridServerHello, HybridTlsSessionKeys, PqTlsHandshakeSynthesizer, Tls13KeyShareEntry};
pub use privacy::PrivacyShield;
pub use rasp::RaspEngine;
pub use ratelimit::RateLimiter;
#[allow(unused_imports)]
pub use seccomp_bpf::{SeccompBpfSynthesizer, SeccompPolicyReport, SockFilter};
#[allow(unused_imports)]
pub use secure_element_bus::{BusProtocol, SecureElementBusDriver, SecureElementFrame, SecureElementStatus, SECURE_ELEMENT_I2C_ADDR};
#[allow(unused_imports)]
pub use shamir_escrow::{ShamirEscrowEngine, ShamirShare};
#[allow(unused_imports)]
pub use sidechannel_auditor::{DudectAuditReport, SideChannelAuditor};
#[allow(unused_imports)]
pub use siem_telemetry::{SecurityEventCategory, SiemSecurityEvent, SiemSeverity, SiemTelemetryEngine};
#[allow(unused_imports)]
pub use silicon_puf::{ChipSiliconId, SiliconPufEngine};
#[allow(unused_imports)]
pub use smartcard_pcsc::{CommandApdu, ResponseApdu, SmartCardHardwareType, SmartCardPcscEngine, SmartCardReaderInfo};
#[allow(unused_imports)]
pub use social_recovery::{GuardianShare, ProximityAttestation, SocialRecoveryEngine, SocialRecoveryStatus, VerifiedGuardianSubmission, PROXIMITY_VALIDITY_WINDOW_SECONDS};
#[allow(unused_imports)]
pub use threshold_mpc::{AggregatedThresholdSignature, MpcParticipantKeyShare, NonceCommitment, PartialSignatureShare, ThresholdMpcEngine};
#[allow(unused_imports)]
pub use time_lock_vdf::{TimeLockVdfEngine, TimeLockedEnvelope, VdfProof};
#[allow(unused_imports)]
pub use tpm_sealing::{PcrRegister, SealedDataPackage, TpmEnclaveStatus, TpmSealingEngine};
pub use validator::OriginValidator;
pub use vault::VaultEngine;
#[allow(unused_imports)]
pub use wycheproof_vectors::{AttackVectorCheckResult, WycheproofAuditReport, WycheproofHarness};
#[allow(unused_imports)]
pub use zk_accumulator::{AccumulatorRevocationProof, AccumulatorWitness, DynamicAccumulatorState, ZkDynamicAccumulator};
#[allow(unused_imports)]
pub use zk_proofs::{ZkProofEngine, ZkProofMetadata, ZkProofVerificationResult, ZkRangeProof};
#[allow(unused_imports)]
pub use zk_ring_signature::{RingSignature, ZkRingSignatureEngine};
#[allow(unused_imports)]
pub use pq_x509::{verify_pq_cert_chain, CertValidationResult, PqX509Certificate, KEY_USAGE_CERT_SIGN, KEY_USAGE_CRL_SIGN, KEY_USAGE_DIGITAL_SIGNATURE, KEY_USAGE_KEY_ENCIPHERMENT, X509_VERSION_3};
#[allow(unused_imports)]
pub use zk_range_proof::{PedersenCommitment, RangeProofVerificationResult, RangeProver, ZkBulletproofRangeProof};
#[allow(unused_imports)]
pub use proximity_bounding::{DistanceBoundingEngine, DistanceBoundingResult, DistanceBoundingSession, RoundExchangeRecord, DEFAULT_MAX_ROUNDTRIP_NANOS, EXCHANGE_ROUND_COUNT};
#[allow(unused_imports)]
pub use stride_threat_model::{RiskSeverity, StrideAssessmentReport, StrideCategory, StrideThreatEvaluator, ThreatEntry};
#[allow(unused_imports)]
pub use mpc_kdf::{DerivedKeyResult, MpcKdfEngine, MpcKdfShare, PartialKdfEvaluation};
#[allow(unused_imports)]
pub use blind_signatures::{BlindSignatureAuthority, BlindSignatureClient, BlindSignatureVerificationResult, BlindedTokenRequest, UnblindedToken};
#[allow(unused_imports)]
pub use attestation_graph::{AttestationEvidenceNode, AttestationGraphEngine, AttestationGraphReport, AttestationRootType};
#[allow(unused_imports)]
pub use binary_hash_chain::{BinaryHashChain, BinaryHashChainReport, MemoryPageBlock};
#[allow(unused_imports)]
pub use stateful_hss::{LmsPrivateKey, LmsPublicKey, LmsSignature, LmsVerificationResult, LmsVerifier};
#[allow(unused_imports)]
pub use ram_honeytokens::{HoneytokenAuditReport, HoneytokenDefenseEngine, HoneytokenTripwire, HoneytokenType};
#[allow(unused_imports)]
pub use noise_pipe::{NoiseEncryptedFrame, NoisePipeChannel, NoisePipeSessionStatus};
#[allow(unused_imports)]
pub use timelock_puzzle::{PuzzleSolutionResult, TimeLockEngine, TimeLockPuzzle};
#[allow(unused_imports)]
pub use homomorphic_vault::{HomomorphicAdditionResult, HomomorphicCiphertext, HomomorphicVaultEngine};
#[allow(unused_imports)]
pub use random_beacon::{BeaconCommitment, BeaconReveal, RandomBeaconEngine, RandomBeaconRoundResult};
#[allow(unused_imports)]
pub use cfi_flattening::{CfiStatusReport, CfiSupervisor, ExecutionPhase, ShadowStackFrame};
#[allow(unused_imports)]
pub use zk_merkle_tree::{MerkleProofStep, ZkMerkleMembershipProof, ZkMerkleTree, ZkMerkleVerificationResult};
#[allow(unused_imports)]
pub use threshold_bls::{BlsAggregatedSignature, BlsKeyShare, BlsPartialSignature, ThresholdBlsEngine};
#[allow(unused_imports)]
pub use pq_ratchet::{PqRatchetEncryptedMessage, PqRatchetSession};
#[allow(unused_imports)]
pub use token_fuzzer::{ApduCommand, FuzzAnomaly, FuzzCampaignReport, FuzzMutationStrategy, TokenFuzzerEngine};
#[allow(unused_imports)]
pub use hardware_kms::{HardwareKmsEngine, KmsEnvelope};

pub mod lifecycle;
#[cfg(all(target_os = "linux", feature = "linux-lifecycle"))]
pub mod linux_lifecycle;
#[cfg(test)]
mod regression_tests;
pub mod vault_session;
pub mod windows_lifecycle;
