/// Aegis Desktop - Air-Gapped Local Crash Reporter (v0.15.0)
///
/// Records runtime panics, assertion failures, and diagnostic traces strictly
/// into the isolated `./sandbox/crashes/` directory. Zero external pings, zero telemetry leaks.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

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
    sandbox_crashes_dir: PathBuf,
    recent_reports: Mutex<Vec<CrashRecord>>,
}

impl CrashReporter {
    pub fn new(project_root: &Path) -> Self {
        let crashes_dir = project_root.join("sandbox").join("crashes");
        let _ = fs::create_dir_all(&crashes_dir);

        Self {
            sandbox_crashes_dir: crashes_dir,
            recent_reports: Mutex::new(Vec::new()),
        }
    }

    pub fn list_crashes(&self) -> Vec<CrashRecord> {
        let r = self.recent_reports.lock().unwrap();
        r.clone()
    }

    pub fn record_crash(&self, error_kind: &str, details: &str) -> CrashRecord {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let crash_id = format!("crash_{:x}", now);
        let filename = format!("{}.json", crash_id);
        let target_path = self.sandbox_crashes_dir.join(&filename);

        let record = CrashRecord {
            crash_id: crash_id.clone(),
            timestamp: now,
            error_kind: error_kind.to_string(),
            stack_trace: details.to_string(),
            isolated_path: format!("./sandbox/crashes/{}", filename),
            zero_telemetry_guarantee: true,
        };

        if let Ok(content) = serde_json::to_string_pretty(&record) {
            let _ = fs::write(&target_path, content);
        }

        let mut r = self.recent_reports.lock().unwrap();
        r.insert(0, record.clone());
        record
    }
}
