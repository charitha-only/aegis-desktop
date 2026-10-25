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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::Platform;
    #[test]
    fn notification_requests_are_bounded_and_do_not_silently_ignore_icons() {
        let mut config = NotificationConfig {
            title: "hello".into(),
            body: "message".into(),
            icon: None,
            urgency: NotificationUrgency::Normal,
            timeout_ms: None,
        };
        config.title = "x".repeat(1025);
        assert!(NotificationEngine::prepare(Platform::Windows, &config).is_err());
        config.title = "hello".into();
        config.body = "x".repeat(16385);
        assert!(NotificationEngine::prepare(Platform::Linux, &config).is_err());
        config.body = "message".into();
        config.icon = Some("unimplemented.png".into());
        assert!(NotificationEngine::prepare(Platform::Macos, &config).is_err());
    }

    #[test]
    fn notification_text_is_data_on_every_platform() {
        let config = NotificationConfig {
            title: "A user's $title <tag>".into(),
            body: "Unicode: සිංහල\nsecond line".into(),
            icon: None,
            urgency: NotificationUrgency::Normal,
            timeout_ms: None,
        };
        for platform in [Platform::Windows, Platform::Macos, Platform::Linux] {
            let command = NotificationEngine::prepare(platform, &config).unwrap();
            let args: Vec<_> = command
                .get_args()
                .map(|a| a.to_string_lossy().into_owned())
                .collect();
            match platform {
                Platform::Windows => {
                    let script = args.last().unwrap();
                    assert!(!script.contains(&config.title));
                    assert!(!script.contains(&config.body));
                    let env: Vec<_> = command.get_envs().collect();
                    assert!(env
                        .iter()
                        .any(|(key, value)| *key == "AEGIS_NOTIFICATION_TITLE"
                            && *value == Some(std::ffi::OsStr::new(&config.title))));
                }
                Platform::Macos => {
                    assert_eq!(
                        &args[args.len() - 2..],
                        &[config.title.clone(), config.body.clone()]
                    );
                    assert!(!args[1].contains(&config.title));
                }
                Platform::Linux => {
                    assert_eq!(
                        &args[args.len() - 3..],
                        &["--".to_string(), config.title.clone(), config.body.clone()]
                    );
                }
            }
        }
    }
}

impl NotificationEngine {
    /// Prepare a native notification command without executing it. Text is always data.
    pub fn prepare(
        platform: crate::platform::Platform,
        config: &NotificationConfig,
    ) -> Result<Command, String> {
        if config.title.len() > 1024
            || config.body.len() > 16384
            || config.title.contains('\0')
            || config.body.contains('\0')
        {
            return Err("Notification text exceeds limits or contains a null character".into());
        }
        if config.icon.is_some() {
            return Err("Custom notification icons are unavailable".into());
        }
        use crate::platform::Platform;
        let mut command = match platform {
            Platform::Windows => {
                let mut command = Command::new("powershell");
                command.args(["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", r#"
[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
[Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null
$xml = [Windows.Data.Xml.Dom.XmlDocument]::new()
$xml.LoadXml('<toast><visual><binding template="ToastGeneric"><text/><text/></binding></visual></toast>')
$nodes = $xml.GetElementsByTagName('text')
$nodes.Item(0).AppendChild($xml.CreateTextNode($env:AEGIS_NOTIFICATION_TITLE)) | Out-Null
$nodes.Item(1).AppendChild($xml.CreateTextNode($env:AEGIS_NOTIFICATION_BODY)) | Out-Null
$toast = [Windows.UI.Notifications.ToastNotification]::new($xml)
[Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('Aegis Desktop').Show($toast)
"#]);
                command
                    .env("AEGIS_NOTIFICATION_TITLE", &config.title)
                    .env("AEGIS_NOTIFICATION_BODY", &config.body);
                command
            }
            Platform::Macos => {
                let mut command = Command::new("osascript");
                command.args(["-e", "on run argv\ndisplay notification (item 2 of argv) with title (item 1 of argv)\nend run", "--", &config.title, &config.body]);
                command
            }
            Platform::Linux => {
                let mut command = Command::new("notify-send");
                command.args(["--urgency", config.urgency.as_str()]);
                if let Some(timeout) = config.timeout_ms {
                    command.args(["--expire-time", &timeout.to_string()]);
                }
                command.args(["--", &config.title, &config.body]);
                command
            }
        };
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        Ok(command)
    }

    /// Success means the notification helper was started, not that a user saw a toast.
    pub fn send(config: &NotificationConfig) -> Result<(), String> {
        use crate::platform::Platform;
        let platform = if cfg!(target_os = "windows") {
            Platform::Windows
        } else if cfg!(target_os = "macos") {
            Platform::Macos
        } else if cfg!(target_os = "linux") {
            Platform::Linux
        } else {
            return Err("Notifications are unsupported on this OS".into());
        };
        let mut child = Self::prepare(platform, config)?
            .spawn()
            .map_err(|e| format!("Notification helper failed: {e}"))?;
        std::thread::spawn(move || {
            let _ = child.wait();
        });
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
