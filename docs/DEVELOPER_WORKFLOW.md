# Developer workflow preview — v0.29.7

Create a source project with `node aegis-cli/bin/aegis.js create <new-directory>`.
Inside the generated directory:

1. `npm run setup` installs frontend dependencies.
2. `npm run doctor` reports Node 22+, Cargo, Rust compiler 1.92+, npm, configuration, lockfiles and installed Vite. `--json` is available through the CLI directly. Failed prerequisites exit with status 1. Diagnostics do not install tools, compile, bind ports or open windows.
3. `npm run dev` starts the development session; pass `-- --no-open` to avoid opening a browser.
4. `npm run package` builds the locked release core and frontend, produces the existing native portable layout, writes `aegis-integrity.json` and verifies it immediately. `npm run package -- --print-plan` shows the output and steps without building.

Package output is `dist-bundle/<configured name with spaces removed>`. Existing output is refused before compilation. Preserve or move previous output before rebuilding. Failures can leave partial new output for inspection; the CLI does not silently delete it. Cargo's output directory is explicitly pinned to this project's `src-core/target`; as of v0.29.11 bundling selects the executable reported by the current successful Cargo build, including target-triple subdirectories.

Keep appId stable across releases. The portable package is unsigned. Its SHA-256 manifest detects file changes against a separately trusted digest; it does not authenticate a publisher, sign an installer, apply updates or establish production readiness. Use the separately documented Windows/macOS/Debian packaging and release gates for distribution.

Doctor checks tool versions and portable prerequisites, not actual compilation/linker usability, every native SDK, signing credentials or complete configuration semantics. A successful release build remains necessary. It executes the installed Cargo/npm version commands with bounded timeouts; use trusted developer tools.

The three-OS portable workflow runs both React and Vanilla TypeScript clean generated projects through locked frontend dependency installation, doctor, release Rust/UI compilation and package inventory verification. This adds developer journey evidence, not physical-device, low-resource, native GUI or independent security-audit evidence.

## Starter selection — v0.29.6

`node aegis-cli/bin/aegis.js create MyApp` retains the React default. Select it explicitly with `--template=react`, or use `node aegis-cli/bin/aegis.js create SmallApp --template=vanilla` for the DOM-based TypeScript starter. The destination comes first; one optional template argument follows. Unknown or repeated options are rejected before creating output.

Both templates include the shared typed SDK, runtime data validation, development flow, Rust core, stable appId configuration and portable packaging tooling. The vanilla starter creates DOM nodes with textContent, a status region and a connection button; it does not bundle a React renderer. Its Vite configuration does not enable the React plugin. Both templates keep the existing shared dependency lockfile, including React development packages; this is not a zero-dependency installation or a separate dependency-minimized toolchain.

The clean-project acceptance fixture builds and packages both templates on each advertised hosted OS, verifies the portable file inventory and checks the stock vanilla output's total raw JavaScript assets against an explicit 65536-byte budget. Results in logs are fixture measurements, not application runtime RAM/CPU/startup guarantees. User code can exceed the budget; representative low-resource devices and whole-process/UI benchmarking remain release gates.

The generator now parses source configuration and requires object-valued window settings before creating output. This prevents malformed-source-config failures from publishing an incomplete project. It does not provide atomic creation or cleanup guarantees for arbitrary copy/write/disk-full/interruption failures. Preserve incomplete output for inspection and choose a new destination before retrying. Existing output is never overwritten.

## Toolchain selection and release plans — v0.29.7

Use `npm run build -- --print-plan`, `npm run build:release -- --print-plan` or `npm run bundle -- --print-plan` to inspect the exact executable, argument arrays and working directories without running tools, compiling, binding ports or writing build output. Plans can be inspected without Cargo on PATH, but the active Node installation must include a discoverable npm-cli.js. A plan is a description, not a promise the toolchain can build. Commands reject unknown extra options instead of silently starting a build.

Release builds use `--locked` and explicitly target the canonical project's src-core/target. The target-dir override prevents CARGO_TARGET_DIR redirection, but did not handle target-triple subdirectories caused by build.target/CARGO_BUILD_TARGET; v0.29.11 corrects that gap using compiler-artifact selection. Debug development instead uses Cargo's compiler-artifact output to locate its executable and continues to support its selected target directory.

Build, bundle, package and doctor resolve npm-cli.js from the running Node installation, then invoke it with process.execPath and argument arrays. The CLI no longer selects npm.cmd from PATH or starts npm through a command shell. npm package scripts retain their normal script-shell behavior; this is not a shell-free process-tree guarantee. Source projects with custom Node distributions that do not include npm beside the runtime must install a supported Node/npm distribution; the CLI fails with an actionable message rather than silently using a different npm.

Cargo/rustc prefer an existing regular tool file under CARGO_HOME/bin, or the user's default .cargo/bin when CARGO_HOME is unset; otherwise they use command-name lookup. Relative CARGO_HOME is resolved in project context. Development uses the same Cargo resolver. Developer tools are trusted local programs, not sandboxed or publisher-authenticated by this resolution. PATH fallback can still select installed Rust tools; inspect the tools field in doctor --json. Version diagnostics run in the target project directory so project toolchain selection is respected. The compiler minimum check follows the current core's Rust 1.92 floor; it does not establish linker, native SDK or future-toolchain compatibility.

The five public regressions inspect release commands with an empty PATH, compare doctor's selected tools to the build plan and verify missing Rust tools produce JSON diagnostic failures while Node-owned npm remains usable. A real Rustup proxy under owned temporary roots also proves diagnostics do not request a toolchain distribution when the selected toolchain is absent. Actual execution is covered separately by both-template three-OS packaging, CLI bundling and owned development startup/restart/cleanup fixtures. Release-build process-tree cancellation and arbitrary filesystem/OS durability remain separate concerns.

Diagnostic version queries explicitly set `RUSTUP_AUTO_INSTALL=0`, using the [official Rustup environment control](https://rust-lang.github.io/rustup/environment-variables.html), so a missing selected toolchain is reported instead of implicitly installed. Build commands retain the caller’s normal tool behavior. Version checks still execute trusted local programs; they are not a sandbox preventing arbitrary wrapper side effects.


## Release artifact selection — v0.29.11

Build/build-release/bundle/package capture Cargo JSON compiler-artifact events and require an absolute executable for the aegis-core binary target from that successful build. Bundle execution uses this artifact rather than a predicted default path. Compiler errors or a missing core artifact stop the flow; there is no stale-path fallback. Normal Cargo freshness checks may reuse an unchanged artifact and still report it.

Read-only plans remain tool-execution-free. coreExecutable is the default predicted path; artifactSelection=cargo-compiler-artifact and captureCoreArtifact/useCoreArtifact command flags describe how execution resolves the actual path. It cannot know custom target outputs without building. A host-target environment/configuration override is covered by a real dependency-free Cargo fixture with a deliberately stale default binary. This does not add foreign-target packaging: foreign binaries can fail to execute, while trusted compiler/build-script side effects and process-tree cancellation remain outside this guarantee.
