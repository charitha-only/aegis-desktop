use serde::{Deserialize, Serialize};
use std::process::Command;

// ─── Window Configuration ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub resizable: bool,
    #[serde(default)]
    pub modal: bool,
    #[serde(default)]
    pub always_on_top: bool,
    #[serde(default)]
    pub kiosk: bool,
    #[serde(default = "default_decorations")]
    pub decorations: bool,
    #[serde(default)]
    pub transparent: bool,
    #[serde(default)]
    pub devtools_enabled: bool,
    #[serde(default)]
    pub custom_protocols: Vec<String>,
}

fn default_decorations() -> bool {
    true
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Aegis Secure Window".to_string(),
            width: 1024,
            height: 720,
            resizable: true,
            modal: false,
            always_on_top: false,
            kiosk: false,
            decorations: true,
            transparent: false,
            devtools_enabled: false,
            custom_protocols: Vec::new(),
        }
    }
}

// ─── Native Webview Runtime Info ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WebviewBackend {
    WryTaoWebKitMacos,
    WryTaoWebKitLinux,
    WebView2Windows,
    HeadlessTest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebviewRuntimeInfo {
    pub backend: String,
    pub engine: String,
    pub is_native_webkit: bool,
    pub wry_version: String,
    pub tao_version: String,
    pub supports_ipc_bridge: bool,
    pub custom_protocols_supported: bool,
    pub security_sandbox_enforced: bool,
}

// ─── Wry/Tao Window Descriptor ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WryTaoWindowDescriptor {
    pub title: String,
    pub inner_size: (u32, u32),
    pub resizable: bool,
    pub always_on_top: bool,
    pub decorations: bool,
    pub transparent: bool,
    pub initial_url: String,
    pub ipc_handler_registered: bool,
    pub devtools: bool,
    pub custom_protocol: String,
    pub initialization_script: String,
    pub user_agent: String,
}

// ─── Window Record ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowRecord {
    pub id: String,
    pub url: String,
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub is_modal: bool,
    pub backend: String,
    pub spawned_at: u64,
}

// ─── Multi-Window & WebKit Manager ─────────────────────────────────────────

pub struct WindowManager;
static NATIVE_ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[derive(Clone)]
pub struct NativeWindowPolicy {
    initial: String,
}

impl NativeWindowPolicy {
    pub fn new(url: &str, config: &WindowConfig) -> Result<Self, String> {
        WindowManager::validate_request(url, config)?;
        if config.width == 0
            || config.height == 0
            || config.width > 16384
            || config.height > 16384
            || config.title.len() > 1024
            || config.title.contains('\0')
            || config.devtools_enabled
            || !config.custom_protocols.is_empty()
        {
            return Err(
                "Invalid native window size/title or unsupported developer tools request".into(),
            );
        }
        Ok(Self {
            initial: url.to_owned(),
        })
    }

