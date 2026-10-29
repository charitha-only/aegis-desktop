# Persistent verified-release observations

```text
node aegis-cli/bin/aegis.js init-update-state <new-trusted-state-directory>
node aegis-cli/bin/aegis.js verify-tracked-update <artifact> <envelope.json> <trusted-public-key.pem> <trusted-policy.json> <trusted-state-directory>
```

These are offline developer tools. Initialize a new state directory explicitly; its parent must exist. Initialization never overwrites an existing directory, even if its state file is missing. The application must use a fixed, appropriately protected state path, separate from downloaded files, with the separately provisioned public key/policy described in [update verification](UPDATE_VERIFICATION.md).

Every tracked attempt freshly verifies the signature, target, expiry, policy's installed-version/sequence baseline and actual artifact bytes. It then checks the stored observation. The first accepted release binds this state directory to one app/OS/architecture. A new observation must advance both version and sequence; lower/equal sequences with changed release fields are rejected, as are higher sequences paired with non-advancing versions.

A retry must match all recorded release fields: target identity, version, sequence, expiry, hash and size. The key, signature, clock and base policy are checked again. It returns `retry: true` without rewriting the observation. Reordered but semantically identical signed metadata may retry; this pins release fields, not envelope formatting/signature bytes. Renewing expiry under the same sequence is rejected. A changed trusted baseline may make a previously accepted release ineligible.

Success reports `tracked: true`, `installed: false`, `autoApply: false` and the fresh verification result. This records a verification observation only; it never advances the policy's installed version, installs or executes anything. Stateless verify/inspect commands do not automatically consult this state. Future runtime/installer integration must explicitly use tracked verification and consume the same reverified staged bytes before application.

The directory contains versioned `state.json` with `schemaVersion: 1` and `lastVerified` (null initially). Recorded fields are appId/platform/architecture/version/sequence/expiresAt/sha256/size. State reads are capped at 8 KiB, ordinary-file checked and validated against the exact schema/canonical JSON form. Unknown, corrupt or missing state fails closed. The state is a local trusted record, not a publisher-signed attestation or tamper-proof counter.

Cooperating writers acquire an exclusive `.lock`. New state is written exclusively to `state.pending`, file-synced, closed and renamed over `state.json` while the lock is held. Ordinary failed writes remove only their own pending file and release their own lock; an existing lock/pending file is never overwritten or automatically deleted. Matching retries do not change the committed state. Initialization failure can leave an incomplete directory, which is not silently reinitialized.

A crash may leave a lock or pending file. Stop all users of that state directory and have its operator inspect/recover it; there is no automatic stale-lock breaking or promotion of an uncommitted pending record. Never delete the whole directory to bypass a validation error. Directory-entry durability, ambiguous filesystem failures and physical power-loss recovery still need acceptance.

POSIX modes request 0700/0600; Windows inherited ACLs are not replaced by those numeric modes. Same-account malware, an administrator, state deletion/replacement, disk-snapshot rollback, a changed state path or compromised trusted policy/key are outside this protection. Secure OS storage/anti-rollback anchors and key lifecycle remain separate gates. This is not complete TUF or automatic-updater compliance.

Tests cover persisted retries/new CLI processes, rollback/equivocation, invalid state, exclusive locks/concurrent observers, pending writes and target binding. The native artifact fixture exercises tracked retries on all three OS runners. The Linux private-tmpfs fixture tests ENOSPC during observation replacement, reuse of the prior committed observation, advancement after space returns and rejection of older staged metadata. Exact-source results belong in IMPLEMENTATION_STATUS.md; physical/other-filesystem faults must not be inferred from these checks.

Implementation references: [Node filesystem API](https://nodejs.org/download/release/v22.16.0/docs/api/fs.html) for file sync/rename operations and [TUF security goals](https://theupdateframework.io/docs/security/) for the wider threat model. This project-specific observation record does not implement the complete TUF roles or client workflow.

Exact-source evidence: `7662d6f`, portable workflow `40837986760`, passed tracked native-artifact retries on Windows/Linux/macOS and real Linux ENOSPC state-preservation/advancement checks. Native workflow `40837986717` and Windows installer/upgrade/backup/real-lock workflow `40837986783` also passed. These are hosted tests, not physical power-loss or tamper-proof storage evidence.

## Read-only operator diagnostics

```text
node aegis-cli/bin/aegis.js inspect-update-state <trusted-state-directory>
```

The inspector validates the committed state using the same bounded strict reader, then reports `lastVerified`, `lockPresent`, `pendingPresent` and `verificationBlockedAtInspection`. It never acquires a writer lock, writes files, reads lock/pending contents, clears them or decides whether a lock is stale. Missing/corrupt state is an error, not an invitation to reset history.

Reports have `snapshotOnly: true`, `recordAuthenticated: false` and `autoApply: false`. This is cached local metadata, not fresh signature/artifact/expiry verification. A false blocker flag is not permission to install: another process can change the directory immediately, and key/policy/clock checks have not run. Use tracked verification for an actual attempt and stop all state users before operator recovery. A coherent transactional snapshot, automated stale-lock clearing and crash recovery are not provided.

Tests preserve committed and blocker bytes, reject invalid state, exercise separate CLI execution and check the native-artifact record on all three hosted OSes. The Linux filesystem-fault fixture also inspects the retained observation while its private tmpfs is full; inspection must require no filesystem writes.
