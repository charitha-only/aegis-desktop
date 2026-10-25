use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsCapabilities {
    pub allow_read: Vec<String>,
    pub allow_write: Vec<String>,
    pub deny_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCapabilities {
    pub allow_system_info: bool,
    pub allow_dialogs: bool,
    pub allow_shell_execution: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoCapabilities {
    pub allow_hashing: bool,
    pub allow_random_generation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityManifest {
    pub fs: FsCapabilities,
    pub system: SystemCapabilities,
    pub crypto: CryptoCapabilities,
}

impl Default for CapabilityManifest {
    fn default() -> Self {
        Self {
            fs: FsCapabilities {
                allow_read: vec!["./sandbox".to_string()],
                allow_write: vec!["./sandbox".to_string()],
                deny_paths: vec!["../".to_string(), "C:\\Windows".to_string()],
            },
            system: SystemCapabilities {
                allow_system_info: true,
                allow_dialogs: true,
                allow_shell_execution: false,
            },
            crypto: CryptoCapabilities {
                allow_hashing: true,
                allow_random_generation: true,
            },
        }
    }
}

pub struct SecurityEngine {
    manifest: CapabilityManifest,
    sandbox_root: RwLock<PathBuf>,
}

impl SecurityEngine {
    pub fn new(manifest: CapabilityManifest, base_dir: &Path) -> Self {
        let sandbox_root = base_dir.join("sandbox");
        if !sandbox_root.exists() {
            let _ = std::fs::create_dir_all(&sandbox_root);
        }
        Self {
            manifest,
            sandbox_root: RwLock::new(sandbox_root),
        }
    }

    pub fn set_sandbox_root(&self, new_root: PathBuf) {
        if !new_root.exists() {
            let _ = std::fs::create_dir_all(&new_root);
        }
        if let Ok(mut lock) = self.sandbox_root.write() {
            *lock = new_root;
        }
    }

    pub fn get_sandbox_root(&self) -> PathBuf {
        self.sandbox_root.read().unwrap().clone()
    }

    pub fn check_fs_read(&self, requested_path: &str) -> Result<PathBuf, String> {
        self.resolve_and_validate_path(requested_path, false)
    }

    pub fn check_fs_write(&self, requested_path: &str) -> Result<PathBuf, String> {
        self.resolve_and_validate_path(requested_path, true)
    }

    pub fn check_system_info(&self) -> Result<(), String> {
        if self.manifest.system.allow_system_info {
            Ok(())
        } else {
            Err("Security Violation: 'system_info' capability is disabled in aegis.config.json".into())
        }
    }

    pub fn check_dialogs(&self) -> Result<(), String> {
        if self.manifest.system.allow_dialogs {
            Ok(())
        } else {
            Err("Security Violation: 'dialogs' capability is disabled".into())
        }
    }

    pub fn check_shell(&self) -> Result<(), String> {
        if self.manifest.system.allow_shell_execution {
            Ok(())
        } else {
            Err("Security Violation: Arbitrary shell execution is permanently BLOCKED by policy".into())
        }
    }

    pub fn check_crypto(&self) -> Result<(), String> {
        if self.manifest.crypto.allow_hashing {
            Ok(())
        } else {
            Err("Security Violation: 'crypto' capability is disabled".into())
        }
    }

    fn resolve_and_validate_path(&self, requested: &str, is_write: bool) -> Result<PathBuf, String> {
        let sandbox_root = self.sandbox_root.read().unwrap().clone();
        let canonical_base = sandbox_root
            .canonicalize()
            .map_err(|e| format!("Failed to canonicalize sandbox root: {}", e))?;

        // 1. Strict Traversal Checks
        if requested.contains("..") {
            return Err("Security Violation: Directory traversal '..' detected!".into());
        }

        // 2. Deny blacklisted explicit patterns
        for denied in &self.manifest.fs.deny_paths {
            if requested.to_lowercase().starts_with(&denied.to_lowercase()) {
                return Err(format!("Security Violation: Path matches denied pattern '{}'", denied));
            }
        }

        let p = Path::new(requested);

        // 3. Absolute path handling:
        // If an absolute path was requested, it MUST strictly fall inside canonical_base.
        // It cannot bypass sandbox by supplying C:\ or /etc.
        let full_target = if p.is_absolute() {
            let canon = if p.exists() {
                p.canonicalize().map_err(|_| "Invalid path target".to_string())?
            } else if is_write {
                // If writing a new file with absolute path, check its parent
                if let Some(parent) = p.parent() {
                    let parent_canon = parent
                        .canonicalize()
                        .map_err(|_| "Security Violation: Parent directory of absolute path is inaccessible".to_string())?;
                    if !parent_canon.starts_with(&canonical_base) {
                        return Err("Security Violation: Absolute path escapes sandbox root boundary!".into());
                    }
                }
                p.to_path_buf()
            } else {
                return Err(format!("File not found in sandbox: {}", requested));
            };

            if canon.exists() && !canon.starts_with(&canonical_base) {
                return Err("Security Violation: Absolute path escapes sandbox root boundary!".into());
            }
            canon
        } else {
            // Relative path: strictly strip any "./", "sandbox/", or "sandbox_*" prefix and resolve within active sandbox_root
            let mut clean = requested.trim_start_matches("./");
            if clean.starts_with("sandbox/") {
                clean = &clean[8..];
            } else if clean.starts_with("sandbox_") {
                if let Some(idx) = clean.find('/') {
                    clean = &clean[idx + 1..];
                }
            }
            sandbox_root.join(clean)
        };

        // 4. Final canonical validation
        if !is_write {
            match full_target.canonicalize() {
                Ok(canon) => {
                    if !canon.starts_with(&canonical_base) {
                        return Err("Security Violation: Path escapes sandbox boundary!".into());
                    }
                    Ok(canon)
                }
                Err(_) => Err(format!("File not found in sandbox: {}", requested)),
            }
        } else {
            // For write, create parents inside sandbox and ensure no escaping
            if let Some(parent) = full_target.parent() {
                let _ = std::fs::create_dir_all(parent);
                if let Ok(canon_parent) = parent.canonicalize() {
                    if !canon_parent.starts_with(&canonical_base) {
                        return Err("Security Violation: Write path parent escapes sandbox boundary!".into());
                    }
                }
            }
            Ok(full_target)
        }
    }
}
