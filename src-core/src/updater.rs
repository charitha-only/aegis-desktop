/// Disabled legacy updater prototype, retained for compatibility with old source.
/// Its hashes check integrity only; they do not authenticate a publisher.
/// Runtime updater IPC remains unavailable. The offline developer verifier is
/// documented in docs/UPDATE_VERIFICATION.md and does not enable auto-apply.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

fn constant_time_eq_ascii(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        diff |= x.to_ascii_lowercase() ^ y.to_ascii_lowercase();
    }
    diff == 0
}

// ─── Update Manifest ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateManifest {
    /// Semantic version of the update (e.g. "0.14.0")
    pub version: String,
    /// Human-readable release notes
    pub notes: String,
    /// URL to download the update binary (developer-supplied)
    pub download_url: String,
    /// Expected SHA-256 hash of the downloaded binary
    pub sha256: String,
    /// Release date (ISO 8601)
    pub released_at: String,
    /// Minimum required version to apply this update
    pub min_version: Option<String>,
    /// Whether this is a security-critical mandatory update
    pub mandatory: bool,
}

// ─── Version Comparison ────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq)]
struct SemVer {
    major: u32,
    minor: u32,
    patch: u32,
}

impl SemVer {
    fn parse(v: &str) -> Option<Self> {
        let v = v.trim_start_matches('v');
        let parts: Vec<u32> = v.split('.').filter_map(|p| p.parse().ok()).collect();
        if parts.len() < 2 {
            return None;
        }
        Some(Self {
            major: parts[0],
            minor: parts[1],
            patch: parts.get(2).copied().unwrap_or(0),
        })
    }

    fn is_newer_than(&self, other: &SemVer) -> bool {
        (self.major, self.minor, self.patch) > (other.major, other.minor, other.patch)
    }
}

// ─── Update Engine ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JournalState {
    #[serde(rename = "IDLE")]
    Idle,
    #[serde(rename = "STAGED")]
    Staged,
    #[serde(rename = "VERIFIED")]
    Verified,
    #[serde(rename = "APPLIED")]
    Applied,
    #[serde(rename = "ROLLED_BACK")]
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateJournal {
    pub journal_version: u32,
    pub state: JournalState,
    pub active_version: String,
    pub target_version: Option<String>,
    pub staged_file: Option<String>,
    pub staged_sha256: Option<String>,
    #[serde(default)]
    pub backup_file: Option<String>,
    pub last_status: String,
}

pub struct AutoUpdater {
    current_version: String,
    update_dir: PathBuf,
}

impl AutoUpdater {
    pub fn new(current_version: &str, project_root: &Path) -> Self {
        let update_dir = project_root.join("sandbox").join("updates");
        Self {
            current_version: current_version.to_string(),
            update_dir,
        }
    }

    /// Check if a given manifest represents a newer version than what is running.
    pub fn check_update_available(&self, manifest: &UpdateManifest) -> UpdateCheckResult {
        let current = SemVer::parse(&self.current_version);
        let available = SemVer::parse(&manifest.version);

        match (current, available) {
            (Some(cur), Some(avail)) => {
                if avail.is_newer_than(&cur) {
                    UpdateCheckResult::UpdateAvailable {
                        current: self.current_version.clone(),
                        available: manifest.version.clone(),
                        mandatory: manifest.mandatory,
                        notes: manifest.notes.clone(),
                    }
                } else {
                    UpdateCheckResult::UpToDate {
                        version: self.current_version.clone(),
                    }
                }
            }
            _ => UpdateCheckResult::ParseError {
                reason: format!(
                    "Failed to parse version strings: current='{}', manifest='{}'",
                    self.current_version, manifest.version
                ),
            },
        }
    }

