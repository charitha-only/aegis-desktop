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

CI obtains Inno Setup 6.7.3 from the immutable official `jrsoftware/issrc` release asset, verifies its pinned SHA-256 digest and publisher Authenticode signature, and installs the compiler only in the disposable runner. Compiler setup is refused outside that environment.


## v0.23 additional acceptance coverage

CI builds the actual v0.22 baseline at commit `49b332c3f8c286f4cfef50e2d383aa568c734985`, installs it, creates authenticated encrypted current/backup generations, upgrades to the current build, and checks byte preservation before authenticating the migrated vault and restoring its backup. It also verifies uninstall/reinstall preservation and reads the retained data after reinstall. Every installer check runs only in a disposable GitHub-hosted runner with unique directories.

A final explicitly enabled CI step calls real Windows `LockWorkStation` in that disposable runner only. It verifies current WTS lock state, rejects retained-token reads and fresh session unlock while locked, and checks ephemeral secret cleanup. It is refused on the developer PC and on self-hosted runners. The runner can remain locked until disposal; no unlock credentials are needed or stored. This is real workstation-lock coverage, not a hardware sleep/resume/hibernate test.

Sanitized installer and lock evidence JSON files are uploaded as `windows-validation-evidence`, recording the exact Git commit and distinguishing tests actually run from pending external audit, publisher signing and real suspend/resume. They contain no passwords, tokens or vault plaintext. Evidence is not a certificate of security. Signed builds are restricted to main, run Rust tests before certificate import, and use actions pinned to exact commits. Protect the release environment as described above; pinning does not replace its access controls.
