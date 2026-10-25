use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{mpsc, Mutex};
use std::time::{Duration, Instant};

pub fn native_helper() -> Result<Command, String> {
    if !cfg!(feature = "file-dialog") {
        return Err("UNAVAILABLE: build with the file-dialog feature".into());
    }
    let mut command =
        Command::new(std::env::current_exe().map_err(|_| "File picker executable unavailable")?);
    command.arg("--file-picker-helper");
    // Require the portal on Linux; do not leave a separately spawned zenity fallback behind.
    #[cfg(target_os = "linux")]
    command.env("PATH", "/dev/null");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    Ok(command)
}

pub fn run_helper() -> Result<(), String> {
    #[cfg(not(feature = "file-dialog"))]
    {
        Err("UNAVAILABLE: build with the file-dialog feature".into())
    }
    #[cfg(feature = "file-dialog")]
    {
        // Also bound lifetime if the parent is terminated without dropping its job.
        std::thread::spawn(|| {
            std::thread::sleep(Duration::from_secs(120));
            std::process::exit(70);
        });
        let mut input = Vec::new();
        std::io::stdin()
            .take(8193)
            .read_to_end(&mut input)
            .map_err(|_| "Invalid file picker input")?;
        if input.len() > 8192 {
            return Err("File picker input exceeds limits".into());
        }
        let request: FileSelectionRequest =
            serde_json::from_slice(&input).map_err(|_| "Invalid file picker input")?;
        let request = FileSelectionRequest::new(&request.title)?;
        let outcome = match rfd::FileDialog::new().set_title(request.title).pick_file() {
            Some(path) => FileSelectionOutcome::Selected {
                path: path
                    .into_os_string()
                    .into_string()
                    .map_err(|_| "Selected path is not UTF-8")?,
            },
            None => FileSelectionOutcome::NoSelection,
        };
        let output = serde_json::to_vec(&outcome).map_err(|_| "Invalid file picker result")?;
        FileSelectionOutcome::from_output(&output)?;
        std::io::stdout()
            .write_all(&output)
            .map_err(|_| "File picker output failed".into())
    }
}

struct Job {
    id: String,
    profile: String,
    child: Child,
    output: mpsc::Receiver<Result<Vec<u8>, String>>,
    started: Instant,
}

impl Drop for Job {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub trait DialogOutcome: Sized {
    fn from_output(output: &[u8]) -> Result<Self, String>;
}

pub type FileDialogJobs = DialogJobs<FileSelectionOutcome>;

pub struct DialogJobs<T: DialogOutcome> {
    outcome: std::marker::PhantomData<T>,
    job: Mutex<Option<Job>>,
    timeout: Duration,
}

impl<T: DialogOutcome> DialogJobs<T> {
    pub fn new(timeout: Duration) -> Self {
        Self {
            outcome: std::marker::PhantomData,
            job: Mutex::new(None),
            timeout,
        }
    }

    /// The command is a trusted provider adapter, never supplied by IPC callers.
    pub fn start(
        &self,
        profile: &str,
        request: &impl Serialize,
        mut command: Command,
    ) -> Result<String, String> {
        let mut slot = self
            .job
            .lock()
            .map_err(|_| "Native dialog state unavailable")?;
        if slot
            .as_ref()
            .is_some_and(|job| job.started.elapsed() >= self.timeout)
        {
            slot.take();
        }
        if slot.is_some() {
            return Err("A native dialog request already exists; poll or cancel it first".into());
        }
        let mut random = [0u8; 16];
        getrandom::getrandom(&mut random).map_err(|_| "Native dialog identifier unavailable")?;
        let child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| "Native dialog could not start")?;
        let (tx, rx) = mpsc::channel();
        let mut job = Job {
            id: hex::encode(random),
            profile: profile.into(),
            child,
            output: rx,
            started: Instant::now(),
        };
        let stdout = job
            .child
            .stdout
            .take()
            .ok_or("Native dialog output unavailable")?;
        std::thread::Builder::new()
            .name("aegis-picker-output".into())
            .spawn(move || {
                let mut output = Vec::new();
                let result = stdout
                    .take(65537)
                    .read_to_end(&mut output)
                    .map_err(|_| "Native dialog output failed".to_string())
                    .and_then(|_| {
                        if output.len() > 65536 {
                            Err("Native dialog response exceeds limits".into())
                        } else {
                            Ok(output)
                        }
                    });
                let _ = tx.send(result);
            })
            .map_err(|_| "Native dialog reader could not start")?;
        let input = serde_json::to_vec(request).map_err(|_| "Invalid native dialog request")?;
        job.child
            .stdin
            .take()
            .ok_or("Native dialog input unavailable")?
            .write_all(&input)
            .map_err(|_| "Native dialog input failed")?;
        let id = job.id.clone();
        *slot = Some(job);
        Ok(id)
    }

