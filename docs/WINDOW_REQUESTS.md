# Window launch requests (v0.26.1)

`window_spawn` requests an external browser launch on Windows, Linux or macOS,
including when the primary application uses the optional native host. It does
not create a managed secondary native window. Managed enumeration, closing,
modal parenting and lifetime tracking remain release gates.

## Input validation

The payload must be an object. Omitted URL/title/size retain the compatibility
defaults `http://localhost:5173`, `Aegis Secondary Window`, 800 by 600. Explicit
nulls, wrong types, unknown fields, negative/fractional dimensions and numbers
outside u32 are rejected rather than silently defaulted or truncated. Dimensions
must also be between 1 and 16384 inclusive. Titles are at most 1024 UTF-8 bytes
and cannot contain NUL. URLs are at most 8192 bytes and retain the local-HTTP,
explicit-port and character restrictions.

The supported option names are `url`, `title`, `width`, `height`, `resizable`,
`modal`, `always_on_top`, `kiosk`, `decorations`, `transparent`,
`devtools_enabled`, `custom_protocols`. The camel-case aliases `alwaysOnTop`,
`devtoolsEnabled` and `customProtocols` are also parsed; sending both spellings
of a field is rejected. Omitted `resizable` consistently defaults to true.

Unsupported modal/kiosk/topmost/transparency/devtools/protocol requests fail.
External launches also reject `resizable: false` and `decorations: false`, since
those controls cannot be enforced. The primary native host still supports its
documented resize/decorations configuration.

## What a success means

Success means the OS accepted a browser-launch process request. It does not prove
the requested UI became visible or finished loading. Responses explicitly state
`backend: "EXTERNAL_BROWSER"`, `configurationApplied: false` and
`windowVisibilityConfirmed: false`. Legacy title/width/height fields describe
requested values; the external launcher does not enforce those settings. The
legacy `windowId` is not a unique managed native-window handle. The `runtime`
field describes the primary runtime, which may be embedded, rather than changing
the secondary launch's explicit external backend.

The demo uses its actual current origin for secondary launches instead of the
development server's fixed port, and reports a launch request in its log.

## Verification

```text
cargo test --locked --manifest-path src-core/Cargo.toml window
node aegis-cli/bin/portable-smoke.js
npm --prefix src-ui run build
```

Pure public preparation/validation tests never open a desktop window. IPC
malformed-input regressions use a blocked remote URL, ensuring that even the old
parser cannot launch anything during the failing test. The native-binary smoke
repeats that rejection on all three OS runners. Native-host GUI fixtures remain
guarded for disposable hosted CI; real-device/accessibility and complete managed
multi-window acceptance remain pending.
