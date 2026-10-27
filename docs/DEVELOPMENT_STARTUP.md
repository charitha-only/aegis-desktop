# Owned development startup (v0.26.4)

After installing UI dependencies, run `npm run dev` from the project root.
The CLI builds the debug core with Cargo, reads its executable from Cargo JSON
artifacts (including custom `CARGO_TARGET_DIR`), and launches it with `--port=0`.
It reads the actual loopback address, checks `/handshake` using the intended UI
origin, then starts Vite on `127.0.0.1:5173`. Only after both are ready does it
open an external browser. The selected address overrides stale development env
values for this Vite instance. Production origin pinning remains unchanged.

## Options

```text
node aegis-cli/bin/aegis.js dev --no-open
node aegis-cli/bin/aegis.js dev --core-binary=/absolute/path/to/aegis-core --ui-port=5173
node aegis-cli/bin/aegis.js dev --no-open --print-plan
```

`--core-binary` skips compilation and requires an unpackaged executable; a sibling
`ui/index.html` is rejected. Paths resolve from the invocation directory. The project root is canonicalized
before Vite configuration, so Windows 8.3 aliases do not trigger Vite's intentional
short-path denial. Filesystem strictness and deny rules remain enabled.
`--ui-port` accepts decimal ports 1–65535. Configure the corresponding UI origin
in `security.allowedOrigins` before using a different port. The launcher never
widens origin permissions. Unknown, duplicate or malformed options fail before
launch. `--print-plan` performs no build, binding or desktop launch.

## Readiness and ownership

Core readiness has a 30-second budget and the handshake probe a five-second
budget. An occupied UI port fails instead of selecting another port or stopping
its existing owner. A startup failure closes the newly owned core and Vite.
SIGINT/SIGTERM stop owned development resources; unexpected core exit also closes
the UI and reports failure. Vite configuration restarts are allowed five seconds
to recover a listening server, observed using a monotonic clock. A longer outage
ends the session. Normal browser windows remain under user control.

Only tracked child PIDs are targeted. Windows cleanup targets that child tree;
POSIX children receive their own process group. Termination escalates only for
an owned child that remains alive. This is cooperative development supervision,
not a process sandbox: forced termination of the launcher, already-orphaned
children, malicious project configuration and hanging Vite plugin cleanup are
outside the guarantee. Project config/plugins and the selected binary are trusted.
The readiness response does not authenticate a server against same-account malware.

## Verification

```text
node --test aegis-cli/test/development.test.js
node aegis-cli/bin/dev-smoke.js
```

Fixtures use temporary project data, real core/Vite endpoints and `--no-open`.
They cover Windows short-path project roots, dynamic address injection over a stale env value, UI restart, normal
shutdown, cancellation, unexpected core exit and occupied-port preservation.
Hosted Windows/Linux/macOS CI additionally copies a clean core source fixture
and exercises the default Cargo build with a custom target directory. No local
fixture opens windows, touches the clipboard or locks/suspends the workstation.
These checks do not establish physical-device, accessibility or sensitive-app
production acceptance.
