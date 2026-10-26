/// Aegis Desktop — System Tray & Native Notification Engine (v0.13.0)
///
/// Prepares notification helpers and tracks bounded provider completion.
/// Native system-tray icons are not implemented.
use serde::{Deserialize, Serialize};
use std::process::Command;

// ─── Notification Config ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

pub type NotificationJobs = crate::file_dialog::DialogJobs<NotificationOutcome>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationOutcome {
    status: &'static str,
    delivery_confirmed: bool,
}

impl crate::file_dialog::DialogOutcome for NotificationOutcome {
    fn from_output(output: &[u8]) -> Result<Self, String> {
        if !output.is_empty() {
            return Err("Unexpected notification provider output".into());
        }
        Ok(Self {
            status: "providerAccepted",
            delivery_confirmed: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::Platform;
    #[test]
    fn linux_notification_backslash_escapes_cannot_recreate_markup() {
        let config = NotificationConfig {
            title: "fixture".into(),
            body: r"\074b\076text\074/b\076 C:\new\folder".into(),
            icon: None,
            urgency: NotificationUrgency::Normal,
            timeout_ms: None,
        };
        let command = NotificationEngine::prepare(Platform::Linux, &config).unwrap();
        assert_eq!(
            command.get_args().last().unwrap(),
            r"\\074b\\076text\\074/b\\076 C:\\new\\folder"
        );
    }
    #[test]
    fn notification_provider_completion_is_not_visual_delivery() {
        let jobs = super::NotificationJobs::new(std::time::Duration::from_secs(5));
        let mut command = std::process::Command::new("node");
        command.args([
            "-e",
            "process.stdin.on('data',()=>{process.exitCode=7});process.stdin.resume()",
        ]);
        let id = jobs.start_command("work", command).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if let Some(outcome) = jobs.poll("work", &id).unwrap() {
                assert_eq!(
                    serde_json::to_value(outcome).unwrap(),
                    serde_json::json!({"status":"providerAccepted","deliveryConfirmed":false})
                );
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    #[test]
    #[cfg(windows)]
    fn windows_notification_errors_stop_instead_of_reporting_success() {
        use std::os::windows::process::CommandExt;
        let config = NotificationConfig {
            title: "fixture".into(),
            body: "text".into(),
            icon: None,
            urgency: NotificationUrgency::Normal,
            timeout_ms: None,
        };
        let command = NotificationEngine::prepare(Platform::Windows, &config).unwrap();
        let script = command.get_args().last().unwrap().to_str().unwrap();
        // Exercise only the error-policy prefix, never the WinRT notification calls.
        let prefix = script
            .split("[Windows.UI.Notifications.ToastNotificationManager")
            .next()
            .unwrap();
        let output = std::process::Command::new("powershell").args(["-NoProfile", "-NonInteractive", "-Command", &format!("{prefix}\nWrite-Error 'Synthetic provider failure'; Write-Output 'Unexpected continuation'")])
            .creation_flags(0x08000000).output().unwrap();
        assert!(
            !output.status.success(),
            "Provider errors must fail the helper"
        );
        assert!(!String::from_utf8_lossy(&output.stdout).contains("Unexpected continuation"));
    }
    #[test]
    fn linux_notification_body_is_literal_text_instead_of_active_markup() {
        let config = NotificationConfig {
            title: "literal title".into(),
            body: "<b>text</b> & \"quoted\"".into(),
            icon: None,
            urgency: NotificationUrgency::Normal,
            timeout_ms: None,
        };
        let command = NotificationEngine::prepare(Platform::Linux, &config).unwrap();
        assert_eq!(
            command.get_args().last().unwrap(),
            "&lt;b&gt;text&lt;/b&gt; &amp; &quot;quoted&quot;"
        );
    }
    #[test]
    fn notification_options_are_bounded_and_unsupported_options_are_not_ignored() {
        let mut config = NotificationConfig {
            title: "fixture".into(),
            body: "text".into(),
            icon: None,
            urgency: NotificationUrgency::Normal,
            timeout_ms: Some(999),
        };
        assert!(NotificationEngine::prepare(Platform::Linux, &config).is_err());
        config.timeout_ms = Some(60001);
        assert!(NotificationEngine::prepare(Platform::Linux, &config).is_err());
        config.timeout_ms = Some(5000);
        assert!(NotificationEngine::prepare(Platform::Linux, &config).is_ok());
        for platform in [Platform::Windows, Platform::Macos] {
            assert!(NotificationEngine::prepare(platform, &config).is_err());
            config.timeout_ms = None;
            config.urgency = NotificationUrgency::Critical;
            assert!(NotificationEngine::prepare(platform, &config).is_err());
            config.urgency = NotificationUrgency::Normal;
            assert!(NotificationEngine::prepare(platform, &config).is_ok());
            config.timeout_ms = Some(5000);
        }
    }
    #[test]
    fn notification_request_parsing_rejects_overflow_and_unknown_options() {
        let config = NotificationEngine::parse_request(&serde_json::json!({"title":"fixture","body":"text","urgency":"critical","timeout_ms":5000})).unwrap();
        assert_eq!(config.urgency.as_str(), "critical");
        assert_eq!(config.timeout_ms, Some(5000));
        for payload in [
            serde_json::json!({"title":"fixture","body":"text","timeout_ms":4294967296u64}),
            serde_json::json!({"title":"fixture","body":"text","urgency":"arbitrary"}),
            serde_json::json!({"title":"fixture","body":"text","ignoredOption":true}),
            serde_json::json!({"title":false,"body":"text"}),
        ] {
            assert!(NotificationEngine::parse_request(&payload).is_err());
        }
    }
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
    pub fn parse_request(payload: &serde_json::Value) -> Result<NotificationConfig, String> {
        serde_json::from_value(payload.clone()).map_err(|_| "Invalid notification request".into())
    }
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
        if config
            .timeout_ms
            .is_some_and(|timeout| !(1000..=60000).contains(&timeout))
        {
            return Err("Notification timeout must be between 1000 and 60000 milliseconds".into());
        }
        if !matches!(platform, Platform::Linux)
            && (config.timeout_ms.is_some()
                || !matches!(config.urgency, NotificationUrgency::Normal))
        {
            return Err("Notification urgency/timeout overrides are unavailable on this OS".into());
        }
        let mut command = match platform {
            Platform::Windows => {
                let mut command = Command::new("powershell");
                command.args(["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", r#"
$ErrorActionPreference = 'Stop'
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
                command.args(["-e", "on run argv\nwith timeout of 10 seconds\ndisplay notification (item 2 of argv) with title (item 1 of argv)\nend timeout\nend run", "--", &config.title, &config.body]);
                command
            }
            Platform::Linux => {
                let mut command = Command::new("notify-send");
                command.args(["--urgency", config.urgency.as_str()]);
                if let Some(timeout) = config.timeout_ms {
                    command.args(["--expire-time", &timeout.to_string()]);
                }
                let body = config
                    .body
                    .replace('\\', "\\\\")
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;")
                    .replace('"', "&quot;")
                    .replace('\'', "&apos;");
                command.args(["--", &config.title, &body]);
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

    /// Completion is polled; a successful provider exit never proves visual delivery.
    pub fn start(
        jobs: &NotificationJobs,
        profile: &str,
        config: &NotificationConfig,
    ) -> Result<String, String> {
        let command = Self::prepare(crate::platform::Platform::current()?, config)?;
        jobs.start_command(profile, command)
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
            "apiVersion": env!("CARGO_PKG_VERSION"),
            "platform": std::env::consts::OS,
            "notificationEngineActive": true,
            "notificationBackend": Self::notification_backend(),
            "supportedUrgencies": if cfg!(target_os = "linux") {vec!["low", "normal", "critical"]} else {vec!["normal"]},
            "providerAvailabilityVerified": false,
            "deliveryConfirmation": false,
            "trayIconImplemented": false,
            "note": "Provider completion does not confirm visual delivery. Native tray icons are not implemented."
        })
    }

    fn notification_backend() -> &'static str {
        #[cfg(target_os = "windows")]
        return "Windows WinRT ToastNotification API (PowerShell)";
        #[cfg(target_os = "macos")]
        return "macOS notification helper (osascript)";
        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        return "Linux libnotify (notify-send)";
    }
}