    /// Verify the SHA-256 hash of a downloaded update file before applying using bounded streaming chunks.
    /// Returns Ok(()) if the hash matches, Err if tampered/mismatched.
    pub fn verify_integrity(&self, file_path: &Path, expected_sha256: &str) -> Result<(), String> {
        let mut file = fs::File::open(file_path)
            .map_err(|e| format!("Cannot open update file for verification: {e}"))?;

        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let n = file
                .read(&mut buffer)
                .map_err(|e| format!("Cannot read update file for verification: {e}"))?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        let computed = format!("{:x}", hasher.finalize());

        if constant_time_eq_ascii(&computed, expected_sha256.trim()) {
            let mut journal = self.read_journal();
            if journal.state == JournalState::Staged {
                journal.state = JournalState::Verified;
                journal.staged_sha256 = Some(expected_sha256.to_string());
                journal.last_status = "Cryptographic integrity verified; ready for install".to_string();
                let _ = self.write_journal(&journal);
            }
            Ok(())
        } else {
            Err(format!(
                "SECURITY VIOLATION: SHA-256 mismatch!\nExpected:  {}\nComputed:  {}\nUpdate payload rejected. Possible tampering detected.",
                expected_sha256, computed
            ))
        }
    }

    fn journal_file(&self) -> PathBuf {
        self.update_dir.join("journal.json")
    }

    pub fn read_journal(&self) -> UpdateJournal {
        let default_journal = UpdateJournal {
            journal_version: 1,
            state: JournalState::Idle,
            active_version: self.current_version.clone(),
            target_version: None,
            staged_file: None,
            staged_sha256: None,
            backup_file: None,
            last_status: "No pending update transactions".to_string(),
        };

        let file = self.journal_file();
        if !file.exists() {
            return default_journal;
        }

        match fs::read_to_string(&file) {
            Ok(content) => serde_json::from_str(&content).unwrap_or(default_journal),
            Err(_) => default_journal,
        }
    }

    pub fn write_journal(&self, journal: &UpdateJournal) -> Result<(), String> {
        let _ = fs::create_dir_all(&self.update_dir);
        let content = serde_json::to_string_pretty(journal)
            .map_err(|e| format!("Failed to serialize journal: {e}"))?;
        fs::write(self.journal_file(), content)
            .map_err(|e| format!("Failed to write journal: {e}"))
    }

    pub fn rollback_staged(&self) -> Result<serde_json::Value, String> {
        let mut journal = self.read_journal();
        if let Some(ref staged) = journal.staged_file {
            let path = PathBuf::from(staged);
            if path.exists() {
                let _ = fs::remove_file(path);
            }
        }
        journal.state = JournalState::RolledBack;
        journal.last_status = "Transaction rolled back and staged payload quarantined".to_string();
        self.write_journal(&journal)?;
        Ok(serde_json::json!({
            "rolledBack": true,
            "journal": journal
        }))
    }

    /// Stage a downloaded update binary into the quarantine directory.
    /// The update is NOT applied until `verify_integrity` passes.
    pub fn stage_update(
        &self,
        source_path: &Path,
        version: &str,
        expected_sha256: Option<&str>,
    ) -> Result<StagedUpdate, String> {
        fs::create_dir_all(&self.update_dir)
            .map_err(|e| format!("Cannot create update staging dir: {}", e))?;

        let ext = source_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("bin");
        let safe_ver = version.replace(|c: char| !c.is_alphanumeric() && c != '.', "_");
        let staged_name = format!("aegis-update-{}.{}", safe_ver, ext);
        let staged_path = self.update_dir.join(&staged_name);

        fs::copy(source_path, &staged_path)
            .map_err(|e| format!("Failed to stage update: {}", e))?;

        let journal = UpdateJournal {
            journal_version: 1,
            state: JournalState::Staged,
            active_version: self.current_version.clone(),
            target_version: Some(version.to_string()),
            staged_file: Some(staged_path.to_string_lossy().to_string()),
            staged_sha256: expected_sha256.map(|s| s.to_string()),
            backup_file: None,
            last_status: format!("Update payload staged at {}", staged_name),
        };
        let _ = self.write_journal(&journal);

        Ok(StagedUpdate {
            staged_path: staged_path.to_string_lossy().to_string(),
            version: version.to_string(),
            ready_to_apply: false,
        })
    }

