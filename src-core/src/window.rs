/// Aegis Desktop - Enhanced Multi-Window Manager (v0.12.0)
///
/// Manages primary and secondary application windows through the OS-native
/// WebView runtime (Edge App Mode on Windows, xdg-open on Linux, open on macOS).
/// Supports spawning modal dialogs, secondary panels, and floating utility windows.

use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

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
        }
    }
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
    pub spawned_at: u64,
}

// ─── Multi-Window Manager ──────────────────────────────────────────────────

pub struct WindowManager {
    active_windows: Arc<Mutex<HashMap<String, WindowRecord>>>,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            active_windows: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Spawn a new OS-native application window pointing to `url`
    pub fn spawn_window(url: &str, config: &WindowConfig) -> Result<String, Box<dyn std::error::Error>> {
        let window_id = Self::generate_window_id();

        #[cfg(target_os = \"windows\")]
        Self::spawn_windows(url, config)?;

        #[cfg(target_os = \"macos\")]
        Self::spawn_macos(url, config)?;

        #[cfg(all(not(target_os = \"windows\"), not(target_os = \"macos\")))]
        Self::spawn_linux(url, config)?;

        Ok(window_id)
    }

    /// Spawn a secondary window and register it in the window registry
    pub fn open_secondary(
        &self,
        window_id: &str,
        url: &str,
        config: &WindowConfig,
    ) -> Result<WindowRecord, String> {
        Self::spawn_window(url, config)
            .map_err(|e| format!(\"Failed to open secondary window: {}\", e))?;

        let record = WindowRecord {
            id: window_id.to_string(),
            url: url.to_string(),
            title: config.title.clone(),
            width: config.width,
            height: config.height,
            is_modal: config.modal,
            spawned_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        let mut registry = self.active_windows.lock().unwrap();
        registry.insert(window_id.to_string(), record.clone());

        Ok(record)
    }

    /// List all currently tracked secondary windows
    pub fn list_windows(&self) -> Vec<WindowRecord> {
        let registry = self.active_windows.lock().unwrap();
        registry.values().cloned().collect()
    }

    /// Remove a window from the registry (called when window closes)
    pub fn close_window(&self, window_id: &str) -> bool {
        let mut registry = self.active_windows.lock().unwrap();
        registry.remove(window_id).is_some()
    }

    fn generate_window_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        format!("win_{:x}", ts)
    }

    // ─── Platform-specific window launchers ─────────────────────────────

    #[cfg(target_os = \"windows\")]
    fn spawn_windows(url: &str, config: &WindowConfig) -> Result<(), Box<dyn std::error::Error>> {
        let edge_paths = [
            \"C:\\\\Program Files (x86)\\\\Microsoft\\\\Edge\\\\Application\\\\msedge.exe\",
            \"C:\\\\Program Files\\\\Microsoft\\\\Edge\\\\Application\\\\msedge.exe\",
        ];

        let mut launched = false;
        for path in &edge_paths {
            if std::path::Path::new(path).exists() {
                let mut cmd = Command::new(path);
                cmd.arg(format!(\"--app={}\", url))
                    .arg(format!(\"--window-size={},{}\", config.width, config.height))
                    .arg(format!(\"--window-name={}\", config.title))
                    .arg(\"--no-first-run\")
                    .arg(\"--no-default-browser-check\");

                if config.kiosk {
                    cmd.arg(\"--kiosk\");
                }

                if config.always_on_top {
                    cmd.arg(\"--force-display-list-compositing\");
                }

                if cmd.spawn().is_ok() {
                    launched = true;
                    break;
                }
            }
        }

        if !launched {
            // Fallback to default browser
            Command::new(\"cmd\").args([\"/C\", \"start\", url]).spawn()?;
        }
        Ok(())
    }

    #[cfg(target_os = \"macos\")]
    fn spawn_macos(url: &str, _config: &WindowConfig) -> Result<(), Box<dyn std::error::Error>> {
        Command::new(\"open\")
            .args([\"-a\", \"Safari\", url])
            .spawn()?;
        Ok(())
    }

    #[cfg(all(not(target_os = \"windows\"), not(target_os = \"macos\")))]
    fn spawn_linux(url: &str, _config: &WindowConfig) -> Result<(), Box<dyn std::error::Error>> {
        Command::new(\"xdg-open\").arg(url).spawn()?;
        Ok(())
    }
}
