# Portable filesystem contract

## Paths and profile identity

Application paths are relative to the active profile root. Traversal, absolute paths, alternate stream syntax, backslashes, reserved device names and linked path components are rejected by the shared capability resolver. Profiles reject case-insensitive ID and folder collisions on every platform, so a project cannot rely on Linux case sensitivity and later collide on Windows or a case-insensitive macOS volume.

Platform data/config/cache locations are resolved without creating directories. Applications must distinguish persistent data from disposable cache. Directory resolution does not grant filesystem permission to write there through the sandbox IPC API. Packaged startup creates its own data root. Packaged startup uses the configured stable appId; when absent, the executable-parent folder name is the legacy fallback. Keep appId unchanged across package renames. This is not an automatic migration of arbitrary legacy data identities.

## Writes and recovery

Cooperating writers use the OS file lock in `.aegis-write.lock`. Replacement writes a random, exclusive temporary file in the target directory, flushes it, closes it and renames it onto the target. Unix temporary files request mode 0600; Windows permissions inherit from the containing application-data directory. Unix directory sync is attempted after rename. A failure after rename can report an error even though the new file is visible; callers must reread before retrying a non-idempotent operation.

Vault replacement authenticates the old encrypted generation under the write lock. The prior encrypted generation is retained as `.bak`. API bounds and quota checks apply separately. These guarantees cover cooperating application writers, not a malicious process racing path resolution with equivalent filesystem access.

Tests on hosted Windows, Linux and macOS runners exercise these operations on those runners' filesystems. This is not validation of every filesystem, network share, removable device, abrupt power failure or storage-controller cache. Real-device crash/power-loss acceptance remains a release gate for applications that require it.

## Lifecycle and OS credential storage

The Windows adapter observes WTS/power messages and requires successful registration for retained sessions. Linux/macOS retained sessions fail closed until real lifecycle adapters exist. Stateless encrypted-file operations remain available with a fresh passphrase.

An OS keychain/credential-store provider is not implemented. `platform_info.osKeyStore` is false; encrypted files must not be described as OS-keychain-backed. Embedded windows and native installers are independent capabilities and are not implied by portable core builds.

## Directory listings and SDK response data — v0.29.8

`fs_list_sandbox` still requires root-directory read permission in the active profile. Failure to open/enumerate the directory or inspect an included entry now returns an IPC failure, without partial successful data. Previously a failed read_dir could be reported as a successful empty list, while entry/metadata errors could be omitted or replaced with zero/false defaults. This was a confirmed correctness/availability issue, not a demonstrated capability bypass.

The operation scans at most 4096 directory entries. Internal .aegis- names and names that cannot be represented as Unicode strings remain filtered, but consume the scan budget. Seeing an additional entry fails the operation explicitly. It does not delete entries or return a silently truncated list. A retry after reducing the directory count can succeed. Pagination/application-owned indexing for larger directories remains future work; the cap is a compatibility constraint for apps with large single-directory layouts.

Successful public entries are sorted lexicographically by UTF-8 name bytes, not locale or browser JavaScript collation. sizeBytes is filesystem metadata length, not a recursive directory usage measurement. The count limit bounds collected entries and enumeration work; it is not a disk-I/O time limit, an atomic filesystem snapshot, whole-process memory budget or protection against a same-account filesystem actor racing path resolution. Existing capability/path checks and atomic-write guarantees remain unchanged.

The SDK validates successful fs_read_file data (nonempty path and string content, including empty), fs_write_file receipts (nonempty path and nonnegative safe-integer bytesWritten), and fs_list_sandbox data (at most 4096 entries with nonempty name, boolean isDir and nonnegative safe-integer sizeBytes). Malformed data returns INVALID_RESPONSE without including invalid data. Zero-byte files, empty lists, Unicode and additional fields are allowed. Backend denials still pass through unchanged. These shape checks do not independently verify file contents, actual byte counts on disk, path authority or authenticity of a compromised backend. A rejected write receipt may follow an already completed write; reread before retrying.

Repeat `cargo test --locked --manifest-path src-core/Cargo.toml ipc_filesystem_listing` for real non-directory failure, exact scan boundary, overflow rejection and recovery, and sorted output through public IPC. The SDK loopback suite tests malformed file/list receipts; actual-core acceptance writes/reads TEST_ONLY Unicode content under temporary roots, verifies UTF-8 byte receipts, sorted listings, internal-name filtering and traversal denial on each OS.
