# Optional embedded desktop preview

Build with `cargo build --release --manifest-path src-core/Cargo.toml --features native-desktop`, then run the resulting executable with `--native-window`. In development, start the UI development server separately. Packaged mode loads the bundled UI. The existing CLI/bundler builds the smaller default core unless the feature is explicitly selected.

Windows/macOS use Wry without its default Linux feature set, hosted by Winit. Linux uses GTK4/WebKit6 directly. This removes the old GTK3/GLib dependency chain from the resolved graph. The new complete lockfile passed the current RustSec check without ignored advisories or warning exceptions. Dependency scanning does not establish application security or eliminate unknown flaws.

The window loop runs on the main thread; the loopback server runs on a worker thread. The native policy permits only navigation within the initial local HTTP origin and rejects invalid sizes/titles, developer tools and unsupported modal/kiosk/always-on-top/transparent requests. Webview popups and new permission requests are denied. Windows/macOS downloads are denied; Linux cancels session downloads. Ephemeral/incognito browsing is requested without claiming complete memory erasure.

The default browser backend and existing demo UI remain available. This preview does not implement the full multi-window/dialog/file-selection/clipboard/lifecycle interfaces required by v0.26. The existing capability report still describes the external-browser API, rather than claiming those new controls work.

Linux requires GTK4 and WebKitGTK 6 development/runtime libraries. Windows requires WebView2 and an appropriate native build toolchain; this development PC has no MSVC linker, so native-feature compilation is validated in hosted CI. The default GNU core build remains locally testable.

The optional-native workflow compiles/links/tests the feature on all three operating systems without opening a window. Actual rendering, accessibility, keyboard navigation, display/DPI behavior, GUI process cleanup and physical-device lifecycle acceptance remain outstanding; compilation is not a substitute for those tests.

Upstream references: [Wry feature definitions](https://github.com/tauri-apps/wry/blob/dev/Cargo.toml), [Winit](https://docs.rs/winit/latest/winit/), [WebKit6](https://docs.rs/webkit6/latest/webkit6/).