    /// Atomically applies a verified staged update payload to the target executable path.
    /// Creates a pre-apply backup of the current executable.
    /// If replacement fails, it automatically reverts from the backup.
    pub fn apply_staged(&self, target_binary: Option<&Path>) -> Result<serde_json::Value, String> {
        let mut journal = self.read_journal();
        if journal.state != JournalState::Verified {
            return Err(format!(
                "Cannot apply update: journal state is {:?}, must be VERIFIED",
                journal.state
            ));
        }

        let staged_str = journal
            .staged_file
            .as_ref()
            .ok_or_else(|| "Staged file path missing from update journal".to_string())?;
        let staged_path = PathBuf::from(staged_str);
        if !staged_path.exists() {
            return Err(format!("Staged payload file not found at {:?}", staged_path));
        }

        // TOCTOU defense: re-verify cryptographic integrity immediately before apply
        if let Some(ref expected_hash) = journal.staged_sha256 {
            self.verify_integrity(&staged_path, expected_hash)
                .map_err(|e| format!("Pre-apply integrity check failed: {}", e))?;
        }

        // Determine destination target binary
        let current_exe = match target_binary {
            Some(p) => p.to_path_buf(),
            None => std::env::current_exe()
                .map_err(|e| format!("Cannot resolve current executable path: {}", e))?,
        };

        // Prepare backup path (e.g. aegis-core.exe.old)
        let backup_path = current_exe.with_extension(format!(
            "{}.old",
            current_exe.extension().and_then(|e| e.to_str()).unwrap_or("exe")
        ));

        // Step 1: Create backup of current binary
        if current_exe.exists() {
            if backup_path.exists() {
                let _ = fs::remove_file(&backup_path);
            }
            if let Err(e) = fs::rename(&current_exe, &backup_path) {
                // If rename fails (e.g. across mount boundaries), fall back to copy
                fs::copy(&current_exe, &backup_path)
                    .map_err(|ce| format!("Failed to create executable backup: {} (copy error: {})", e, ce))?;
            }
        }

        // Step 2: Copy verified staged payload to the target binary location
        if let Err(e) = fs::copy(&staged_path, &current_exe) {
            // Rollback immediately if copying fails
            if backup_path.exists() {
                let _ = fs::rename(&backup_path, &current_exe);
            }
            journal.state = JournalState::RolledBack;
            journal.last_status = format!("Apply failed during binary copy: {}; rolled back to backup", e);
            let _ = self.write_journal(&journal);
            return Err(format!("Binary replacement failed: {}; rollback restored original executable", e));
        }

        // Step 3: Transition journal state to Applied
        let applied_version = journal.target_version.clone().unwrap_or_else(|| "unknown".to_string());
        journal.state = JournalState::Applied;
        journal.backup_file = Some(backup_path.to_string_lossy().to_string());
        journal.last_status = format!(
            "Successfully applied update to version {} (backup at {:?})",
            applied_version,
            backup_path.file_name().unwrap_or_default()
        );
        self.write_journal(&journal)?;

        Ok(serde_json::json!({
            "applied": true,
            "targetVersion": applied_version,
            "executable": current_exe.to_string_lossy().to_string(),
            "backupFile": backup_path.to_string_lossy().to_string(),
            "restartRequired": true,
            "journal": journal
        }))
    }

    /// Get current updater status for IPC inspection.
    pub fn get_status(&self) -> serde_json::Value {
        let updates_exist = self.update_dir.exists()
            && fs::read_dir(&self.update_dir)
                .map(|mut d| d.next().is_some())
                .unwrap_or(false);

        let staged: Vec<String> = if self.update_dir.exists() {
            fs::read_dir(&self.update_dir)
                .ok()
                .map(|d| {
                    d.filter_map(|e| e.ok())
                        .filter_map(|e| e.file_name().into_string().ok())
                        .collect()
                })
                .unwrap_or_default()
        } else {
            vec![]
        };

        serde_json::json!({
            "autoUpdaterVersion": format!("v{}", self.current_version),
            "currentVersion": self.current_version,
            "stagingDir": self.update_dir.to_string_lossy(),
            "hasStagedUpdates": updates_exist,
            "stagedFiles": staged,
            "journal": self.read_journal(),
            "verificationAlgorithm": "SHA-256",
            "securityPolicy": "All updates must pass SHA-256 hash verification before execution",
            "airGapCompatible": true,
            "autoApply": false,
            "note": "Supply your own manifest URL — Aegis never auto-downloads without developer opt-in"
        })
    }
}