    pub fn allows_navigation(&self, target: &str) -> bool {
        let origin = |url: &str| {
            url.strip_prefix("http://")
                .unwrap_or("")
                .split(['/', '?', '#'])
                .next()
                .unwrap_or("")
                .to_owned()
        };
        WindowManager::validate_request(target, &WindowConfig::default()).is_ok()
            && origin(&self.initial) == origin(target)
    }
}
impl WindowManager {
    #[cfg(feature = "native-desktop")]
    pub(crate) fn mark_native_active() {
        NATIVE_ACTIVE.store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn detect_runtime() -> WebviewRuntimeInfo {
        if NATIVE_ACTIVE.load(std::sync::atomic::Ordering::SeqCst) {
            return WebviewRuntimeInfo {
                backend: "NATIVE_EMBEDDED".into(),
                engine: if cfg!(windows) {
                    "WebView2"
                } else if cfg!(target_os = "macos") {
                    "WKWebView"
                } else {
                    "GTK4/WebKit6"
                }
                .into(),
                is_native_webkit: !cfg!(windows),
                wry_version: if cfg!(target_os = "linux") {
                    ""
                } else {
                    "0.57.1"
                }
                .into(),
                tao_version: String::new(),
                supports_ipc_bridge: true,
                custom_protocols_supported: false,
                security_sandbox_enforced: false,
            };
        }
        WebviewRuntimeInfo {
            backend: "EXTERNAL_BROWSER".into(),
            engine: "System browser (Edge app mode on Windows)".into(),
            is_native_webkit: false,
            wry_version: String::new(),
            tao_version: String::new(),
            supports_ipc_bridge: true,
            custom_protocols_supported: false,
            security_sandbox_enforced: false,
        }
    }
    pub fn spawn_window(
        url: &str,
        config: &WindowConfig,
    ) -> Result<String, Box<dyn std::error::Error>> {
        Self::validate_request(url, config)?;
        #[cfg(target_os = "windows")]
        {
            let mut found = false;
            for var in ["ProgramFiles(x86)", "ProgramFiles"] {
                if let Some(root) = std::env::var_os(var) {
                    let edge = std::path::PathBuf::from(root)
                        .join("Microsoft/Edge/Application/msedge.exe");
                    if edge.is_file() {
                        Command::new(edge).arg(format!("--app={url}")).spawn()?;
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                return Err(
                    "Microsoft Edge is not installed; open the loopback URL manually".into(),
                );
            }
        }
        #[cfg(target_os = "macos")]
        {
            Command::new("open").args(["-a", "Safari", url]).spawn()?;
        }
        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open").arg(url).spawn()?;
        }
        Ok(format!("external-browser-{}", std::process::id()))
    }

    /// Validate without launching a browser or interacting with the desktop.
    pub fn validate_request(url: &str, config: &WindowConfig) -> Result<(), String> {
        if config.modal || config.kiosk || config.always_on_top || config.transparent {
            return Err(
                "Requested window controls are unsupported by the external-browser backend".into(),
            );
        }
        if url
            .chars()
            .any(|c| c.is_control() || c.is_whitespace() || c == '"' || c == '\\')
        {
            return Err("Invalid window URL".into());
        }
        let authority = url
            .strip_prefix("http://")
            .ok_or("Only local HTTP app windows are supported")?
            .split(['/', '?', '#'])
            .next()
            .unwrap_or("");
        let port = authority
            .strip_prefix("127.0.0.1:")
            .or_else(|| authority.strip_prefix("localhost:"))
            .ok_or("Only local HTTP app windows are supported")?;
        if port.is_empty()
            || !port.bytes().all(|c| c.is_ascii_digit())
            || port.parse::<u16>().ok().filter(|p| *p != 0).is_none()
        {
            return Err("Invalid local window port".into());
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_policy_restricts_navigation_and_rejects_unsupported_settings() {
        let config = WindowConfig::default();
        let policy = NativeWindowPolicy::new("http://127.0.0.1:48291/", &config).unwrap();
        assert!(policy.allows_navigation("http://127.0.0.1:48291/settings"));
        for target in [
            "http://127.0.0.1:48292/",
            "https://example.com/",
            "file:///private",
            "javascript:void(0)",
            "http://127.0.0.1:48291@example.com/",
        ] {
            assert!(!policy.allows_navigation(target), "{target}");
        }
        let mut invalid = config.clone();
        invalid.width = 0;
        assert!(NativeWindowPolicy::new("http://localhost:5173", &invalid).is_err());
        invalid = config.clone();
        invalid.devtools_enabled = true;
        assert!(NativeWindowPolicy::new("http://localhost:5173", &invalid).is_err());
        invalid = config;
        invalid.modal = true;
        assert!(NativeWindowPolicy::new("http://localhost:5173", &invalid).is_err());
        invalid.modal = false;
        invalid.custom_protocols = vec!["custom".into()];
        assert!(NativeWindowPolicy::new("http://localhost:5173", &invalid).is_err());
    }
    #[test]
    fn window_request_validation_is_safe_without_launching_a_browser() {
        let config = WindowConfig::default();
        assert!(WindowManager::validate_request("http://127.0.0.1:48291/", &config).is_ok());
        assert!(
            WindowManager::validate_request("http://localhost:5173/?view=app", &config).is_ok()
        );
        for url in [
            "https://example.com",
            "http://127.0.0.1:1234@example.com/",
            "http://localhost:0/",
            "http://localhost:65536/",
            "http://localhost:abc/",
            "http://127.0.0.1:80\\example.com/",
        ] {
            assert!(
                WindowManager::validate_request(url, &config).is_err(),
                "{url}"
            );
        }
        let mut modal = config;
        modal.modal = true;
        assert!(WindowManager::validate_request("http://localhost:5173", &modal).is_err());
    }
    #[test]
    fn truthful_runtime() {
        let r = WindowManager::detect_runtime();
        assert!(!r.is_native_webkit);
        assert!(!r.security_sandbox_enforced);
        assert!(r.wry_version.is_empty());
    }
}
