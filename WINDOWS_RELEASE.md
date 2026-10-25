# Windows release signing and installer verification

The runtime uses a hidden top-level Windows window registered for current-session WTS notifications and normal power broadcasts. Every session transition, suspend and resume revokes retained vault sessions. WTS lock/connect state is checked before and after session unlock/read; unknown state is denied. Registration failure or window destruction disables session access. Native callback tests send messages only to a window owned by the test process. They do not actually lock or suspend the developer's computer.

## Publisher signing setup

There is currently no publisher certificate configured. No signed release has been produced by this change.

1. Obtain a publisher code-signing certificate/service with Windows Authenticode support. Never send its private key/password through chat or commit it.
2. For a locally installed CurrentUser/My certificate, run `scripts/sign-windows.ps1 -Path <bundled-core.exe> -Thumbprint <publisher-thumbprint>`. Install Windows SDK SignTool first. The helper requires code-signing usage, a private key, a valid chain and a trusted timestamp.
3. Compile using `scripts/build-installer.ps1 -ExpectedThumbprint <publisher-thumbprint>` with Inno Setup 6 installed. Then sign the generated installer using the signing helper. Verify with `-VerifyOnly`.
4. The manual `Signed Windows installer` workflow supports an exportable PFX through protected `windows-release` environment secrets `AEGIS_SIGNING_PFX_BASE64` and `AEGIS_SIGNING_PFX_PASSWORD`. Restrict that environment to trusted maintainers/revisions and enable approval protection. Hardware-backed/nonexportable certificates need a provider-specific signing integration; do not export them to use this workflow.
5. The workflow fails when credentials are absent and uploads only after core signature verification, installer signing and install/runtime/uninstall checks pass. It does not publish a GitHub Release automatically.

The signed installer covers its embedded UI/config payload. Signing the portable core alone does not authenticate loose UI/config files, so this workflow offers the installer only. The framework auto-updater remains disabled; this is executable signature verification, not an update protocol.

## Installer test scope

Normal CI explicitly compiles an unsigned TEST installer on a disposable GitHub-hosted Windows runner. It installs without elevation into a unique temporary directory, starts the installed executable, verifies the native lifecycle monitor, exercises authenticated sessions, uninstalls, and verifies encrypted vault fixtures are preserved unchanged. It never runs the installer on the developer's live application directory.

This provides disposable-runner install/uninstall coverage, not a pristine Windows image or every supported hardware/power configuration. Before distribution, test signed installation, upgrade, uninstall/reinstall, real Win+L, sleep/resume/hibernate and remote disconnect on supported Windows devices. Confirm the application clears its own UI/plaintext and clipboard; core revocation cannot recall data already sent to a browser.

Primary references: [WTS session notifications](https://learn.microsoft.com/en-us/windows/win32/termserv/wm-wtssession-change), [power broadcast events](https://learn.microsoft.com/en-us/windows/win32/power/wm-powerbroadcast-messages), [SignTool](https://learn.microsoft.com/en-us/windows-hardware/drivers/devtest/signtool).
