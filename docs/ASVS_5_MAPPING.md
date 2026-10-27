# ASVS 5.0.0: scoped framework evidence

Baseline: [OWASP ASVS tag v5.0.0](https://github.com/OWASP/ASVS/tree/v5.0.0/5.0),
[official English requirements JSON](https://raw.githubusercontent.com/OWASP/ASVS/v5.0.0/5.0/docs_en/OWASP_Application_Security_Verification_Standard_5.0.0_en.json).
Recorded 2026-10-28. Requirement IDs below refer to that pinned version.
Descriptions are project summaries, not a reproduced standard.

This initial mapping covers selected browser/IPC controls in Aegis. It is not a
full ASVS assessment, a level claim, a certification or an independent audit.
Unlisted requirements remain unassessed. An application built on Aegis needs its
own scope, threat model and evidence. Do not count unassessed items as passing.

## Status vocabulary

- **Implemented/tested scope**: the named framework behavior has regression evidence;
  it does not mean every application use of that requirement passes.
- **Partial**: some evidence exists; a material gap remains.
- **Unassessed**: no complete requirement-level assessment yet.
- **Not applicable in this scope**: feature absent here; reassess if an app adds it.

## Initial evidence matrix

| Pinned requirement | Scope / status | Evidence and remaining work |
| --- | --- | --- |
| v5.0.0-3.4.4 | MIME sniffing protection: implemented/tested scope | `src-core/src/server.rs` has one response boundary. `portable-smoke.js` checks handshake, asset/root requests, denied origin, preflight, unsupported body type and missing route. CI repeats against raw and packaged cores. |
| v5.0.0-3.4.5 | Referrer suppression: implemented/tested scope | Same boundary sets `Referrer-Policy: no-referrer`, including error paths. The regression failed on an absent header before remediation. Browser-network enforcement itself is not exercised by this headless HTTP fixture. |
| v5.0.0-3.4.6 | Framing restriction: implemented/tested header scope | Every core response carries a separate CSP with `frame-ancestors 'none'`. This intersects with the app asset policy. Interactive hostile-frame acceptance remains unperformed. |
| v5.0.0-3.4.3 | Content policy: partial | Separate baseline includes `object-src 'none'` and `base-uri 'none'`; packaged assets retain configured CSP. Existing policy permits inline content and broad resource sources. A trusted-resource allowlist/nonces/hashes assessment, app compatibility and L3 per-response policy remain open. |
| v5.0.0-4.1.1 | Response media types: partial | Handshake/IPC and JSON error branches have JSON media types; known packaged assets have mapped types. Complete media-type/charset coverage of arbitrary application assets remains unassessed. |
| v5.0.0-3.5.2 | Cross-origin requests: partial | Exact Host, configured/exact-app Origin, JSON-only IPC and token checks; denied-origin regression in `portable-smoke.js`. Missing Origin is allowed for native clients. Process token is not user authentication; application CSRF scope and all browser request contexts need separate assessment. |
| v5.0.0-14.3.2 | Sensitive response caching: partial | Handshake/IPC responses already set `Cache-Control: no-store`. Coverage must be extended when apps add sensitive routes; browser cache acceptance is not established by header inspection alone. |
| v5.0.0-2.2.1 and v5.0.0-2.2.2 | Trusted input validation: partial | Server content/body limits and Rust command validation; window, filesystem and vault regressions. A command-by-command validation inventory remains open. |
| v5.0.0-1.2.5 | OS command inputs: partial | Arbitrary shell IPC rejected; native helper arguments are bounded and validated. Helper-specific regression coverage exists; complete command invocation review and app/plugin code assessment remain open. |
| v5.0.0-1.5.2 | Deserialization: partial | Typed Rust IPC envelopes and bounded JSON input. A complete payload/schema and duplicate-field interoperability review remains open. |
| v5.0.0-14.3.1 | Client secret removal: unassessed | Framework lock clears retained backend credentials; plaintext already returned to UI cannot be retracted. App DOM/storage/session-lifecycle acceptance remains required. |
| Database injection, LDAP/JNDI and mail controls | Not applicable to current built-in service scope | No built-in database/LDAP/JNDI/mail service. Third-party/native plugins are trusted code and are not covered by this exclusion. Reassess per application. |

## v0.26.5 response contract

Core HTTP replies, including rejection/preflight paths, use a common responder.
It sets `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, and an
additional `Content-Security-Policy` containing `frame-ancestors 'none';
object-src 'none'; base-uri 'none'`. Multiple CSP policies are intentionally
restrictive together; custom asset policy cannot relax this baseline. Apps that
require embedding or an HTML base element must redesign that flow rather than
assume it is permitted. JSON errors carry an explicit JSON content type.

The public integration fixture observes status/headers over a real owned core;
portable packaging CI exercises the same fixture against the bundled runtime.
It uses no user browser, windows, clipboard or workstation lock/suspend.
Exact source revision and three-OS results are tracked in
[IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md). Run locally after building:

```text
cargo build --release --locked --manifest-path src-core/Cargo.toml
node aegis-cli/bin/portable-smoke.js
npm --prefix src-ui run build
node aegis-cli/bin/package-smoke.js
```

## Next assessment slices

1. Inventory every IPC command's input rules, capability and vault-session boundary.
2. Review CSP resources/inline dependencies and test actual browser framing and
   navigation without disrupting developer desktop sessions.
3. Map authentication, cryptography and key lifecycle requirement by requirement;
   process handshake tokens must never be recorded as user authentication.
4. Map dependency/distribution/update trust and retain failed/retest evidence.

Desktop additions remain in [RELEASE_GATES.md](RELEASE_GATES.md): OS lifecycle,
key memory, installers/signing/notarization, accessibility, low-resource devices,
and independent review are not completed by this web standard mapping.
