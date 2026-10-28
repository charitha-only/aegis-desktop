# Native macOS packaging preview

On a native macOS build host with Node 22 and Apple's command-line tools:

```text
node aegis-cli/bin/aegis.js package-macos <bundle-directory> <output.pkg> <trusted-manifest-sha256>
```

Build the portable core and UI first. Generate and retain the manifest digest separately using `package-manifest`. The output parent must exist and be outside the input bundle; an existing output is never overwritten. This builder verifies both source and copied snapshot, checks the core has the host's single native architecture and only system dynamic-library dependencies, then invokes native `pkgbuild`. It does not install, elevate or execute the supplied core. The config version must be numeric `major.minor.patch` (up to four/two/two digits); prerelease strings are rejected.

The exact appId determines the stable bundle/receipt identifier and `/Applications/Aegis-<hash>.app` path. Human-readable names are escaped into Info.plist. The package fixes the install destination and replaces the previous bundle during upgrades rather than following relocated copies or merging obsolete assets. Runtime data stays in the OS app-data location, outside the application. No installer scripts remove user data. The launcher starts the default external-browser backend with a dynamically selected loopback port; optional embedded-native acceptance is separate.

The guarded smoke test only performs privileged installation on disposable GitHub-hosted macOS runners. It checks installed receipt versions, actual headless core IPC, a renamed upgrade, app removal plus receipt removal, reinstall and unchanged encrypted vault/backup bytes with authenticated decryption. It inspects the launcher without opening a window. Removing a receipt alone does not remove an installed app. The fixture removes only its random app directory and receipt, preserving its user data until acceptance finishes.

These are unsigned preview packages. There is no Developer ID signature, notarization, Gatekeeper/Finder launch acceptance, universal binary or older-OS compatibility claim. The digest is integrity evidence, not publisher authentication. Physical-device recovery, authenticated updates and independent audit remain separate gates. Consult [Apple's bundle metadata reference](https://developer.apple.com/library/archive/documentation/General/Reference/InfoPlistKeyReference/Articles/CoreFoundationKeys.html) and native `man pkgbuild`, `man installer` and `man pkgutil` on the build host.

Native execution evidence: source `65ff429` passed the hosted macOS package lifecycle fixture in portable workflow `40779616646`. The fixture checks receipt versions 1.0.0/2.0.0, launcher inspection, retained ciphertext/backup bytes and current-vault decryption. It does not simulate power loss or authenticate a backup restore.
