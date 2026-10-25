use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
    sandbox_root: PathBuf,
}

impl SecurityEngine {
    pub fn new(manifest: CapabilityManifest, base_dir: &Path) -> Self {
        let sandbox_root = base_dir.join("sandbox");
        if !sandbox_root.exists() {
            let _ = std::fs::create_dir_all(&sandbox_root);
        }
        Self {
            manifest,
            sandbox_root,
        }
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
        // 1. Prevent traversal strings
        if requested.contains("..") {
            return Err("Security Violation: Directory traversal '..' detected!".into());
        }

        // 2. Deny blacklisted absolute prefixes
        for denied in &self.manifest.fs.deny_paths {
            if requested.to_lowercase().starts_with(&denied.to_lowercase()) {
                return Err(format!("Security Violation: Path matches denied pattern '{}'", denied));
            }
        }

        // 3. Resolve relative to sandbox
        let clean_path = requested.trim_start_matches("./").trim_start_matches("sandbox/");
        let full_target = self.sandbox_root.join(clean_path);

        // Normalize
        let canonical_base = self.sandbox_root.canonicalize().unwrap_or(self.sandbox_root.clone());
        
        // For read, path should exist
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
            // For write, verify parent directory
            if let Some(parent) = full_target.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            Ok(full_target)
        }
    }
}
