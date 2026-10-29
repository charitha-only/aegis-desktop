# TypeScript SDK transport preview — v0.29.5

The existing `aegis` singleton remains available. Applications may instead import `AegisBridge` from their generated `src/aegis-bridge` and construct an independent client:

```ts
const client = new AegisBridge({requestTimeoutMs: 30000, maxResponseBytes: 16 * 1024 * 1024});
const response = await client.platformInfo();
if (!response.success && response.errorCode === 'TIMEOUT') {
  // Inspect application state before deciding whether to retry a mutation.
}
```

The default timeout is 15000 ms, configurable to an integer from 1 to 120000 ms. Each handshake and IPC HTTP request has its own deadline through response-body parsing; automatic initialization followed by invocation can therefore consume two deadlines. JavaScript event-loop suspension can delay timers; this is not a real-time execution budget. Invalid constructor settings throw synchronously. Timeouts abort the client fetch, not a server-side operation. No automatic mutation replay or cancellation acknowledgment is provided.

Handshake tokens must match the current core's 32-byte lowercase hex encoding. Failed explicit initialization clears the cached process token. It does not lock an authenticated vault or invalidate credentials held by another client. Concurrent `init` calls and automatic initialization by `invoke` share one pending handshake per client. Calling `init` after initialization starts an explicit refresh and clears cached credentials before subsequent dependent invocations join it. A failed handshake sends no dependent commands; a later call starts a fresh attempt. Separate client instances do not share state. Already-issued commands continue and cannot be revoked by refreshing the process token.

IPC responses must use a successful HTTP status and a JSON envelope with the matching request ID and a boolean success field. Backend failure envelopes require a string error. HTTP 429 remains an HTTP failure with a string message even if its response field is malformed. As of v0.29.5, the covered commands listed below additionally validate runtime data. Other commands remain unvalidated below the envelope; compile-time types do not establish runtime data validity.

Local failures add optional `errorCode` values: `HTTP_ERROR`, `INVALID_RESPONSE`, `NETWORK_ERROR`, `TIMEOUT`, `RESPONSE_TOO_LARGE`. Existing `id`, `success` and human-readable `error` fields remain. Backend authorization/validation errors retain their existing wire contract and do not necessarily include this local field. A timeout or interrupted response can leave operation completion unknown. Error strings are diagnostic text, not stable identifiers or safe HTML.

The transport checks do not authenticate a publisher, identify a user, add replay prevention or sandbox a compromised frontend/core. The process handshake token remains distinct from vault authorization. HTTPS/public-network API use is outside this local transport contract.

Run `node aegis-cli/bin/sdk-contract-smoke.js` after installing UI dependencies. Its real loopback HTTP fixtures load the actual TypeScript client through Vite and test malformed handshake, HTTP status/envelope/JSON rejection, stalled-body deadlines with successful handshake retry, string-valued rate-limit failure handling, declared/chunked response overflow, small-response recovery, separate handshake/rate-limit bounds, shared successful and failed initialization/retry, and invalid byte-budget rejection. `node aegis-cli/bin/sdk-smoke.js` separately tests the SDK against the compiled Rust core. Both run in the three-OS portable workflow; browser engine/physical-device acceptance remains separate.

## Response byte budgets

Handshake and HTTP 429 JSON bodies have a fixed 4096-byte cap. Successful IPC bodies default to 16 MiB; `maxResponseBytes` accepts integers from 1024 through 67108864, otherwise the constructor throws. This default permits the existing storage limit plus JSON escaping overhead. Applications should choose a smaller budget when their commands permit it; the cap applies to the entire JSON envelope, not only its `data` field.

The reader counts actual response stream bytes, including chunked bodies, instead of trusting Content-Length. Browser fetch exposes content-decoded bytes, so this is not a compressed-wire-byte limit. It rejects before retaining a chunk that would exceed the budget, cancels its reader and aborts the client fetch. A geometrically growing buffer avoids retaining an unbounded number of tiny chunk objects. The browser may already have buffered a chunk; decoded strings, parsed objects and temporary buffers consume additional memory. This is a bounded application response reader, not a total process heap limit or protection against an already-compromised process.

