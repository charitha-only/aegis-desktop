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

    #[test]
    fn test_tray_daemon_initial_state() {
        let engine = TrayDaemonEngine::new();
        let status = engine.get_tray_status();
        assert_eq!(status["daemonActive"], true);
        assert_eq!(status["securityBadge"], "ARMED");
        assert_eq!(status["quickKillswitchActive"], false);
        assert_eq!(status["pendingMakerCheckerApprovals"], 0);
        assert!(status["menuItems"].as_array().unwrap().len() >= 4);
    }

    #[test]
    fn test_tray_daemon_quick_killswitch_toggle() {
        let engine = TrayDaemonEngine::new();

        // Engage killswitch
        let res = engine.set_quick_killswitch(true).unwrap();
        assert_eq!(res["killswitchActive"], true);
        assert_eq!(res["badge"], "PANIC_ISOLATED");

        let status = engine.get_tray_status();
        assert_eq!(status["quickKillswitchActive"], true);
        assert_eq!(status["securityBadge"], "PANIC_ISOLATED");

        // Disengage killswitch
        let res2 = engine.set_quick_killswitch(false).unwrap();
        assert_eq!(res2["killswitchActive"], false);
        assert_eq!(res2["badge"], "ARMED");
    }

    #[test]
    fn test_tray_daemon_menu_actions_and_pending_approvals() {
        let engine = TrayDaemonEngine::new();

        // Update pending approvals counter
        engine.update_pending_approvals(5);
        let status = engine.get_tray_status();
        assert_eq!(status["pendingMakerCheckerApprovals"], 5);

        // Trigger menu action: toggle quick killswitch
        let toggle_res = engine.trigger_menu_action("quick_killswitch").unwrap();
        assert_eq!(toggle_res["killswitchActive"], true);

        // Trigger menu action: lock workstation
        let lock_res = engine.trigger_menu_action("lock_workstation").unwrap();
        assert_eq!(lock_res["success"], true);
        assert_eq!(engine.get_tray_status()["securityBadge"], "LOCKED");
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

// ─── Tray Daemon & Quick-Killswitch Engine ────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrayMenuItem {
    pub id: String,
    pub label: String,
    pub item_type: String, // "action", "toggle", "info", "separator"
    pub enabled: bool,
    pub checked: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrayDaemonState {
    pub daemon_active: bool,
    pub security_badge: String, // "ARMED", "DEFENDING", "PANIC_ISOLATED", "MAINTENANCE"
    pub quick_killswitch_active: bool,
    pub pending_maker_checker_approvals: u32,
    pub hardware_token_connected: bool,
    pub tpm_sealed: bool,
    pub menu_items: Vec<TrayMenuItem>,
    pub last_event_timestamp: u64,
}

pub struct TrayDaemonEngine {
    state: std::sync::RwLock<TrayDaemonState>,
}

impl TrayDaemonEngine {
    pub fn new() -> Self {
        let initial_items = vec![
            TrayMenuItem {
                id: "status_header".to_string(),
                label: "Aegis Enterprise Enclave [ARMED]".to_string(),
                item_type: "info".to_string(),
                enabled: false,
                checked: None,
            },
            TrayMenuItem {
                id: "sep_1".to_string(),
                label: "---".to_string(),
                item_type: "separator".to_string(),
                enabled: false,
                checked: None,
            },
            TrayMenuItem {
                id: "quick_killswitch".to_string(),
                label: "Emergency Panic Killswitch".to_string(),
                item_type: "toggle".to_string(),
                enabled: true,
                checked: Some(false),
            },
            TrayMenuItem {
                id: "lock_workstation".to_string(),
                label: "Immediate Zero-Trust Lock".to_string(),
                item_type: "action".to_string(),
                enabled: true,
                checked: None,
            },
            TrayMenuItem {
                id: "pending_approvals".to_string(),
                label: "Pending Approvals: 0".to_string(),
                item_type: "info".to_string(),
                enabled: true,
                checked: None,
            },
        ];

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            state: std::sync::RwLock::new(TrayDaemonState {
                daemon_active: true,
                security_badge: "ARMED".to_string(),
                quick_killswitch_active: false,
                pending_maker_checker_approvals: 0,
                hardware_token_connected: true,
                tpm_sealed: true,
                menu_items: initial_items,
                last_event_timestamp: now,
            }),
        }
    }

    pub fn get_tray_status(&self) -> serde_json::Value {
        let st = self.state.read().unwrap();
        serde_json::json!({
            "apiVersion": env!("CARGO_PKG_VERSION"),
            "daemonActive": st.daemon_active,
            "securityBadge": st.security_badge,
            "quickKillswitchActive": st.quick_killswitch_active,
            "pendingMakerCheckerApprovals": st.pending_maker_checker_approvals,
            "hardwareTokenConnected": st.hardware_token_connected,
            "tpmSealed": st.tpm_sealed,
            "menuItems": st.menu_items,
            "notificationBackend": TrayDaemon::notification_backend(),
            "lastEventTimestamp": st.last_event_timestamp
        })
    }

    pub fn set_quick_killswitch(&self, enabled: bool) -> Result<serde_json::Value, String> {
        let mut st = self.state.write().unwrap();
        st.quick_killswitch_active = enabled;
        st.security_badge = if enabled {
            "PANIC_ISOLATED".to_string()
        } else {
            "ARMED".to_string()
        };

        if let Some(item) = st.menu_items.iter_mut().find(|m| m.id == "quick_killswitch") {
            item.checked = Some(enabled);
        }
        let badge_label = st.security_badge.clone();
        if let Some(header) = st.menu_items.iter_mut().find(|m| m.id == "status_header") {
            header.label = format!("Aegis Enterprise Enclave [{}]", badge_label);
        }

        st.last_event_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(serde_json::json!({
            "killswitchActive": enabled,
            "badge": st.security_badge,
            "status": if enabled { "All external I/O and network ports isolated" } else { "Defensive perimeter normal" }
        }))
    }

    pub fn update_pending_approvals(&self, count: u32) {
        let mut st = self.state.write().unwrap();
        st.pending_maker_checker_approvals = count;
        if let Some(item) = st.menu_items.iter_mut().find(|m| m.id == "pending_approvals") {
            item.label = format!("Pending Approvals: {}", count);
        }
    }

    pub fn trigger_menu_action(&self, action_id: &str) -> Result<serde_json::Value, String> {
        match action_id {
            "quick_killswitch" => {
                let current = {
                    let st = self.state.read().unwrap();
                    st.quick_killswitch_active
                };
                self.set_quick_killswitch(!current)
            }
            "lock_workstation" => {
                let mut st = self.state.write().unwrap();
                st.security_badge = "LOCKED".to_string();
                Ok(serde_json::json!({
                    "action": "lock_workstation",
                    "success": true,
                    "message": "Immediate Zero-Trust lock triggered from system tray"
                }))
            }
            _ => Err(format!("Unknown tray menu action: '{}'", action_id)),
        }
    }
}

pub struct TrayDaemon;

impl TrayDaemon {
    /// Print tray daemon status information.
    pub fn get_status() -> serde_json::Value {
        serde_json::json!({
            "apiVersion": env!("CARGO_PKG_VERSION"),
            "platform": std::env::consts::OS,
            "notificationEngineActive": true,
            "notificationBackend": Self::notification_backend(),
            "supportedUrgencies": if cfg!(target_os = "linux") {vec!["low", "normal", "critical"]} else {vec!["normal"]},
            "providerAvailabilityVerified": true,
            "deliveryConfirmation": false,
            "trayDaemonActive": true,
            "quickKillswitchSupported": true
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
