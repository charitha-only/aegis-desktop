# Portable filesystem contract

## Paths and profile identity

Application paths are relative to the active profile root. Traversal, absolute paths, alternate stream syntax, backslashes, reserved device names and linked path components are rejected by the shared capability resolver. Profiles reject case-insensitive ID and folder collisions on every platform, so a project cannot rely on Linux case sensitivity and later collide on Windows or a case-insensitive macOS volume.

Platform data/config/cache locations are resolved without creating directories. Applications must distinguish persistent data from disposable cache. Directory resolution does not grant filesystem permission to write there through the sandbox IPC API. Packaged startup creates its own data root. The current application identity remains the executable-parent folder name; stable configured identity and migration are still needed before arbitrary folder renaming is supported.

## Writes and recovery

Cooperating writers use the OS file lock in `.aegis-write.lock`. Replacement writes a random, exclusive temporary file in the target directory, flushes it, closes it and renames it onto the target. Unix temporary files request mode 0600; Windows permissions inherit from the containing application-data directory. Unix directory sync is attempted after rename. A failure after rename can report an error even though the new file is visible; callers must reread before retrying a non-idempotent operation.

Vault replacement authenticates the old encrypted generation under the write lock. The prior encrypted generation is retained as `.bak`. API bounds and quota checks apply separately. These guarantees cover cooperating application writers, not a malicious process racing path resolution with equivalent filesystem access.

Tests on hosted Windows, Linux and macOS runners exercise these operations on those runners' filesystems. This is not validation of every filesystem, network share, removable device, abrupt power failure or storage-controller cache. Real-device crash/power-loss acceptance remains a release gate for applications that require it.

## Lifecycle and OS credential storage

The Windows adapter observes WTS/power messages and requires successful registration for retained sessions. Linux/macOS retained sessions fail closed until real lifecycle adapters exist. Stateless encrypted-file operations remain available with a fresh passphrase.

An OS keychain/credential-store provider is not implemented. `platform_info.osKeyStore` is false; encrypted files must not be described as OS-keychain-backed. Embedded windows and native installers are independent capabilities and are not implied by portable core builds.
