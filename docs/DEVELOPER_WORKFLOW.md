# Developer workflow preview — v0.29.6

Create a source project with `node aegis-cli/bin/aegis.js create <new-directory>`.
Inside the generated directory:

1. `npm run setup` installs frontend dependencies.
2. `npm run doctor` reports Node 22+, Cargo, npm, configuration, lockfiles and installed Vite. `--json` is available through the CLI directly. Failed prerequisites exit with status 1. Diagnostics do not install tools, compile, bind ports or open windows.
3. `npm run dev` starts the development session; pass `-- --no-open` to avoid opening a browser.
4. `npm run package` builds the locked release core and frontend, produces the existing native portable layout, writes `aegis-integrity.json` and verifies it immediately. `npm run package -- --print-plan` shows the output and steps without building.

Package output is `dist-bundle/<configured name with spaces removed>`. Existing output is refused before compilation. Preserve or move previous output before rebuilding. Failures can leave partial new output for inspection; the CLI does not silently delete it. Cargo's output directory is explicitly pinned to this project's `src-core/target` so redirected Cargo output cannot select an old local executable.

Keep appId stable across releases. The portable package is unsigned. Its SHA-256 manifest detects file changes against a separately trusted digest; it does not authenticate a publisher, sign an installer, apply updates or establish production readiness. Use the separately documented Windows/macOS/Debian packaging and release gates for distribution.

Doctor checks portable prerequisites, not compiler/linker usability, every native SDK, signing credentials or complete configuration semantics. A successful release build remains necessary. It executes the installed Cargo/npm version commands with bounded timeouts; use trusted developer tools.

The three-OS portable workflow runs both React and Vanilla TypeScript clean generated projects through locked frontend dependency installation, doctor, release Rust/UI compilation and package inventory verification. This adds developer journey evidence, not physical-device, low-resource, native GUI or independent security-audit evidence.

## Starter selection — v0.29.6

`node aegis-cli/bin/aegis.js create MyApp` retains the React default. Select it explicitly with `--template=react`, or use `node aegis-cli/bin/aegis.js create SmallApp --template=vanilla` for the DOM-based TypeScript starter. The destination comes first; one optional template argument follows. Unknown or repeated options are rejected before creating output.

Both templates include the shared typed SDK, runtime data validation, development flow, Rust core, stable appId configuration and portable packaging tooling. The vanilla starter creates DOM nodes with textContent, a status region and a connection button; it does not bundle a React renderer. Its Vite configuration does not enable the React plugin. Both templates keep the existing shared dependency lockfile, including React development packages; this is not a zero-dependency installation or a separate dependency-minimized toolchain.

The clean-project acceptance fixture builds and packages both templates on each advertised hosted OS, verifies the portable file inventory and checks the stock vanilla output's total raw JavaScript assets against an explicit 65536-byte budget. Results in logs are fixture measurements, not application runtime RAM/CPU/startup guarantees. User code can exceed the budget; representative low-resource devices and whole-process/UI benchmarking remain release gates.

The generator now parses source configuration and requires object-valued window settings before creating output. This prevents malformed-source-config failures from publishing an incomplete project. It does not provide atomic creation or cleanup guarantees for arbitrary copy/write/disk-full/interruption failures. Preserve incomplete output for inspection and choose a new destination before retrying. Existing output is never overwritten.
