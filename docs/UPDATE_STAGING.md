# Verified update staging preview

```text
node aegis-cli/bin/aegis.js stage-update <artifact> <envelope.json> <trusted-public-key.pem> <trusted-policy.json> <new-stage-directory>
node aegis-cli/bin/aegis.js inspect-update <stage-directory> <trusted-public-key.pem> <trusted-policy.json>
```

Use the independently provisioned trust inputs and signed envelope format from [update verification](UPDATE_VERIFICATION.md). Both commands are offline developer tools. They never install, execute or download an update and never advance the trusted policy's version/sequence floor. Runtime updater IPC stays disabled.

The stage parent must exist; the destination must not exist. Initial signature/target/version/expiry/sequence/hash verification happens before directory creation. The command creates the new directory exclusively, copies only `artifact.bin` and `envelope.json` with size limits and exclusive file creation, then verifies those copied bytes again against the external key/policy. Verification results must match the initial check. Source changes do not silently substitute a different update.

Copied file buffers are synced. Readiness data is written to `ready.pending`, synced and renamed to `ready.json` last within the newly owned directory. Success requires exactly the three final files. Inspection checks directory/file types, bounded marker bytes, current trust policy and clock, the signed envelope and actual copied artifact bytes; it compares readiness data to freshly computed verification results. The marker itself is not a trust root. There are no metadata-provided paths, keys or policies inside the stage.

Ordinary failures remove only the directory this attempt created. Existing destinations are never overwritten or recursively removed by staging. Concurrent cooperating attempts at the same destination yield one successful creator. A rejected download creates no destination, allowing a repaired download to retry the same path. Inspect a completed stage to reuse it; staging again at that path is rejected.

Abrupt process termination can leave an incomplete folder. Inspection rejects missing markers, pending/extra files and mismatched bytes. Do not automatically trust or delete a pre-existing partial folder; choose a fresh destination or have its operator explicitly remove it. This preview does not resume partial copies or implement installer retries.

POSIX directory/file modes request 0700/0600. On Windows these mode values do not establish equivalent per-user ACL isolation; inherited ACLs and the operating-system account remain part of the trust model. See [Node filesystem documentation](https://nodejs.org/download/release/v22.16.0/docs/api/fs.html). Same-account malicious writers, racing filesystem replacement and hardlinks/parent-directory aliases are not a sandbox boundary.

Tests cover source independence, marker/staged-byte changes, missing markers, extra files, expiry/sequence policy changes, exclusive concurrent creation, invalid-download repair/retry and actual CLI behavior. The native fixture stages a copied real core binary, changes the original download, confirms the staged copy still verifies, then alters staged bytes and requires rejection. It does not execute or install that artifact.

File sync and marker ordering are not a universal power-loss guarantee. Directory-entry durability, disk-full/write-fault behavior on other filesystems, abrupt-process and physical power-loss tests still require further acceptance. Inspection is point-in-time: a future installer must consume/reverify the same controlled bytes and implement transactional installation/migration/recovery. Persisted rollback/retry state, key lifecycle, signing and independent review remain open. Never infer a completed automatic updater from a successful staging result.

Exact-source hosted evidence: `af77854`, portable workflow `40793593926`, passed actual native artifact staging/inspection on Windows/Linux/macOS. Native workflow `40793593939` and Windows installer/upgrade/backup/real-lock workflow `40793593945` also passed. These results do not cover physical power failure or disk-full recovery.

Bounded hosted-Linux ENOSPC cleanup and capacity-restoration retry now have [real filesystem fault acceptance](UPDATE_FILESYSTEM_FAULTS.md). This evidence does not extend to NTFS/APFS, physical media or power loss.
