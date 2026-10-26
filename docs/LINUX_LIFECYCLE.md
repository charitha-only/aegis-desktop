# Linux lifecycle observer preview

The optional `linux-lifecycle` Cargo feature observes systemd-logind signals and revokes process-local vault sessions and ephemeral encrypted secrets. It does **not** authorize Linux retained vault sessions. Linux and macOS production routers still require an unavailable authorizing monitor and deny retained-session unlock/read; fresh-passphrase vault operations remain available.

## Contract

- Production connects only to `unix:path=/run/dbus/system_bus_socket`, ignoring session/system bus address environment overrides. Registration requires the current `org.freedesktop.login1` owner to have Unix UID 0.
- The subscription pins that unique owner and the `/org/freedesktop/login1` path namespace. All matching signals revoke conservatively, including session lock/unlock, property changes, sleep/resume and shutdown. Events concerning another session can also revoke; this is intentionally a global revoke-only preview.
- Registration and termination revoke. A lost bus, failed owner query or changed service owner terminates observation and revokes. There is no automatic reconnection, token resurrection or monitor-authority grant.
- Queue capacity is 32 messages. A one-second timer checks owner continuity and whether the owning session manager still exists. Production bus connection and method calls have two-second deadlines. These bounds do not promise a hard real-time deadline while the process/OS is stalled or suspended.
- No lock, unlock, suspend, inhibit or shutdown methods are called. The observer only reads bus ownership and subscribes to signals.
- Default builds omit this feature and its runtime overhead. Optional builds use `cargo build --locked --manifest-path src-core/Cargo.toml --features linux-lifecycle`. Native adapters may combine features with `native-desktop,file-dialog,linux-lifecycle`.

## Verification

Portable session tests exercise revocation and prove the observer cannot mark the authorizing platform monitor active. Linux protocol acceptance runs under an owned `dbus-run-session` on a disposable hosted runner. A controlled service exercises lock/unlock, prepare-for-sleep/resume, shutdown, wrong service UID, an impostor signal sender and service-name loss. No real desktop is locked or suspended. The fixture is ignored by default and requires explicit hosted-runner/private-bus guards.

The D-Bus fixture validates protocol handling, not physical sleep/resume, lock-screen behavior or zeroization of plaintext previously returned to an application.

## Gates before enabling retained sessions

Resolve and validate the actual PID-bound local graphical session and its UID. Check active/locked state before and after sensitive operations without trusting stale cached events. Handle suspend/resume ordering, service replacement, session removal and monitor stalls. Validate these paths on actual GNOME/KDE X11/Wayland systems, including processes outside a logind session, unsupported/non-systemd distributions and disconnect/restart failures. Physical acceptance and independent review remain required.

Reference: [systemd-logind D-Bus contract](https://www.freedesktop.org/software/systemd/man/latest/org.freedesktop.login1.html). `PreparingForSleep` and `PreparingForShutdown` do not emit property-change signals, so the observer subscribes to manager signals rather than relying on property updates alone.
