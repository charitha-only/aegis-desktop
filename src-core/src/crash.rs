//! Bounded, application-wide local diagnostics. Caller traces are never stored.
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

const MAX_RECORDS: usize = 32;
const MAX_BYTES: usize = 64 * 1024;
const LOCATION: &str = "./diagnostics/events.json";
const MESSAGE: &str =
    "Application diagnostic event; caller trace omitted to avoid persisting secrets";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashRecord {
    pub crash_id: String,
    pub timestamp: u64,
    pub error_kind: String,
    pub stack_trace: String,
    pub isolated_path: String,
    pub zero_telemetry_guarantee: bool,
}

pub struct CrashReporter {
    base_dir: PathBuf,
    access: Mutex<()>,
}
impl CrashReporter {
    pub fn new(project_root: &Path) -> Self {
        Self {
            base_dir: project_root.to_path_buf(),
            access: Mutex::new(()),
        }
    }
    fn directory(&self) -> Result<PathBuf, String> {
        require_directory(&self.base_dir)?;
        let dir = self.base_dir.join("diagnostics");
        match fs::create_dir(&dir) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(_) => return Err("Cannot create diagnostic directory".into()),
        }
        require_directory(&dir)?;
        Ok(dir)
    }
    fn lock(dir: &Path) -> Result<fs::File, String> {
        let path = dir.join(".aegis-diagnostics.lock");
        require_regular_or_missing(&path)?;
        let mut options = fs::OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let lock = options
            .open(path)
            .map_err(|_| "Cannot open diagnostic lock")?;
        lock.try_lock()
            .map_err(|_| "Diagnostics busy; retry the operation")?;
        Ok(lock)
    }
    fn load(dir: &Path) -> Result<Vec<CrashRecord>, String> {
        let path = dir.join("events.json");
        require_regular_or_missing(&path)?;
        let file = match fs::File::open(path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(_) => return Err("Cannot open diagnostic history".into()),
        };
        if file
            .metadata()
            .map_err(|_| "Cannot inspect diagnostic history")?
            .len()
            > MAX_BYTES as u64
        {
            return Err("Diagnostic history exceeds size limit".into());
        }
        let mut bytes = Vec::new();
        file.take(MAX_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "Cannot read diagnostic history")?;
        if bytes.len() > MAX_BYTES {
            return Err("Diagnostic history exceeds size limit".into());
        }
        let records: Vec<CrashRecord> = serde_json::from_slice(&bytes)
            .map_err(|_| "Invalid diagnostic history; preserve it for recovery")?;
        let mut ids = std::collections::HashSet::new();
        if records.len() > MAX_RECORDS
            || records.iter().any(|r| {
                let id = r.crash_id.strip_prefix("crash_").unwrap_or("");
                id.len() != 32
                    || !id.bytes().all(|b| b.is_ascii_hexdigit())
                    || !ids.insert(&r.crash_id)
                    || !valid_kind(&r.error_kind)
                    || r.stack_trace != MESSAGE
                    || r.isolated_path != LOCATION
                    || !r.zero_telemetry_guarantee
            })
        {
            return Err("Invalid diagnostic history; preserve it for recovery".into());
        }
        Ok(records)
    }
    pub fn list_crashes(&self) -> Result<Vec<CrashRecord>, String> {
        let _access = self
            .access
            .lock()
            .map_err(|_| "Diagnostic state unavailable")?;
        let dir = self.directory()?;
        let _lock = Self::lock(&dir)?;
        Self::load(&dir)
    }
    pub fn record_crash(&self, error_kind: &str, _details: &str) -> Result<CrashRecord, String> {
        if !valid_kind(error_kind) {
            return Err("Unsupported diagnostic kind".into());
        }
        let _access = self
            .access
            .lock()
            .map_err(|_| "Diagnostic state unavailable")?;
        let dir = self.directory()?;
        let _lock = Self::lock(&dir)?;
        let mut records = Self::load(&dir)?;
        let mut random = [0u8; 16];
        getrandom::getrandom(&mut random).map_err(|_| "Secure diagnostic identity unavailable")?;
        let record = CrashRecord {
            crash_id: format!("crash_{}", hex::encode(random)),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            error_kind: error_kind.into(),
            stack_trace: MESSAGE.into(),
            isolated_path: LOCATION.into(),
            zero_telemetry_guarantee: true,
        };
        records.insert(0, record.clone());
        records.truncate(MAX_RECORDS);
        let data =
            serde_json::to_vec(&records).map_err(|_| "Cannot serialize diagnostic history")?;
        if data.len() > MAX_BYTES {
            return Err("Diagnostic history exceeds size limit".into());
        }
        crate::storage::atomic_write(&dir.join("events.json"), &data, false)?;
        Ok(record)
    }
}
fn valid_kind(kind: &str) -> bool {
    matches!(
        kind,
        "ControlledDiagnosticTrap" | "assertion" | "panic" | "oom" | "ipc_error" | "sandbox_escape"
    )
}
fn linked(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return true;
        }
    }
    metadata.file_type().is_symlink()
}
fn require_directory(path: &Path) -> Result<(), String> {
    let meta = fs::symlink_metadata(path).map_err(|_| "Cannot inspect diagnostic directory")?;
    if linked(&meta) || !meta.is_dir() {
        return Err("Linked or invalid diagnostic directory rejected".into());
    }
    Ok(())
}
fn require_regular_or_missing(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(meta) if linked(&meta) || !meta.is_file() => {
            Err("Linked or invalid diagnostic file rejected".into())
        }
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("Cannot inspect diagnostic file".into()),
    }
}
