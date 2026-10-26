# Stable packaged application identity

Set a permanent top-level `appId` in `aegis.config.json`, for example `"appId": "org.example.notes"`. Packaged startup selects the data directory from this ID rather than the executable's parent-folder name. Changing a display name or renaming/moving the package folder then preserves the selected vaults, profiles and native cache root. Development-mode storage remains in the working project directory.

The existing framework demo uses `AegisDesktopApplication`, matching its prior default package-folder identity and preserving the established upgrade data path. Newly created projects receive `app.<lowercase-project-name>`; publishers should choose a unique permanent namespace before first distribution. Keep IDs unique even under case-insensitive comparison. This is an application namespace, not authentication, OS account isolation or a defense against another same-account application choosing the same ID.

IDs use the same portable directory validation as the platform directory interface: 1–128 ASCII bytes, letters/digits/dot/underscore/hyphen/space only, no dot traversal, trailing dot/space or Windows reserved device names. Lowercase reverse-domain-style IDs without spaces are recommended. Explicit null, non-string, empty or invalid values fail packaged startup; they never fall back silently. Bundling rejects an invalid explicit ID before writing output.

## Existing applications

If `appId` is absent, the runtime preserves the legacy package-folder identity. It does not guess a new identity or move data. To adopt a stable ID without moving an existing application's data, set it to the exact prior package-folder name, including case. Users who previously renamed their package may have multiple legacy directories; inspect and authenticate the intended vault before selecting that identity.

Changing an already-distributed `appId` selects another data directory. There is no automatic merge, rename, overwrite or cross-ID migration. Keep the old ID for compatible releases or design a separately reviewed authenticated migration and recovery flow. Do not delete an old directory merely because a newly selected directory appears empty. Signing/notarization and native installer registration identity remain separate distribution concerns.

## Verification

Public platform tests cover stable selection, legacy fallback and rejection of malformed explicit IDs. The CLI test verifies generated source configuration. The portable workflow runs a real native packaged process under isolated temporary HOME/LOCALAPPDATA/XDG roots on Windows, Linux and macOS: write an encrypted vault, stop the owned process, rename its package folder, restart and authenticate the same vault. It also verifies invalid identity exits without starting an IPC server. The existing Windows previous-version installer upgrade test continues to check its established data path and encrypted backup preservation.
