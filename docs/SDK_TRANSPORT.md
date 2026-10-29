# TypeScript SDK transport preview — v0.29.2

The existing `aegis` singleton remains available. Applications may instead import `AegisBridge` from their generated `src/aegis-bridge` and construct an independent client:

```ts
const client = new AegisBridge({requestTimeoutMs: 30000});
const response = await client.platformInfo();
if (!response.success && response.errorCode === 'TIMEOUT') {
  // Inspect application state before deciding whether to retry a mutation.
}
```

The default timeout is 15000 ms, configurable to an integer from 1 to 120000 ms. Each handshake and IPC HTTP request has its own deadline through response-body parsing; automatic initialization followed by invocation can therefore consume two deadlines. JavaScript event-loop suspension can delay timers; this is not a real-time execution budget. Invalid constructor settings throw synchronously. Timeouts abort the client fetch, not a server-side operation. No automatic mutation replay or cancellation acknowledgment is provided.

Handshake tokens must match the current core's 32-byte lowercase hex encoding. Failed explicit initialization clears the cached process token. It does not lock an authenticated vault or invalidate credentials held by another client. Concurrent initialization is not serialized; applications should initialize once before concurrent calls.

IPC responses must use a successful HTTP status and a JSON envelope with the matching request ID and a boolean success field. Backend failure envelopes require a string error. HTTP 429 remains an HTTP failure with a string message even if its response field is malformed. Nested `data` values are not validated against TypeScript types; compile-time types do not establish runtime data validity.

Local failures add optional `errorCode` values: `HTTP_ERROR`, `INVALID_RESPONSE`, `NETWORK_ERROR`, `TIMEOUT`. Existing `id`, `success` and human-readable `error` fields remain. Backend authorization/validation errors retain their existing wire contract and do not necessarily include this local field. A timeout or interrupted response can leave operation completion unknown. Error strings are diagnostic text, not stable identifiers or safe HTML.

The transport checks do not authenticate a publisher, identify a user, add replay prevention, impose response-byte limits or sandbox a compromised frontend/core. The process handshake token remains distinct from vault authorization. HTTPS/public-network API use is outside this local transport contract.

Run `node aegis-cli/bin/sdk-contract-smoke.js` after installing UI dependencies. Its real loopback HTTP fixtures load the actual TypeScript client through Vite and test malformed handshake, HTTP status/envelope/JSON rejection, stalled-body deadlines with successful handshake retry and string-valued rate-limit failure handling. `node aegis-cli/bin/sdk-smoke.js` separately tests the SDK against the compiled Rust core. Both run in the three-OS portable workflow; browser engine/physical-device acceptance remains separate.