    pub fn poll(&self, profile: &str, id: &str) -> Result<Option<T>, String> {
        let mut slot = self
            .job
            .lock()
            .map_err(|_| "Native dialog state unavailable")?;
        let job = slot.as_mut().ok_or("Unknown native dialog request")?;
        if job.profile != profile || job.id != id {
            return Err("Unknown native dialog request".into());
        }
        if job.started.elapsed() >= self.timeout {
            slot.take();
            return Err("Native dialog timed out".into());
        }
        let exited = job
            .child
            .try_wait()
            .map_err(|_| "Native dialog status unavailable")?;
        if exited.is_none() {
            return Ok(None);
        }
        let output = match job.output.try_recv() {
            Ok(result) => result,
            Err(mpsc::TryRecvError::Empty) => return Ok(None),
            Err(_) => Err("Native dialog output unavailable".into()),
        };
        let success = exited.is_some_and(|status| status.success());
        slot.take();
        if !success {
            return Err("Native dialog helper failed".into());
        }
        T::from_output(&output?).map(Some)
    }

    pub fn cancel(&self, profile: &str, id: &str) -> Result<(), String> {
        let mut slot = self
            .job
            .lock()
            .map_err(|_| "Native dialog state unavailable")?;
        let job = slot.as_ref().ok_or("Unknown native dialog request")?;
        if job.profile != profile || job.id != id {
            return Err("Unknown native dialog request".into());
        }
        slot.take();
        Ok(())
    }

