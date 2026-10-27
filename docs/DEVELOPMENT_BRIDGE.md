# Selecting a development core (v0.26.3)

The development SDK uses `http://127.0.0.1:48291` when no override is supplied.
Set `VITE_AEGIS_BRIDGE_URL` before starting Vite to point it at a separately
started core. This value is public development configuration, not a secret.

For example, start the core in one terminal:

```text
cargo run --manifest-path src-core/Cargo.toml -- --port=49321
```

Then start the UI in another terminal. PowerShell:

```powershell
$env:VITE_AEGIS_BRIDGE_URL = 'http://127.0.0.1:49321'
npm --prefix src-ui run dev
```

Linux/macOS shells:

```sh
VITE_AEGIS_BRIDGE_URL=http://127.0.0.1:49321 npm --prefix src-ui run dev
```

The default `aegis dev` launcher now builds an owned development core, selects a
free core port and passes its actual address to Vite automatically. See
[DEVELOPMENT_STARTUP.md](DEVELOPMENT_STARTUP.md). The manual commands above remain
available. Keep the manual UI on the configured development port 5173; origin
policy still applies. For a manually started `--port=0` core, read its actual
address from startup output and set the same value before starting the UI.

## Validation and packaged behavior

Only literal `http://127.0.0.1:<port>` origins are accepted, with an explicit
decimal port from 1 to 65535 and an optional single trailing slash. Invalid
types, empty values, port zero/overflow, leading-zero ports, hostname aliases,
remote hosts, HTTPS, credentials, paths, query strings and fragments fail before
an IPC fetch. Using the literal IPv4 address also matches the core's exact Host
policy. Configuration validity does not guarantee that the selected OS port can
be bound or that a server there is the expected application.

Packaged clients use their own validated application origin. They ignore
`VITE_AEGIS_BRIDGE_URL`, including values baked into a production build. Invalid
configuration returns a normal unsuccessful `init`/`invoke` result instead of
throwing during SDK module initialization. Custom-nonce invocation performs the
same endpoint validation; it cannot bypass the check.

Endpoint selection is not server identity authentication. The process handshake
token remains a bearer credential, separate from vault/user authentication.
This change does not introduce TLS, app-instance attestation, network timeouts,
automatic retries or protection against a malicious same-account process.

## Verification

```text
node --test aegis-cli/test/bridge-origin.test.js
node aegis-cli/bin/sdk-smoke.js
npm --prefix src-ui run build
```

The integration fixture requires the built release core and installed UI
dependencies. It loads the actual TypeScript SDK through Vite's module runner
and calls an owned dynamic-port native core, then checks controlled rejection of
an invalid endpoint for both initialization and custom-nonce invocation. It uses
no browser, Vite HTTP listener, HMR WebSocket listener or user application data.
Portable CI repeats the fixture on Windows, Linux and macOS. Public resolver
tests first reproduced the ignored development override and acceptance of invalid
destinations; packaged-origin pinning is covered separately.
