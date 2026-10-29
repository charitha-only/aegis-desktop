# Bounded update filesystem fault acceptance

The guarded `aegis-cli/bin/update-filesystem-fault-smoke.js` acceptance fixture runs only on disposable GitHub-hosted Linux runners. It uses the public verify/stage/inspect CLI commands against a signed 8 MiB test artifact and an ephemeral publisher key. No production keys, installers or user vaults are used.

It mounts a private tmpfs under an exclusively created temporary fixture, limited to 1 MiB, with owner-only directory mode and nodev/nosuid/noexec. The Linux kernel must return ENOSPC during staging. The test requires CLI failure, no residual/ready stage, failed inspection and unchanged trusted policy. It raises only this private mount's capacity to 32 MiB and retries at the same path, requiring successful fresh inspection.

A second check fills that bounded filesystem while a verified stage already exists. Another staging attempt must fail with ENOSPC and leave no second destination. The earlier stage must still authenticate, and the policy must remain unchanged. Removing only the test filler frees capacity; retry must then succeed.

The fixture validates its exact mount path/type/source before unmounting. If ownership verification or unmount fails, recursive cleanup does not run. The filesystem is bounded to 32 MiB; it does not fill the runner's system disk. Local Windows/macOS and self-hosted environments are rejected before creating a fixture or invoking sudo/mount. Syntax and unsupported-host behavior can be checked locally; actual ENOSPC results require the hosted runner.

This tests the existing implementation under a real kernel allocation failure rather than mocking filesystem writes. It is a tmpfs capacity test, not evidence for physical disks, NTFS/APFS, media errors, directory durability, abrupt power loss, installer migrations or persisted rollback state. Those gates remain open. See [Linux kernel tmpfs documentation](https://docs.kernel.org/filesystems/tmpfs.html) for the bounded allocation and remount model, and [update staging](UPDATE_STAGING.md) for the API contract.