    pub fn invalidate(&self) {
        if let Ok(mut slot) = self.job.lock() {
            slot.take();
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "camelCase", deny_unknown_fields)]
pub enum FileSelectionOutcome {
    Selected { path: String },
    NoSelection,
}

impl FileSelectionOutcome {
    pub fn from_output(output: &[u8]) -> Result<Self, String> {
        if output.len() > 65536 {
            return Err("File picker response exceeds limits".into());
        }
        let outcome: Self =
            serde_json::from_slice(output).map_err(|_| "Invalid file picker response")?;
        if let Self::Selected { path } = &outcome {
            if path.len() > 32768
                || path.contains('\0')
                || !std::path::Path::new(path).is_absolute()
            {
                return Err("Invalid selected file path".into());
            }
        }
        Ok(outcome)
    }
}

impl DialogOutcome for FileSelectionOutcome {
    fn from_output(output: &[u8]) -> Result<Self, String> {
        FileSelectionOutcome::from_output(output)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileSelectionRequest {
    title: String,
}

impl FileSelectionRequest {
    pub fn new(title: &str) -> Result<Self, String> {
        if title.len() > 1024 || title.contains('\0') {
            return Err("Invalid file picker title".into());
        }
        Ok(Self {
            title: title.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn abandoned_expired_request_does_not_permanently_block_new_dialogs() {
        let jobs = super::FileDialogJobs::new(std::time::Duration::from_millis(30));
        let request = super::FileSelectionRequest::new("fixture").unwrap();
        let command = || {
            let mut command = std::process::Command::new("node");
            command.args(["-e", "setTimeout(()=>{},60000)"]);
            command
        };
        let old = jobs.start("work", &request, command()).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(60));
        let new = jobs.start("work", &request, command()).unwrap();
        assert_ne!(old, new);
        assert!(jobs.poll("work", &old).is_err());
        jobs.cancel("work", &new).unwrap();
    }
    #[test]
    fn cancel_and_profile_invalidation_revoke_pending_requests() {
        let jobs = super::FileDialogJobs::new(std::time::Duration::from_secs(5));
        let request = super::FileSelectionRequest::new("fixture").unwrap();
        let launch = || {
            let mut command = std::process::Command::new("node");
            command.args(["-e", "setTimeout(()=>{},60000)"]);
            jobs.start("work", &request, command).unwrap()
        };
        let id = launch();
        assert!(jobs.cancel("other", &id).is_err());
        jobs.cancel("work", &id).unwrap();
        assert!(jobs.poll("work", &id).is_err());
        let id = launch();
        jobs.invalidate();
        assert!(jobs.poll("work", &id).is_err());
    }
    #[test]
    fn picker_timeout_terminates_owned_helper_and_releases_slot() {
        let jobs = super::FileDialogJobs::new(std::time::Duration::from_millis(30));
        let request = super::FileSelectionRequest::new("fixture").unwrap();
        let mut command = std::process::Command::new("node");
        command.args(["-e", "setTimeout(()=>{},60000)"]);
        let id = jobs.start("work", &request, command).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(60));
        assert_eq!(
            jobs.poll("work", &id).err().as_deref(),
            Some("Native dialog timed out")
        );
        assert!(jobs.poll("work", &id).is_err());
    }
    #[test]
    fn picker_job_is_nonblocking_profile_scoped_and_consumed_once() {
        use std::{process::Command, time::Duration};
        let jobs = super::FileDialogJobs::new(Duration::from_secs(5));
        let request = super::FileSelectionRequest::new("fixture").unwrap();
        let mut command = Command::new("node");
        command.args([
            "-e",
            "setTimeout(()=>process.stdout.write('{\"status\":\"noSelection\"}'),300)",
        ]);
        let id = jobs.start("work", &request, command).unwrap();
        assert!(jobs.poll("other", &id).is_err());
        assert!(jobs.poll("work", &id).unwrap().is_none());
        let mut second = Command::new("node");
        second.arg("--version");
        assert!(jobs.start("work", &request, second).is_err());
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(result) = jobs.poll("work", &id).unwrap() {
                assert_eq!(
                    serde_json::to_value(result).unwrap()["status"],
                    "noSelection"
                );
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(jobs.poll("work", &id).is_err());
    }
    #[test]
    fn outcome_rejects_malformed_and_relative_paths_without_granting_access() {
        let path = std::env::temp_dir().join("Aegis fixture.txt");
        let output =
            serde_json::to_vec(&serde_json::json!({"status":"selected","path":path})).unwrap();
        assert!(super::FileSelectionOutcome::from_output(&output).is_ok());
        assert!(super::FileSelectionOutcome::from_output(br#"{"status":"noSelection"}"#).is_ok());
        for output in [
            br#"{"status":"selected","path":"../secret"}"#.as_slice(),
            b"not json",
            br#"{"status":"cancelled"}"#,
        ] {
            assert!(super::FileSelectionOutcome::from_output(output).is_err());
        }
    }
    #[test]
    fn request_rejects_invalid_titles_without_opening_a_picker() {
        assert!(super::FileSelectionRequest::new("Select a file අ").is_ok());
        assert!(super::FileSelectionRequest::new("a\0b").is_err());
        assert!(super::FileSelectionRequest::new(&"x".repeat(1025)).is_err());
    }
}