// ─── Result Types ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum UpdateCheckResult {
    #[serde(rename = "update_available")]
    UpdateAvailable {
        current: String,
        available: String,
        mandatory: bool,
        notes: String,
    },
    #[serde(rename = "up_to_date")]
    UpToDate { version: String },
    #[serde(rename = "parse_error")]
    ParseError { reason: String },
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct StagedUpdate {
    pub staged_path: String,
    pub version: String,
    pub ready_to_apply: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempDir(std::path::PathBuf);
    impl TempDir {
        fn new() -> Self {
            let mut bytes = [0u8; 16];
            getrandom::getrandom(&mut bytes).expect("secure random");
            let path = std::env::temp_dir().join(format!("aegis-updater-test-{}", hex::encode(bytes)));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
        fn path(&self) -> &std::path::Path {
            &self.0
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn test_semver_parsing_and_comparison() {
        let v1 = SemVer::parse("1.2.3").unwrap();
        let v2 = SemVer::parse("v1.2.4").unwrap();
        let v3 = SemVer::parse("1.3.0").unwrap();
        let v4 = SemVer::parse("2.0.0").unwrap();
        let v_short = SemVer::parse("0.15").unwrap();

        assert_eq!(v_short.patch, 0);
        assert!(v2.is_newer_than(&v1));
        assert!(v3.is_newer_than(&v2));
        assert!(v4.is_newer_than(&v3));
        assert!(!v1.is_newer_than(&v2));
        assert!(!v1.is_newer_than(&v1));

        assert!(SemVer::parse("invalid").is_none());
    }

    #[test]
    fn test_constant_time_eq_ascii() {
        assert!(constant_time_eq_ascii("abcdef", "ABCDEF"));
        assert!(constant_time_eq_ascii("1234567890abcdef", "1234567890ABCDEF"));
        assert!(!constant_time_eq_ascii("abcdef", "abcdeg"));
        assert!(!constant_time_eq_ascii("abcdef", "abcdefg"));
        assert!(!constant_time_eq_ascii("", "a"));
    }

    #[test]
    fn test_check_update_available() {
        let temp = TempDir::new();
        let updater = AutoUpdater::new("0.30.2", temp.path());

        let mut manifest = UpdateManifest {
            version: "0.30.3".into(),
            notes: "Bug fixes".into(),
            download_url: "http://example.com/update".into(),
            sha256: "0".repeat(64),
            released_at: "2026-11-10".into(),
            min_version: None,
            mandatory: false,
        };

        match updater.check_update_available(&manifest) {
            UpdateCheckResult::UpdateAvailable { current, available, .. } => {
                assert_eq!(current, "0.30.2");
                assert_eq!(available, "0.30.3");
            }
            _ => panic!("Expected UpdateAvailable"),
        }

        manifest.version = "0.30.2".into();
        match updater.check_update_available(&manifest) {
            UpdateCheckResult::UpToDate { version } => {
                assert_eq!(version, "0.30.2");
            }
            _ => panic!("Expected UpToDate"),
        }
    }

    #[test]
    fn test_verify_integrity_streaming() {
        let temp = TempDir::new();
        let file_path = temp.path().join("test_update.bin");
        let payload = b"Aegis Desktop Update Payload Test Bytes 1234567890";
        fs::write(&file_path, payload).unwrap();

        let mut hasher = Sha256::new();
        hasher.update(payload);
        let expected_sha256 = format!("{:x}", hasher.finalize());

        let updater = AutoUpdater::new("0.30.3", temp.path());
        assert!(updater.verify_integrity(&file_path, &expected_sha256).is_ok());

        let tampered_sha256 = "1".repeat(64);
        assert!(updater.verify_integrity(&file_path, &tampered_sha256).is_err());
    }

    #[test]
    fn test_transactional_journal_workflow() {
        let temp = TempDir::new();
        let updater = AutoUpdater::new("0.30.6", temp.path());

        // 1. Initial State: Idle
        let journal = updater.read_journal();
        assert_eq!(journal.state, JournalState::Idle);
        assert_eq!(journal.active_version, "0.30.6");

        // 2. Stage an update
        let payload = b"Aegis v0.30.7 Staged Update Package Bytes";
        let src_file = temp.path().join("downloaded_v0.30.7.bin");
        fs::write(&src_file, payload).unwrap();

        let mut hasher = Sha256::new();
        hasher.update(payload);
        let expected_sha256 = format!("{:x}", hasher.finalize());

        let staged = updater.stage_update(&src_file, "0.30.7", Some(&expected_sha256)).unwrap();
        assert_eq!(staged.version, "0.30.7");

        let journal_after_stage = updater.read_journal();
        assert_eq!(journal_after_stage.state, JournalState::Staged);
        assert_eq!(journal_after_stage.target_version.as_deref(), Some("0.30.7"));
        assert!(PathBuf::from(journal_after_stage.staged_file.as_ref().unwrap()).exists());

        // 3. Verify integrity: transitions to Verified
        let staged_path = PathBuf::from(journal_after_stage.staged_file.as_ref().unwrap());
        assert!(updater.verify_integrity(&staged_path, &expected_sha256).is_ok());

        let journal_after_verify = updater.read_journal();
        assert_eq!(journal_after_verify.state, JournalState::Verified);

        // 4. Rollback: quarantines staged file and marks RolledBack
        let rollback_res = updater.rollback_staged().unwrap();
        assert_eq!(rollback_res["rolledBack"], true);

        let journal_after_rollback = updater.read_journal();
        assert_eq!(journal_after_rollback.state, JournalState::RolledBack);
        assert!(!staged_path.exists());
    }

    #[test]
    fn test_apply_staged_transactional_workflow() {
        let temp = TempDir::new();
        let updater = AutoUpdater::new("0.30.7", temp.path());

        // Create a mock current binary
        let mock_exe = temp.path().join("mock-aegis-core.exe");
        fs::write(&mock_exe, b"OLD_BINARY_V0.30.7").unwrap();

        // 1. Stage update payload
        let new_payload = b"NEW_BINARY_V0.30.8_ENTERPRISE";
        let src_file = temp.path().join("new_release.bin");
        fs::write(&src_file, new_payload).unwrap();

        let mut hasher = Sha256::new();
        hasher.update(new_payload);
        let expected_sha256 = format!("{:x}", hasher.finalize());

        let staged = updater.stage_update(&src_file, "0.30.8", Some(&expected_sha256)).unwrap();
        let staged_path = PathBuf::from(staged.staged_path);

        // 2. Cannot apply while still in STAGED state
        assert!(updater.apply_staged(Some(&mock_exe)).is_err());

        // 3. Verify integrity -> VERIFIED
        assert!(updater.verify_integrity(&staged_path, &expected_sha256).is_ok());

        // 4. Apply update atomically
        let apply_result = updater.apply_staged(Some(&mock_exe)).unwrap();
        assert_eq!(apply_result["applied"], true);
        assert_eq!(apply_result["targetVersion"], "0.30.8");

        // 5. Verify journal state is APPLIED
        let journal = updater.read_journal();
        assert_eq!(journal.state, JournalState::Applied);
        assert_eq!(journal.target_version.as_deref(), Some("0.30.8"));

        // 6. Verify target executable replaced with new payload
        let updated_bytes = fs::read(&mock_exe).unwrap();
        assert_eq!(updated_bytes, new_payload);

        // 7. Verify backup created
        let backup_path = PathBuf::from(journal.backup_file.unwrap());
        assert!(backup_path.exists());
        let backup_bytes = fs::read(&backup_path).unwrap();
        assert_eq!(backup_bytes, b"OLD_BINARY_V0.30.7");
    }
}
