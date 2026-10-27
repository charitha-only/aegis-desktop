# Trusted IPC input rules: initial inventory

This is a scoped command inventory, not a completed audit of every command.
The HTTP envelope is typed as `IpcRequest`; the core validates its process token
before dispatch. Application payload fields are interpreted by Rust handlers.
The process token is a reusable bearer credential, not user identity.

## firewall_evaluate (v0.26.6)

| Field | Current contract | Remaining assessment |
| --- | --- | --- |
| `host` | Required JSON string, passed to the policy matcher | Complete hostname length, syntax and normalization contract remains unassessed. This API does not resolve DNS or make connections. |
| `port` | Omission defaults to 443. Explicit values must be JSON unsigned integers from 1 through 65535. Wrong types, null, fractions, zero, negatives and overflow are rejected before evaluation. | Port validity alone does not imply that a destination is authorized or reachable. |
| `nonce` | Optional string for expected attestation nonce; handler otherwise uses the envelope nonce | This is not a user login. Full optional-field type validation remains unassessed. |
| `attestationProof` | Optional typed proof; invalid proof cannot establish verification | Real TPM provider remains unavailable; TPM-dependent policy decisions must not grant verified status. |

The command evaluates local rules and returns a decision. It is **not an OS
firewall**, does not intercept browser/plugin traffic, does not establish a
connection and does not authenticate a remote service. A calling app must apply
its own destination/authorization policy through an actual network adapter.

### Confirmed validation bug and remediation

The prior `as u16` conversion truncated oversized integers, while wrong types
silently selected the default. A real IPC regression demonstrated a successful
result for port 65979 (reported as 443) before remediation. Checked conversion
now rejects invalid values; absence alone selects the default. This is classified
as a command validation/logic defect; no remote execution or network boundary
bypass has been demonstrated.

`aegis-cli/bin/portable-smoke.js` covers overflow (65979 and 65536), zero,
negative/fractional/string/null/boolean/object values, valid ports 1 and 65535,
and omission. Portable CI repeats the same checks against an owned raw core
and a packaged runtime on Windows, Linux and macOS. Fixtures make only local
IPC requests; policy evaluation performs no external networking.

```text
cargo build --release --locked --manifest-path src-core/Cargo.toml
node aegis-cli/bin/portable-smoke.js
npm --prefix src-ui run build
node aegis-cli/bin/package-smoke.js
```

## Remaining inventory

Review each filesystem, profile, vault/session, native-service, plugin/Wasm,
window and diagnostics payload field, including unknown fields, optional values,
length/type/range bounds, capability gates and retained-session authorization.
Disabled provider dispatch must remain fail closed. Nested payload duplicate-key
handling and cross-parser behavior remain unassessed. This document does not
mark those unfinished areas as passing.

See [ASVS_5_MAPPING.md](ASVS_5_MAPPING.md) for pinned requirements and scope, and
[IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) for exact-source evidence.
