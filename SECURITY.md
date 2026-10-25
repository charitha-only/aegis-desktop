# 🛡️ Security Policy

The Aegis Desktop project takes the security of our framework, users, and downstream applications seriously.

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability, capability bypass, or path traversal loophole within Aegis Desktop, please do **NOT** open a public issue.

Instead, please report the vulnerability privately:
1. **GitHub Security Advisory:** Submit a private advisory via the **Security** tab of this repository.
2. **Email Disclosure:** Contact the core maintainers directly with full reproduction details.

### What to Include in Your Report:
- A clear description of the vulnerability and attack vector (e.g., IPC manipulation, Nonce bypass, Path traversal).
- Step-by-step reproduction instructions or a minimal Proof of Concept (PoC).
- Potential impact on the host operating system or isolated sandbox.
- Proposed patch or mitigation (if available).

## Security Architecture Principles
Aegis adheres to strict defense-in-depth security principles:
- **Zero-Trust Capabilities:** Explicit permissions must be declared in `aegis.config.json`.
- **Strict Nonce Authentication:** All IPC calls require a negotiated 256-bit cryptographic token.
- **Path Traversal Defense:** All paths are canonicalized against the `./sandbox` root directory.
- **Revoked Shell Execution:** Arbitrary command line executions are strictly prohibited by default policy.
