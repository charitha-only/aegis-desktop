use serde::{Deserialize, Serialize};

pub fn native_helper() -> Result<std::process::Command, String> {
    if !cfg!(feature = "native-dialogs") {
        return Err("UNAVAILABLE: build with the native-dialogs feature".into());
    }
    let mut command = std::process::Command::new(
        std::env::current_exe().map_err(|_| "Message dialog executable unavailable")?,
    );
    command.arg("--message-dialog-helper");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    Ok(command)
}

pub fn run_helper() -> Result<(), String> {
    #[cfg(not(feature = "native-dialogs"))]
    {
        Err("UNAVAILABLE: build with the native-dialogs feature".into())
    }
    #[cfg(feature = "native-dialogs")]
    {
        use std::io::Read;
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_secs(120));
            std::process::exit(70);
        });
        let mut input = Vec::new();
        std::io::stdin()
            .take(131073)
            .read_to_end(&mut input)
            .map_err(|_| "Invalid message dialog input")?;
        if input.len() > 131072 {
            return Err("Message dialog input exceeds limits".into());
        }
        let request: MessageRequest =
            serde_json::from_slice(&input).map_err(|_| "Invalid message dialog input")?;
        let request = MessageRequest::new(&request.title, &request.message, &request.kind)?;
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::process::CommandExt;
            // Replacing this helper keeps the visible provider under the supervisor's PID.
            let _ = request.linux_command()?.exec();
            Err("Native message dialog provider unavailable".into())
        }
        #[cfg(not(target_os = "linux"))]
        {
            use std::io::Write;
            let level = match request.kind.as_str() {
                "warning" => rfd::MessageLevel::Warning,
                "error" => rfd::MessageLevel::Error,
                _ => rfd::MessageLevel::Info,
            };
            let result = rfd::MessageDialog::new()
                .set_title(request.title)
                .set_description(request.message)
                .set_level(level)
                .set_buttons(rfd::MessageButtons::Ok)
                .show();
            if result != rfd::MessageDialogResult::Ok {
                return Err("Message dialog acknowledgement unavailable".into());
            }
            std::io::stdout()
                .write_all(br#"{"status":"acknowledged"}"#)
                .map_err(|_| "Message dialog output failed".into())
        }
    }
}

pub type MessageDialogJobs = crate::file_dialog::DialogJobs<MessageOutcome>;

#[derive(Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "camelCase", deny_unknown_fields)]
pub enum MessageOutcome {
    Acknowledged,
}

impl crate::file_dialog::DialogOutcome for MessageOutcome {
    fn from_output(output: &[u8]) -> Result<Self, String> {
        // The Linux provider replaces the helper with zenity, whose successful
        // OK action exits zero with empty stdout. The supervisor checks exit status first.
        if cfg!(target_os = "linux") && output.is_empty() {
            return Ok(Self::Acknowledged);
        }
        if output.len() > 1024 {
            return Err("Invalid message dialog result".into());
        }
        serde_json::from_slice(output).map_err(|_| "Invalid message dialog result".into())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MessageRequest {
    title: String,
    message: String,
    kind: String,
}

impl MessageRequest {
    pub fn linux_command(&self) -> Result<std::process::Command, String> {
        Self::new(&self.title, &self.message, &self.kind)?;
        let mut command = std::process::Command::new("zenity");
        command
            .arg(format!("--{}", self.kind))
            .args(["--no-markup", "--timeout=120"])
            .arg(format!("--title={}", self.title))
            .arg(format!("--text={}", self.message));
        Ok(command)
    }
    pub fn new(title: &str, message: &str, kind: &str) -> Result<Self, String> {
        if title.len() > 1024
            || message.len() > 16384
            || title.contains('\0')
            || message.contains('\0')
        {
            return Err("Invalid message dialog text".into());
        }
        if !matches!(kind, "info" | "warning" | "error") {
            return Err("Unsupported dialog type".into());
        }
        Ok(Self {
            title: title.into(),
            message: message.into(),
            kind: kind.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn failed_provider_cannot_report_acknowledgement() {
        let jobs = super::MessageDialogJobs::new(std::time::Duration::from_secs(5));
        let request = super::MessageRequest::new("fixture", "message", "info").unwrap();
        let mut command = std::process::Command::new("node");
        command.args([
            "-e",
            "process.stdout.write('{\"status\":\"acknowledged\"}');process.exitCode=7",
        ]);
        let id = jobs.start("work", &request, command).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            match jobs.poll("work", &id) {
                Ok(None) => {}
                Ok(Some(_)) => panic!("Failed provider must not acknowledge"),
                Err(error) => {
                    assert_eq!(error, "Native dialog helper failed");
                    break;
                }
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    #[test]
    fn linux_message_command_preserves_literal_text_and_disables_markup() {
        let request =
            super::MessageRequest::new("--title=$title", "<b>literal</b>\n'value'", "warning")
                .unwrap();
        let command = request.linux_command().unwrap();
        assert_eq!(command.get_program(), "zenity");
        let args: Vec<_> = command
            .get_args()
            .map(|a| a.to_string_lossy().to_string())
            .collect();
        assert_eq!(
            args,
            [
                "--warning",
                "--no-markup",
                "--timeout=120",
                "--title=--title=$title",
                "--text=<b>literal</b>\n'value'"
            ]
        );
    }
    #[test]
    fn message_result_requires_acknowledgement_and_rejects_file_results() {
        use crate::file_dialog::DialogOutcome;
        assert!(super::MessageOutcome::from_output(br#"{"status":"acknowledged"}"#).is_ok());
        assert_eq!(
            super::MessageOutcome::from_output(b"").is_ok(),
            cfg!(target_os = "linux")
        );
        for output in [
            b"garbage".as_slice(),
            br#"{"status":"selected","path":"/tmp/a"}"#,
            br#"{"status":"cancelled"}"#,
        ] {
            assert!(super::MessageOutcome::from_output(output).is_err());
        }
    }
    #[test]
    fn message_requests_validate_kind_text_and_limits_before_native_work() {
        use super::MessageRequest;
        assert!(MessageRequest::new("Title අ", "Text <tag> $value\nline", "info").is_ok());
        assert!(MessageRequest::new("title", "message", "warning").is_ok());
        assert!(MessageRequest::new("title", "message", "error").is_ok());
        assert!(MessageRequest::new("title", "message", "arbitrary").is_err());
        assert!(MessageRequest::new("a\0b", "message", "info").is_err());
        assert!(MessageRequest::new("title", "a\0b", "info").is_err());
        assert!(MessageRequest::new(&"x".repeat(1025), "message", "info").is_err());
        assert!(MessageRequest::new("title", &"x".repeat(16385), "info").is_err());
    }
}
