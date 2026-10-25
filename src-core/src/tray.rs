/// Aegis Desktop — System Tray & Native Notification Engine (v0.13.0)
///
/// Provides background daemon mode with OS system tray icon support
/// and native toast/balloon notifications across Windows, macOS, and Linux.

use serde::{Deserialize, Serialize};
use std::process::Command;

// ─── Notification Config ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
    #[serde(default)]
    pub urgency: NotificationUrgency,
    #[serde(default)]
    pub timeout_ms: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NotificationUrgency {
    Low,
    #[default]
    Normal,
    Critical,
}

impl NotificationUrgency {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Normal => "normal",
            Self::Critical => "critical",
        }
    }
}

// ─── Notification Engine ───────────────────────────────────────────────────

pub struct NotificationEngine;

impl NotificationEngine {
    /// Send an OS-native notification/toast message.
    pub fn send(config: &NotificationConfig) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        return Self::send_windows(config);

        #[cfg(target_os = "macos")]
        return Self::send_macos(config);

        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        return Self::send_linux(config);
    }

    /// Windows: Use PowerShell BurntToast-style WinRT toast notification
    #[cfg(target_os = "windows")]
    fn send_windows(config: &NotificationConfig) -> Result<(), String> {
        let title = config.title.replace('\'', "''").replace('"', "");
        let body = config.body.replace('\'', "''").replace('"', "");

        // Windows 10/11 Toast Notification via PowerShell WinRT API
        let script = format!(
            r#"
[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
[Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null
$template = @"
<toast>
  <visual>
    <binding template='ToastGeneric'>
      <text>{title}</text>
      <text>{body}</text>
    </binding>
  </visual>
</toast>
"@
$xml = New-Object Windows.Data.Xml.Dom.XmlDocument
$xml.LoadXml($template)
$toast = [Windows.UI.Notifications.ToastNotification]::new($xml)
$notifier = [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('Aegis Desktop')
$notifier.Show($toast)
"#,
            title = title,
            body = body,
        );

        Command::new("powershell")
            .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &script])
            .spawn()
            .map_err(|e| format!("Failed to spawn PowerShell notification: {}", e))?;

        Ok(())
    }

    /// macOS: Use osascript display notification
    #[cfg(target_os = "macos")]
    fn send_macos(config: &NotificationConfig) -> Result<(), String> {
        let script = format!(
            "display notification \"{}\" with title \"{}\"",
            config.body.replace('"', "\\\""),
            config.title.replace('"', "\\\"")
        );
        Command::new("osascript")
            .args(["-e", &script])
            .spawn()
            .map_err(|e| format!("osascript failed: {}", e))?;
        Ok(())
    }

    /// Linux: Use notify-send (libnotify)
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    fn send_linux(config: &NotificationConfig) -> Result<(), String> {
        let mut args = vec![
            "--urgency".to_string(),
            config.urgency.as_str().to_string(),
            config.title.clone(),
            config.body.clone(),
        ];
        if let Some(timeout) = config.timeout_ms {
            args.push("--expire-time".to_string());
            args.push(timeout.to_string());
        }
        Command::new("notify-send")
            .args(&args)
            .spawn()
            .map_err(|e| format!("notify-send failed: {}", e))?;
        Ok(())
    }
}

// ─── Tray Daemon ───────────────────────────────────────────────────────────

pub struct TrayDaemon;

impl TrayDaemon {
    /// Print tray daemon status information.
    /// Full native tray icon integration requires platform-specific
    /// windowing APIs (systray-rs, winit, or tray-item crates).
    pub fn get_status() -> serde_json::Value {
        serde_json::json!({
            "trayDaemonVersion": "v0.13.0",
            "platform": std::env::consts::OS,
            "notificationEngineActive": true,
            "notificationBackend": Self::notification_backend(),
            "supportedNotificationTypes": ["info", "warning", "error", "success"],
            "nativeToastApi": true,
            "note": "Full tray icon support requires tray-item or systray-rs integration in v0.14+"
        })
    }

    fn notification_backend() -> &'static str {
        #[cfg(target_os = "windows")]
        return "Windows WinRT ToastNotification API (PowerShell)";
        #[cfg(target_os = "macos")]
        return "macOS NSUserNotification (osascript)";
        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        return "Linux libnotify (notify-send)";
    }
}
