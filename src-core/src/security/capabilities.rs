use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsCapabilities {
    pub allow_read: Vec<String>,
    pub allow_write: Vec<String>,
    pub deny_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemCapabilities {
    #[serde(default)]
    pub allow_clipboard_write: bool,
    pub allow_system_info: bool,
    pub allow_dialogs: bool,
    pub allow_shell_execution: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
                allow_clipboard_write: false,
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

    pub fn actual_capabilities(&self) -> serde_json::Value {
        serde_json::json!({"fsRead":!self.manifest.fs.allow_read.is_empty(),"fsWrite":!self.manifest.fs.allow_write.is_empty(),"systemInfo":self.manifest.system.allow_system_info,"crypto":self.manifest.crypto.allow_hashing,"shellExecution":false})
    }

    pub fn check_fs_read(&self, requested_path: &str) -> Result<PathBuf, String> {
        if !path_allowed(requested_path, &self.manifest.fs.allow_read)
            || path_allowed(requested_path, &self.manifest.fs.deny_paths)
        {
            return Err("Filesystem read capability is disabled".into());
        }
        self.resolve_and_validate_path(requested_path, false)
    }

    pub fn check_fs_write(&self, requested_path: &str) -> Result<PathBuf, String> {
        if !path_allowed(requested_path, &self.manifest.fs.allow_write)
            || path_allowed(requested_path, &self.manifest.fs.deny_paths)
        {
            return Err("Filesystem write capability is disabled".into());
        }
        self.resolve_and_validate_path(requested_path, true)
    }

    pub fn check_system_info(&self) -> Result<(), String> {
        if self.manifest.system.allow_system_info {
            Ok(())
        } else {
            Err(
                "Security Violation: 'system_info' capability is disabled in aegis.config.json"
                    .into(),
            )
        }
    }

    pub fn check_dialogs(&self) -> Result<(), String> {
        if self.manifest.system.allow_dialogs {
            Ok(())
        } else {
            Err("Security Violation: 'dialogs' capability is disabled".into())
        }
    }

    pub fn check_clipboard_write(&self) -> Result<(), String> {
        if self.manifest.system.allow_clipboard_write {
            Ok(())
        } else {
            Err("Security Violation: 'clipboard_write' capability is disabled".into())
        }
    }

    pub fn check_shell(&self) -> Result<(), String> {
        if self.manifest.system.allow_shell_execution {
            Ok(())
        } else {
            Err(
                "Security Violation: Arbitrary shell execution is permanently BLOCKED by policy"
                    .into(),
            )
        }
    }

    pub fn check_crypto(&self) -> Result<(), String> {
        if self.manifest.crypto.allow_hashing {
            Ok(())
        } else {
            Err("Security Violation: 'crypto' capability is disabled".into())
        }
    }

    pub fn check_write_quota(&self, path: &str, bytes: usize, limit: u64) -> Result<(), String> {
        fn usage(dir: &Path) -> Result<u64, String> {
            let mut total = 0u64;
            for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let meta = entry.path().symlink_metadata().map_err(|e| e.to_string())?;
                if meta.file_type().is_symlink() {
                    return Err("Linked storage paths are forbidden".into());
                }
                total = total
                    .checked_add(if meta.is_dir() {
                        usage(&entry.path())?
                    } else {
                        meta.len()
                    })
                    .ok_or("Storage quota overflow")?;
            }
            Ok(total)
        }
        let target = self.check_fs_write(path)?;
        let old = std::fs::metadata(target).map_or(0, |m| m.len());
        let used = usage(&self.get_sandbox_root())?;
        if used.saturating_sub(old).saturating_add(bytes as u64) > limit {
            return Err("Profile storage quota exceeded".into());
        }
        Ok(())
    }

    fn resolve_and_validate_path(&self, requested: &str, write: bool) -> Result<PathBuf, String> {
        use std::path::Component;
        if requested.contains(':') || requested.contains('\\') || requested.contains('\0') {
            return Err("Security Violation: invalid path syntax".into());
        }
        let clean = requested.strip_prefix("./").unwrap_or(requested);
        let clean = clean.strip_prefix("sandbox/").unwrap_or(clean);
        let path = Path::new(clean);
        if path
            .components()
            .any(|c| !matches!(c, Component::Normal(_) | Component::CurDir))
        {
            return Err("Security Violation: only relative sandbox paths are allowed".into());
        }
        let root = self
            .get_sandbox_root()
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let mut target = root.clone();
        let components: Vec<_> = path.components().collect();
        for (i, component) in components.iter().enumerate() {
            if let Component::Normal(part) = component {
                let text = part.to_string_lossy();
                let device = text.split('.').next().unwrap_or("").to_ascii_uppercase();
                if text.starts_with(".aegis-")
                    || matches!(
                        device.as_str(),
                        "CON"
                            | "PRN"
                            | "AUX"
                            | "NUL"
                            | "COM1"
                            | "COM2"
                            | "COM3"
                            | "COM4"
                            | "COM5"
                            | "COM6"
                            | "COM7"
                            | "COM8"
                            | "COM9"
                            | "LPT1"
                            | "LPT2"
                            | "LPT3"
                            | "LPT4"
                            | "LPT5"
                            | "LPT6"
                            | "LPT7"
                            | "LPT8"
                            | "LPT9"
                    )
                {
                    return Err("Security Violation: reserved storage name".into());
                }
                if text.ends_with('.') || text.ends_with(' ') {
                    return Err("Security Violation: ambiguous path".into());
                }
                target.push(part);
            }
            match std::fs::symlink_metadata(&target) {
                Ok(meta) => {
                    if meta.file_type().is_symlink() {
                        return Err("Security Violation: linked sandbox paths are forbidden".into());
                    }
                    let canon = target.canonicalize().map_err(|e| e.to_string())?;
                    if !canon.starts_with(&root) {
                        return Err("Security Violation: path escapes sandbox".into());
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound && write => {
                    if i + 1 < components.len() {
                        std::fs::create_dir(&target).map_err(|e| e.to_string())?;
                    }
                }
                Err(e) => return Err(format!("File not found in sandbox: {e}")),
            }
        }
        Ok(target)
    }
}

fn path_allowed(requested: &str, patterns: &[String]) -> bool {
    let clean = requested.strip_prefix("./").unwrap_or(requested);
    let clean = clean.strip_prefix("sandbox/").unwrap_or(clean);
    patterns.iter().any(|rule| {
        let r = rule.strip_prefix("./").unwrap_or(rule);
        if r == "sandbox" || r == "sandbox/**" {
            return true;
        }
        let Some(r) = r.strip_prefix("sandbox/") else {
            return false;
        };
        if let Some(prefix) = r.strip_suffix("/**") {
            clean == prefix || clean.starts_with(&format!("{prefix}/"))
        } else {
            r == clean
        }
    })
}
