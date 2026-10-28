# Native Debian packaging preview

`package-deb` runs on a native Debian/Ubuntu Linux host with Node 22, dpkg-dev and the runtime libraries used to build the core. It packages an already built portable folder, using a separately retained manifest SHA-256:

```text
node aegis-cli/bin/aegis.js package-deb <bundle-directory> <output.deb> <trusted-manifest-sha256> "Your Name <your-maintainer-email>"
```

Use your real maintainer identity. The output parent must exist; an existing output is never overwritten. This command does not install, request privilege elevation or execute the input core. It verifies the bundle, copies and verifies a staging snapshot, validates native ELF architecture, derives library dependencies with dpkg-shlibdeps and builds with dpkg-deb. Installation is a separate administrator decision.

The exact appId determines a stable package name and `/opt` installation directory. Display-name and version changes retain identity. The desktop entry uses the default external-browser backend with a dynamically selected loopback port. User data is resolved by the runtime outside the installed payload. No maintainer scripts delete user data. File/directory modes are normalized; installed files are root-owned.

The guarded acceptance script runs only on disposable GitHub-hosted Linux runners. It builds two versions of the actual core/UI, installs, upgrades with a changed display name, removes and reinstalls the fixture, then compares encrypted vault and backup bytes and authenticates decryption. It purges only its randomly named fixture. It never opens a desktop window. Local Windows tests cover metadata and rejection of unsupported build hosts; they are not Linux installation evidence.

Packages are unsigned. A pinned digest checks bytes, not publisher identity. Dependencies reflect the build host ABI; this does not promise compatibility with older distributions, other architectures or all Linux desktops. macOS native packaging, signing/notarization, authenticated updates, physical-device recovery and independent review remain separate gates.

Native tools follow the official [dpkg-deb documentation](https://manpages.debian.org/trixie/dpkg/dpkg-deb.1.en.html), [dpkg-shlibdeps documentation](https://manpages.debian.org/trixie/dpkg-dev/dpkg-shlibdeps.1.en.html) and [Debian control-field policy](https://www.debian.org/doc/debian-policy/ch-controlfields).
