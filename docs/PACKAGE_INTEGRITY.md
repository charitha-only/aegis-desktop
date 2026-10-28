# v0.28.0 package integrity tooling preview

These are developer/release-operator file verification tools, not an installer,
application auto-updater, publisher signature or an independent security audit.
Node.js is required for these commands, not for the packaged Rust runtime.

## Generate from trusted build output

```text
node aegis-cli/bin/aegis.js package-manifest dist-bundle/YourApp
```

This writes `aegis-integrity.json` exclusively, failing if it already exists.
It contains a canonical schema-1 sorted file inventory with relative paths,
byte sizes and SHA-256 digests. The descriptor excludes itself. Standard output
is JSON containing the descriptor SHA-256, path, file count and byte total.
Existing bundles are not modified by `npm run bundle` to add this descriptor;
manifest creation is an explicit subsequent operation.

Retain the descriptor digest through a separately trusted channel, such as
owner-controlled authenticated release metadata. A hash downloaded alongside
untrusted contents from the same untrusted source does **not** authenticate a
publisher. A malicious producer can publish malicious files and matching hashes.
No owner signing key, trust root or signature is fabricated by this tool.

## Verify without executing

```text
node aegis-cli/bin/aegis.js verify-package dist-bundle/YourApp TRUSTED_MANIFEST_SHA256
```

Supply the actual 64-digit SHA-256 retained above, not the placeholder text.
The verifier first checks the existing descriptor against that pin, then rebuilds
the actual canonical inventory. It never parses or follows descriptor-supplied
paths. Missing, extra or altered files, and descriptor replacement, fail with a
nonzero exit. Success is JSON with `valid: true`,
`verificationScope: "fileBytesAgainstPinnedManifest"` and
`publisherAuthenticated: false`. This result says nothing about application logic,
malware absence or authorization to execute. Unknown/noncanonical descriptor
formats fail rather than being accepted as a partial manifest.

## Resource and path scope

- At most 4,096 content files, 8,192 content entries, 32 nested directory levels,
  and 1 GiB of content. The root descriptor does not count against content entries.
- At most 1 MiB of descriptor data, enforced on actual reading as well as metadata.
  Directory enumeration is incremental; file hashing streams and detects size
  changes while reading. Generated descriptors exceeding the limit are rejected.
- Roots must be ordinary directories; child symlinks/junctions and non-regular
  file entries are rejected. Common Windows reserved names, control/format
  characters, reserved punctuation and trailing dots/spaces are rejected.
  NFC/lowercase collisions are rejected. This is not a proof of every OS Unicode
  filename/extraction behavior. Paths and filename lengths are bounded.

The inventory covers regular file names and bytes, not empty directory identity,
Unix permission/executable bits, ACLs, extended attributes, ownership, installed
state or OS code-signing metadata. Parent-directory aliases and hardlinks are not
a complete filesystem sandbox. Concurrent changes/path-resolution races and
modification after verification remain outside the guarantee; verify immutable,
operator-controlled build output. The tool does not automatically execute or
install a verified package. A partial descriptor write may remain after an OS
failure; never reset a user's descriptor silently to make a check pass.

## Evidence

Public tests cover a known `abc` SHA-256 value, successful verification, modified
bytes, missing/extra files, descriptor replacement, traversal-like descriptor
records that are never followed, linked directories and existing-file preservation.
The actual package fixture invokes the CLI on a native Rust portable bundle,
boots it without a desktop, modifies an owned UI asset and verifies rejection.
Three-OS portable CI repeats those checks; exact-source results are in
[IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md).

```text
node --test aegis-cli/test/package-integrity.test.js
node aegis-cli/bin/package-smoke.js
```

## Remaining v0.28 gates

Native Linux/macOS installer formats and install/upgrade/uninstall/data migration,
owner signing/timestamping/notarization, signed update metadata and rollback/
interruption/recovery policy, actual SBOMs/provenance for published artifacts and
physical power-loss tests remain. This file inventory is **not an SBOM** and
portable-bundle CI is not native-installer parity. No binary release is published
or signed merely by tagging this source preview. See [RELEASE_GATES.md](RELEASE_GATES.md).
