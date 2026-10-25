# Aegis Desktop

Shared language for Aegis's local framework, encrypted storage and application profiles.

## Language

**Capability**:
An explicit runtime permission granted to an application for a class of local operations.
_Avoid_: Implicit privilege, unrestricted access

**Process Handshake Token**:
A runtime-wide bearer credential for local IPC, distinct from authentication to an encrypted vault. It is reusable and does not by itself identify a user or prevent replay.
_Avoid_: One-time nonce, user login, unlocked vault credential

**Encrypted Vault**:
A persistent container whose contents require its passphrase or an authorized unlocked vault session to read.
_Avoid_: Recoverable master password, hardware-backed vault

**Vault Session**:
Temporary authorization to read one authenticated vault in one profile, revoked by lock, expiry or relevant lifecycle changes.
_Avoid_: Process handshake token, operating-system login

**Vault Lock**:
Revocation of retained vault-session access and removal of its retained credentials. It does not retract plaintext already returned to an application or prevent fresh passphrase authentication.
_Avoid_: Password deletion, universal user logout

**Vault Backup Generation**:
The prior encrypted vault generation retained for authenticated recovery; it can retain an earlier password and contents.
_Avoid_: Offline disaster backup, password recovery service

**Profile Isolation**:
Separation of application storage across local workspaces, distinct from separation between operating-system accounts.
_Avoid_: OS account isolation, protection from same-account malware

**Wasm Sandbox**:
The restricted execution context for guest WebAssembly modules in Aegis.
_Avoid_: GPU execution, unrestricted host execution

**Windows Lifecycle Monitor**:
The local Windows session and power observer whose availability is required for retained vault-session access.
_Avoid_: Automatic vault unlock, browser-memory erasure

**Publisher-Signed Installer**:
A distribution executable authenticated under the publisher's trusted signing identity, distinct from an unsigned test installer or a signature on a dependency.
_Avoid_: Security certification, signed loose application assets

**Independent Security Audit**:
An assessment of a specified source revision by a separate reviewer with documented methods, findings and retest evidence.
_Avoid_: Passing project tests, project-authored handoff, automated certification
