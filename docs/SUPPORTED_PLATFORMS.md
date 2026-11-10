# 💻 Supported Platforms Matrix (v1.0.0 GA)

The Aegis Desktop Framework is engineered for high performance, zero-trust security, and native platform integration. Below is the official support matrix for General Availability (GA).

---

## 1. Operating System Support Matrix

| Operating System | Architecture | Minimum Version | Tested & Verified Baseline | Packaging Format |
| :--- | :---: | :---: | :---: | :---: |
| **Microsoft Windows** | x86_64 (`x64`) | Windows 10 (Build 19041+) | Windows 10 Pro / Windows 11 Enterprise | Inno Setup Installer (`.exe`), Standalone Zip (`.zip`) |
| **Linux Distributions** | x86_64 (`amd64`) | Ubuntu 22.04 LTS / Debian 12 | Ubuntu 24.04 LTS / Debian 12 Bookworm | Debian Package (`.deb`), Standalone Tarball (`.tar.gz`) |
| **Apple macOS** | Apple Silicon (`aarch64`) & Intel (`x86_64`) | macOS 12 (Monterey) | macOS 14 (Sonoma) / macOS 15 (Sequoia) | Universal Installer (`.pkg`), Standalone Tarball (`.tar.gz`) |

---

## 2. Desktop Webview & Display Subsystems

| Platform | Webview Runtime | Display Server / Framework | Native Dialogs / Helpers |
| :--- | :--- | :--- | :--- |
| **Windows** | Microsoft Edge WebView2 (Evergreen) | Win32 API / Winit | Windows Shell Common Dialogs / WinRT Notifications |
| **Linux** | WebKitGTK 4.1 / WebKit6 | X11 / Wayland (via XWayland) | XDG Desktop Portal / Zenity / `notify-send` |
| **macOS** | Apple WebKit (WKWebView) | Cocoa / AppKit | AppKit NSOpenPanel / NSAlert / `osascript` |

---

## 3. Hardware & System Resource Requirements

| Resource | Minimum Requirement | Recommended (Production Enterprise) | Aegis Measured Footprint |
| :--- | :---: | :---: | :---: |
| **CPU** | 1.0 GHz Dual-Core x64 or ARM64 | 2.0 GHz Quad-Core or faster | Single-thread optimized; 0% idle CPU |
| **RAM (Memory)** | 2 GB Total System RAM | 4 GB Total System RAM | **20.4 MB** total process-tree RSS |
| **Storage (Disk)** | 100 MB free space | 500 MB free space | **~35 MB** installed application footprint |
| **GPU / Video** | DirectX 11 / OpenGL 3.3 / Metal | Hardware-accelerated GPU | Software rendering fallback available |
| **Network** | Offline supported (Air-Gap) | Internet connection for updates | Full offline local operation supported |

---

## 4. API Stability & Maintenance Commitment

* **Semantic Versioning**: Releases strictly adhere to SemVer 2.0.0. No breaking public API or IPC contract changes will occur within the `v1.x` release cycle.
* **Security Advisories**: Security patches will be issued directly as patch updates (e.g. `1.0.1`, `1.0.2`) with associated SBOM updates and cryptographically signed release binaries.
