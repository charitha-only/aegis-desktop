# Optional embedded desktop preview

Build with `cargo build --release --manifest-path src-core/Cargo.toml --features native-desktop`, then run the resulting executable with `--native-window`. In development, start the UI development server separately. Packaged mode loads the bundled UI. The existing CLI/bundler builds the smaller default core unless the feature is explicitly selected.

Windows/macOS use Wry without its default Linux feature set, hosted by Winit. Linux uses GTK4/WebKit6 directly. This removes the old GTK3/GLib dependency chain from the resolved graph. The new complete lockfile passed the current RustSec check without ignored advisories or warning exceptions. Dependency scanning does not establish application security or eliminate unknown flaws.

The window loop runs on the main thread; the loopback server runs on a worker thread. The native policy permits only navigation within the initial local HTTP origin and rejects invalid sizes/titles, developer tools and unsupported modal/kiosk/always-on-top/transparent requests. Webview popups and new permission requests are denied. Windows/macOS downloads are denied; Linux cancels session downloads. Ephemeral/incognito browsing is requested without claiming complete memory erasure.

The default browser backend and existing demo UI remain available. This preview does not implement the full multi-window/dialog/file-selection/clipboard/lifecycle interfaces required by v0.26. The existing capability report still describes the external-browser API, rather than claiming those new controls work.

Linux requires GTK 4.10 or newer and WebKitGTK 6 development/runtime libraries. Windows requires WebView2 and an appropriate native build toolchain; this development PC has no MSVC linker, so native-feature compilation is validated in hosted CI. The default GNU core build remains locally testable.

The optional-native workflow compiles/links/tests the feature on all three operating systems, then runs a disposable fixture-window test that checks the expected document title and process exit. Linux uses a virtual display. The fixture refuses to run on this developer PC. Check the exact-commit CI result before claiming that GUI check passed. This is not visual rendering, accessibility, keyboard navigation, display/DPI or physical-device lifecycle acceptance.

Upstream references: [Wry feature definitions](https://github.com/tauri-apps/wry/blob/dev/Cargo.toml), [Winit](https://docs.rs/winit/latest/winit/), [WebKit6](https://docs.rs/webkit6/latest/webkit6/).


Hosted Linux smoke setup includes a temporary AppArmor profile granting user-namespace creation only to `/tmp/aegis-native-*/NativeFixture/aegis-core`. Ubuntu's global namespace restriction is not disabled, and WebKit's sandbox is not disabled. This is a disposable CI prerequisite, not a production confinement profile. The profile is removed in cleanup. Production Linux distribution needs an appropriate supported namespace/sandbox policy; see [Ubuntu's namespace documentation](https://documentation.ubuntu.com/security/security-features/privilege-restriction/apparmor/).

Windows WebView2's cache is directed to the application's data root. The native host releases its webview before its parent window; the smoke harness retries owned fixture cleanup briefly while asynchronous browser processes release handles. This is not a guarantee of complete browser-memory erasure. No user browser processes are killed by the harness.


## Dynamic-port startup (v0.26.2)

For packaged applications, `--port=0` binds an OS-selected loopback port and both
native and external startup use that actual bound origin. Previously the UI URL
incorrectly retained port zero. Development continues to use the separate
`http://localhost:5173` UI server. The manually started development bridge defaults to core port 48291; v0.26.3 adds explicit
`VITE_AEGIS_BRIDGE_URL` selection (see DEVELOPMENT_BRIDGE.md). Inspecting a
different/dynamic development listener does not automatically configure that
value. Dynamic packaged startup is the behavior covered by the v0.26.2 fix.

`--print-launch-plan` prints an `AEGIS_LAUNCH_PLAN=` JSON line with `bridgeUrl`,
`uiUrl`, packaged mode, whether native mode was requested and whether the native
feature was compiled. It initializes the ordinary startup configuration/data
location and temporarily binds the listener, then exits before any desktop
launch or IPC request loop. It is not a side-effect-free configuration parser,
and does not include the bearer token. Native availability is reported separately
from requesting it. Use it to diagnose startup addresses, not as a port reservation
for a later process. Do not combine this inspection flag with action flags such as
`--bundle` or helper modes, which retain their existing early-dispatch behavior.

After building the default optimized core, run
`node aegis-cli/bin/launch-plan-smoke.js` for a temporary, owned packaged and
development fixture without desktop interaction. The three-OS guarded native GUI
fixture now uses `--port=0` directly rather than reserving and releasing a port;
it must load the packaged document and exit successfully. This removes the
reserve/rebind race and exercises the dynamic native startup path on disposable
hosted runners. It does not establish physical-device or accessibility acceptance.

The v0.26.4 combined `aegis dev` command selects and supervises an owned core
and Vite server automatically; see [DEVELOPMENT_STARTUP.md](DEVELOPMENT_STARTUP.md).
It opens an external browser and does not implement managed native multi-window.
