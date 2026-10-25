use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};

// ─── Window Configuration ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            custom_protocols: vec!["aegis".to_string()],
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
impl WindowManager {
    pub fn detect_runtime() -> WebviewRuntimeInfo {
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
        _config: &WindowConfig,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if !url.starts_with("http://127.0.0.1:") && !url.starts_with("http://localhost:") {
            return Err("Only local HTTP app windows are supported".into());
        }
        if url.chars().any(|c| c.is_control() || c == '"' || c == ' ') {
            return Err("Invalid window URL".into());
        }
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
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn truthful_runtime() {
        let r = WindowManager::detect_runtime();
        assert!(!r.is_native_webkit);
        assert!(!r.security_sandbox_enforced);
        assert!(r.wry_version.is_empty());
    }
}
