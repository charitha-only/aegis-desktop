# 0001. Biometric Enclave, Encrypted IPC Stream & Local Crash Reporter (v0.15.0)

## Context
Desktop applications face malicious process inspection on shared host machines and unmonitored uncaught panics. Traditional Electron frameworks either transmit IPC plaintext over unauthenticated websockets or upload diagnostic minidumps to external SaaS collectors, violating Zero-Trust air-gap policies.

## Decision
1. **Biometric Enclave Integration:** Bind master vault keys to native platform enclaves (Windows Hello TPM Virtual Cryptographic Provider / Apple TouchID Secure Enclave) with biometric authorization gating.
2. **Encrypted IPC Stream:** Wrap sensitive IPC payloads in authenticated symmetric keystreams derived from ephemeral session nonces, preventing local network tap snooping.
3. **Local Crash Reporter:** Implement an air-gapped panic handler that saves encrypted crash dumps strictly into `./sandbox/crashes/` with zero outbound pingbacks.

## Consequences
- Elevates Aegis security guarantees to government and defense-grade requirements.
- Zero outbound telemetry guarantees remain 100% intact even during fatal crashes.
