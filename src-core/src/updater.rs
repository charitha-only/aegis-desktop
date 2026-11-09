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
            Ok(())
        } else {
            Err(format!(
                "SECURITY VIOLATION: SHA-256 mismatch!\nExpected:  {}\nComputed:  {}\nUpdate payload rejected. Possible tampering detected.",
                expected_sha256, computed
            ))
        }
    }

    /// Stage a downloaded update binary into the quarantine directory.
    /// The update is NOT applied until `verify_integrity` passes.
    #[allow(dead_code)]
    pub fn stage_update(&self, source_path: &Path, version: &str) -> Result<StagedUpdate, String> {
        fs::create_dir_all(&self.update_dir)
            .map_err(|e| format!("Cannot create update staging dir: {}", e))?;

        let ext = source_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("bin");
        let staged_name = format!("aegis-update-{}.{}", version.replace('.', "_"), ext);
        let staged_path = self.update_dir.join(&staged_name);

        fs::copy(source_path, &staged_path)
            .map_err(|e| format!("Failed to stage update: {}", e))?;

        Ok(StagedUpdate {
            staged_path: staged_path.to_string_lossy().to_string(),
            version: version.to_string(),
            ready_to_apply: false,
        })
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
}
