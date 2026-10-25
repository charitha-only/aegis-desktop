/// Aegis Desktop - Multi-Profile Isolation Subsystem (v0.14.0)
///
/// Provides sandboxed workspace isolation for multiple profiles (e.g., Default, Work, Private).
/// Each profile maintains its own dedicated sandbox directory, config overrides, and privacy policies.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub sandbox_dir: String,
    pub created_at: u64,
    pub strict_isolation: bool,
    pub max_storage_bytes: u64,
}

pub struct ProfileManager {
    base_dir: PathBuf,
    profiles: Mutex<HashMap<String, ProfileConfig>>,
    active_profile_id: Mutex<String>,
}

impl ProfileManager {
    pub fn new(base_dir: &Path) -> Self {
        let mut profiles = HashMap::new();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        profiles.insert(
            "default".to_string(),
            ProfileConfig {
                id: "default".to_string(),
                name: "Primary Profile".to_string(),
                description: "Standard isolated user sandbox with default security boundaries."
                    .to_string(),
                sandbox_dir: "sandbox".to_string(),
                created_at: now,
                strict_isolation: true,
                max_storage_bytes: 50 * 1024 * 1024, // 50MB
            },
        );

        profiles.insert(
            "private_vault".to_string(),
            ProfileConfig {
                id: "private_vault".to_string(),
                name: "High-Security Vault Profile".to_string(),
                description: "Ephemeral, strict zero-leakage sandbox for sensitive credentials."
                    .to_string(),
                sandbox_dir: "sandbox_vault".to_string(),
                created_at: now,
                strict_isolation: true,
                max_storage_bytes: 10 * 1024 * 1024, // 10MB
            },
        );

        let manager = Self {
            base_dir: base_dir.to_path_buf(),
            profiles: Mutex::new(profiles),
            active_profile_id: Mutex::new("default".to_string()),
        };
        if let Ok(text) = fs::read_to_string(base_dir.join("profiles.json")) {
            if let Ok(saved) = serde_json::from_str::<Vec<ProfileConfig>>(&text) {
                let mut lock = manager.profiles.lock().unwrap();
                for cfg in saved {
                    if valid_id(&cfg.id)
                        && cfg.id != "default"
                        && cfg.id != "private_vault"
                        && cfg.sandbox_dir == format!("sandbox_{}", cfg.id)
                        && !lock.values().any(|existing| {
                            existing.id.eq_ignore_ascii_case(&cfg.id)
                                || existing.sandbox_dir.eq_ignore_ascii_case(&cfg.sandbox_dir)
                        })
                    {
                        lock.insert(cfg.id.clone(), cfg);
                    }
                }
            }
        }
        manager
    }

    pub fn get_active_sandbox_path(&self) -> PathBuf {
        let active = self.get_active_profile();
        let p = self.base_dir.join(&active.sandbox_dir);
        if !p.exists() {
            let _ = fs::create_dir_all(&p);
        }
        p
    }

    pub fn list_profiles(&self) -> Vec<ProfileConfig> {
        let p = self.profiles.lock().unwrap();
        p.values().cloned().collect()
    }

    pub fn get_active_profile(&self) -> ProfileConfig {
        let active_id = self.active_profile_id.lock().unwrap().clone();
        let p = self.profiles.lock().unwrap();
        p.get(&active_id).cloned().unwrap_or_else(|| ProfileConfig {
            id: "default".to_string(),
            name: "Primary Profile".to_string(),
            description: "Default fallback".to_string(),
            sandbox_dir: "sandbox".to_string(),
            created_at: 0,
            strict_isolation: true,
            max_storage_bytes: 50 * 1024 * 1024,
        })
    }

    pub fn switch_profile(&self, id: &str) -> Result<ProfileConfig, String> {
        let p = self.profiles.lock().unwrap();
        if let Some(target) = p.get(id) {
            let mut active = self.active_profile_id.lock().unwrap();
            *active = id.to_string();

            // Ensure profile sandbox directory exists
            let path = self.base_dir.join(&target.sandbox_dir);
            if !path.exists() {
                let _ = fs::create_dir_all(&path);
            }

            Ok(target.clone())
        } else {
            Err(format!("Profile '{}' not found in registry", id))
        }
    }

    pub fn create_profile(
        &self,
        id: &str,
        name: &str,
        description: &str,
    ) -> Result<ProfileConfig, String> {
        if !valid_id(id) {
            return Err("Profile ID must be 1-48 ASCII letters, digits or underscores".into());
        }
        let mut p = self.profiles.lock().unwrap();
        if p.len() >= 32 {
            return Err("Profile quota exceeded".into());
        }
        if p.keys().any(|existing| existing.eq_ignore_ascii_case(id)) {
            return Err(format!("Profile ID '{}' already exists", id));
        }

        let sandbox_dir = format!(
            "sandbox_{}",
            id.replace(|c: char| !c.is_alphanumeric(), "_")
        );
        if p.values()
            .any(|existing| existing.sandbox_dir.eq_ignore_ascii_case(&sandbox_dir))
        {
            return Err("Profile storage directory is already assigned".into());
        }
        let p_path = self.base_dir.join(&sandbox_dir);
        let _ = fs::create_dir_all(&p_path);

        let cfg = ProfileConfig {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            sandbox_dir,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            strict_isolation: true,
            max_storage_bytes: 25 * 1024 * 1024,
        };

        let mut saved: Vec<_> = p.values().cloned().collect();
        saved.push(cfg.clone());
        let data = serde_json::to_vec_pretty(&saved).map_err(|e| e.to_string())?;
        crate::storage::atomic_write(&self.base_dir.join("profiles.json"), &data, false)?;
        p.insert(id.to_string(), cfg.clone());
        Ok(cfg)
    }
}

fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 48 && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}