Responses beyond a configured budget now fail where older clients might accept them; adjust the budget within the supported range or paginate application results. HTTP failures other than 429 do not parse the response body. A byte-limit failure does not prove a server mutation was cancelled; inspect application state before retrying.

## Covered runtime data shapes

The shared validation path runs in `invoke`, so both high-level helpers and raw calls to these commands receive the same checks:

| Command | Checked successful data |
| --- | --- |
| `platform_info` | Object; supported OS enum; nonempty architecture; boolean embeddedWebview, lifecycleAdapterCompiled, installerAdapterCompiled, osKeyStore, encryptedFileStorage. |
| `platform_directories` | Object with nonempty string data, config and cache fields. Unicode strings are preserved. |
| `get_security_status` | Object with nonempty engine, boolean nonceVerified/sandboxEnforced and an activeCapabilities object with required fsRead/fsWrite/shellExecution/systemInfo/crypto booleans. Optional flags documented by the SecurityStatus interface must be booleans when present. |

Malformed successful data returns `success:false`, the original request ID and `errorCode:INVALID_RESPONSE`, without including that data. Backend failure envelopes remain unchanged and do not undergo successful-data validation. False flags are valid values. No coercion is performed. Unknown additional fields are preserved for additive protocol evolution; validation covers the named interface fields, not arbitrary unknown extensions. Unknown/custom commands retain generic invocation behavior and need application-defined validation.

These are runtime shape checks, not proof that a capability is actually enforced, a directory is safe/authorized/absolute, or a backend is trustworthy. The Rust runtime still authorizes operations and resolves directories. Applications must not derive permissions from a type check or treat data from a compromised core as trusted. Existing integrations returning incomplete shapes for these known commands must match the documented contract; otherwise they now fail closed.

The SDK fault suite has sixteen cases, including malformed platform/directory/security shapes, true/false typing, nested grants, additive fields, Unicode strings and unchanged backend denials. The actual-core fixture calls the platform commands on each OS and checks native directory roots. It also exercises the vault contract described below. Browser engines, physical devices and remaining command schemas require additional acceptance.

## Vault response contracts — v0.29.5

Successful data for these commands is also validated through the shared invoke path:

| Command | Required data |
| --- | --- |
| `vault_encrypt_store` | Nonempty filename/status strings and positive safe-integer envelopeLength. |
| `vault_session_unlock` | A 64-character lowercase-hex sessionToken, positive safe-integer idleTimeoutSeconds and absoluteTimeoutSeconds, with idle no greater than absolute. |
| `vault_session_lock` | Object containing locked exactly true. |
| `vault_session_read`, `vault_decrypt_retrieve` | Nonempty filename, string decryptedContent (including empty string) and integrityVerified exactly true. |

The current core emits these shapes; custom implementations returning malformed fields now receive INVALID_RESPONSE. Successful responses can include additive fields. Error envelopes remain unchanged. Rejected data, including plaintext or malformed session credentials, is not copied into the returned failure or its error message. This does not erase the fetched/parsed bytes from JavaScript/browser memory, retract earlier results or lock the server automatically. A server-side operation may already have happened despite client rejection; do not automatically replay writes or unlocks.

Validation of integrityVerified is validation of the server's acknowledgement, not independent cryptographic verification by the browser. It cannot detect a malicious core that fabricates an otherwise valid result. Vault authorization and encryption remain backend responsibilities. The SDK does not retain returned vault-session tokens or use them as the process handshake nonce; the application owns their handling.

The actual-core fixture writes/decrypts TEST_ONLY content under owned temporary roots, confirms lock acknowledgement and tests retained-session reads plus post-lock rejection when Windows lifecycle/session monitoring is available. On other/unavailable lifecycle environments it requires unlock failure. This is not physical lock/suspend acceptance or enabling retained sessions on unsupported platforms.
