# Bounded local diagnostic events (v0.26.0)

`crash_record` records a caller-requested diagnostic label. It does not prove that
a real panic, out-of-memory condition or sandbox escape occurred, and does not
install an automatic panic handler or upload data.

## Contract

- `errorKind` is one of `assertion`, `panic`, `oom`, `ipc_error`,
  `sandbox_escape` or `ControlledDiagnosticTrap`. The legacy `kind` alias is
  accepted if `errorKind` is absent. Omission selects `ControlledDiagnosticTrap`;
  explicit malformed/unknown values fail. Arbitrary labels are not stored.
- Caller messages and stack traces are omitted. The TypeScript helper retains
  its message argument for source compatibility but does not transmit it.
- Successful recording returns `id`, `persisted: true` and the original record
  fields. A storage/locking/validation error returns an unsuccessful IPC response;
  no in-memory-only success is substituted.
- `crash_list` returns `entries`, `total`, the legacy `crashes` array,
  `retentionLimit: 32`, `scope: "application"` and the storage location. Timestamps
  in `entries` are Unix seconds rendered as strings.

## Storage and resource boundaries

The newest 32 records are retained in `<application-data-root>/diagnostics/events.json`.
Each event has a cryptographically random 128-bit ID, fixing the old second-based
filename collision. The complete journal is limited to 64 KiB before parsing and
before writing; growth during reading is also checked. There is no unbounded
process-lifetime history or per-request diagnostic-file accumulation.

This journal is application-wide and is separate from profile vaults. It contains
allowlisted labels and a fixed explanatory message, not private free-form traces.
Changing profiles does not clear it. Earlier `sandbox/crashes/crash_*.json` files
are left untouched and are not imported. Preserve or remove those legacy files
explicitly; their old content is outside the new journal's retention guarantee.

An in-process mutex and a nonblocking OS lock serialize cooperating access.
Writers reload the current journal under the lock, then use the shared same-directory
atomic-write helper. A second runtime does not rely on a stale in-memory snapshot.
Busy storage returns a retryable error. Unix-created journal/lock files use mode
0600 through the storage helper/open options; Windows uses inherited application
directory permissions and does not claim a separately provisioned ACL.

Directories, journal and lock targets reject observed symlinks/reparse points or
wrong file types. Invalid/oversized history is preserved and causes operations to
fail; it is not silently overwritten. The caller must preserve the file and repair
or explicitly remove it to start a new journal. Filesystem races by a malicious
same-account actor, hostile hardlinks, every power-loss/storage failure and
cross-filesystem durability are outside this guarantee. The existing storage
helper's OS/filesystem assumptions still apply.

## Verification

```text
cargo test --locked --manifest-path src-core/Cargo.toml ipc_crash_history_is_bounded
node aegis-cli/bin/portable-smoke.js
npm --prefix src-ui run build
```

The public IPC regression first reproduced 40 retained records where the budget
was 32, duplicate IDs in a burst, and missing history after a fresh reporter was
constructed. It now checks retention/restart, rejected labels, persistence failure
and corrupted-history refusal. The native-binary smoke checks both response
formats and omission of a test-only message on each OS runner. These checks are
project-authored evidence; physical-device and independent-review gates remain.
