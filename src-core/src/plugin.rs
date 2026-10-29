/// Aegis Desktop — Custom Rust Plugin Engine (v0.13.0)
///
/// Allows developers to expose their own custom Rust system functions
/// through a composable capability manifest. Plugins are registered at
/// startup and invoked via standard IPC commands through the bridge.
///
/// # Architecture
/// ```
/// Plugin Registry (HashMap<name, PluginHandler>)
///     ↑ register_plugin(name, handler_fn)
///     ↓ dispatch(name, payload, security) → Result<Value, PluginError>
/// IpcRouter → plugin_dispatch IPC command
/// ```
use crate::security::SecurityEngine;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, RwLock,
};

// ─── Plugin Handler Trait ──────────────────────────────────────────────────

/// A plugin handler is any function that accepts a JSON payload and returns
/// a JSON result, with the active security context supplied by the router.
/// Native handlers are trusted host code, not an untrusted-code sandbox.
pub type PluginHandlerFn =
    Box<dyn Fn(&Value, &SecurityEngine) -> Result<Value, String> + Send + Sync>;

// ─── Plugin Descriptor ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDescriptor {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RegistrationError {
    InvalidDescriptor,
    RegistryFull,
    DuplicateName,
    RegistryUnavailable,
}
impl std::fmt::Display for RegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidDescriptor => "Invalid or unsupported plugin descriptor",
            Self::RegistryFull => "Plugin registry limit reached (128 registrations)",
            Self::DuplicateName => "Plugin name is already registered",
            Self::RegistryUnavailable => "Plugin registry is unavailable",
        })
    }
}
impl std::error::Error for RegistrationError {}

#[derive(Debug)]
pub enum PluginError {
    Rejected(String),
    Panicked,
    Disabled,
}
impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rejected(message) => f.write_str(message),
            Self::Panicked => f.write_str("Plugin panicked; handler disabled"),
            Self::Disabled => f.write_str("Plugin disabled after a previous panic"),
        }
    }
}
impl std::error::Error for PluginError {}

// ─── Plugin Registry ──────────────────────────────────────────────────────

struct PluginEntry {
    descriptor: PluginDescriptor,
    handler: PluginHandlerFn,
    faulted: AtomicBool,
}

pub struct PluginRegistry {
    plugins: RwLock<HashMap<String, Arc<PluginEntry>>>,
}

impl PluginRegistry {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            plugins: RwLock::new(HashMap::new()),
        })
    }

    /// Register a plugin with a name, descriptor, and handler function.
    pub fn register<F>(
        &self,
        descriptor: PluginDescriptor,
        handler: F,
    ) -> Result<(), RegistrationError>
    where
        F: Fn(&Value, &SecurityEngine) -> Result<Value, String> + Send + Sync + 'static,
    {
        let name = descriptor.name.as_bytes();
        let version = descriptor.version.as_bytes();
        let valid_text =
            |value: &str, limit| value.len() <= limit && !value.chars().any(char::is_control);
        let mut capabilities = std::collections::HashSet::new();
        if name.is_empty()
            || name.len() > 64
            || !name[0].is_ascii_lowercase()
            || !name
                .iter()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"_-".contains(b))
            || version.is_empty()
            || version.len() > 64
            || !version[0].is_ascii_digit()
            || !version
                .iter()
                .all(|b| b.is_ascii_alphanumeric() || b".+-".contains(b))
            || !valid_text(&descriptor.description, 1024)
            || !valid_text(&descriptor.author, 256)
            || descriptor.capabilities.iter().any(|grant| {
                !matches!(grant.as_str(), "system_info" | "crypto" | "fs_read")
                    || !capabilities.insert(grant)
            })
        {
            return Err(RegistrationError::InvalidDescriptor);
        }
        let mut map = self
            .plugins
            .write()
            .map_err(|_| RegistrationError::RegistryUnavailable)?;
        if map.contains_key(&descriptor.name) {
            return Err(RegistrationError::DuplicateName);
        }
        if map.len() >= 128 {
            return Err(RegistrationError::RegistryFull);
        }
        map.insert(
            descriptor.name.clone(),
            Arc::new(PluginEntry {
                descriptor,
                handler: Box::new(handler),
                faulted: AtomicBool::new(false),
            }),
        );
        println!("[PluginEngine] ✅  Plugin registered: '{}'", map.len());
        Ok(())
    }

    /// Dispatch a call to a registered plugin by name.
    pub fn dispatch(
        &self,
        name: &str,
        payload: &Value,
        security: &SecurityEngine,
    ) -> Result<Value, PluginError> {
        let entry = self.plugins.read().unwrap().get(name).cloned();
        match entry {
            Some(entry) => {
                let descriptor = &entry.descriptor;
                let handler = &entry.handler;
                let faulted = &entry.faulted;
                for capability in &descriptor.capabilities {
                    match capability.as_str() {
                        "system_info" => security
                            .check_system_info()
                            .map_err(PluginError::Rejected)?,
                        "crypto" => security.check_crypto().map_err(PluginError::Rejected)?,
                        "fs_read" if security.actual_capabilities()["fsRead"] == true => {}
                        _ => {
                            return Err(PluginError::Rejected(
                                "Plugin capability denied or unsupported".into(),
                            ))
                        }
                    }
                }
                if faulted.load(Ordering::SeqCst) {
                    return Err(PluginError::Disabled);
                }
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    handler(payload, security)
                })) {
                    Ok(result) => result.map_err(PluginError::Rejected),
                    Err(_) => {
                        faulted.store(true, Ordering::SeqCst);
                        Err(PluginError::Panicked)
                    }
                }
            }
            None => Err(PluginError::Rejected("Plugin not registered".into())),
        }
    }

    /// List all registered plugin descriptors.
    pub fn list_plugins(&self) -> Vec<PluginDescriptor> {
        let map = self.plugins.read().unwrap();
        map.values().map(|entry| entry.descriptor.clone()).collect()
    }

    /// List registrations disabled after a caught handler panic.
    pub fn disabled_plugins(&self) -> Vec<String> {
        let map = self.plugins.read().unwrap();
        let mut disabled: Vec<_> = map
            .iter()
            .filter(|(_, entry)| entry.faulted.load(Ordering::SeqCst))
            .map(|(name, _)| name.clone())
            .collect();
        disabled.sort_unstable();
        disabled
    }

    /// Check if a plugin exists by name.
    pub fn has_plugin(&self, name: &str) -> bool {
        let map = self.plugins.read().unwrap();
        map.contains_key(name)
    }
}

// ─── Built-in Demo Plugins ─────────────────────────────────────────────────

/// Register Aegis built-in demonstration plugins into the registry.
/// These illustrate how third-party developers can expose custom Rust
/// functionality through the Aegis plugin capability manifest.
pub fn register_builtin_plugins(registry: &Arc<PluginRegistry>) -> Result<(), RegistrationError> {
    // Plugin 1: System Memory Info
    registry.register(
        PluginDescriptor {
            name: "system_memory".to_string(),
            version: "1.0.0".to_string(),
            description: "Unavailable until a native memory statistics provider is configured."
                .to_string(),
            author: "Aegis Core".to_string(),
            capabilities: vec!["system_info".to_string()],
        },
        |_payload, _security| {
            Err("UNAVAILABLE: Native memory statistics provider is not configured".into())
        },
    )?;

    // Plugin 2: File Checksum Verifier
    registry.register(
        PluginDescriptor {
            name: "file_checksum".to_string(),
            version: "1.0.0".to_string(),
            description:
                "Computes SHA-256 checksum of any sandboxed file for integrity verification."
                    .to_string(),
            author: "Aegis Core".to_string(),
            capabilities: vec!["fs_read".to_string(), "crypto".to_string()],
        },
        |payload, security| {
            use sha2::{Digest, Sha256};
            let filename = payload
                .get("filename")
                .and_then(|f| f.as_str())
                .ok_or("Missing 'filename' parameter")?;

            use std::io::Read;
            let path = security.check_fs_read(filename)?;
            let metadata =
                std::fs::symlink_metadata(&path).map_err(|_| "Cannot inspect checksum file")?;
            if !metadata.is_file() || metadata.len() > crate::storage::MAX_FILE_BYTES as u64 {
                return Err(
                    "Checksum requires a regular file within the storage size limit".into(),
                );
            }
            let file = std::fs::File::open(&path).map_err(|_| "Cannot open checksum file")?;
            let mut reader = file.take(crate::storage::MAX_FILE_BYTES as u64 + 1);
            let mut hasher = Sha256::new();
            let mut buffer = zeroize::Zeroizing::new([0u8; 8192]);
            let mut size = 0usize;
            loop {
                let count = reader
                    .read(&mut *buffer)
                    .map_err(|_| "Cannot read checksum file")?;
                if count == 0 {
                    break;
                }
                size += count;
                if size > crate::storage::MAX_FILE_BYTES {
                    return Err("Checksum file exceeds size limit".into());
                }
                hasher.update(&buffer[..count]);
            }
            let hash = format!("{:x}", hasher.finalize());

            Ok(serde_json::json!({
                "plugin": "file_checksum",
                "filename": filename,
                "algorithm": "SHA-256",
                "checksum": hash,
                "sizeBytes": size
            }))
        },
    )?;

    // Plugin 3: elapsed time since built-in registration, using a monotonic clock.
    let started = std::time::Instant::now();
    registry.register(
        PluginDescriptor {
            name: "process_uptime".to_string(),
            version: "1.0.0".to_string(),
            description: "Returns elapsed seconds since built-in plugin registration.".to_string(),
            author: "Aegis Core".to_string(),
            capabilities: vec!["system_info".to_string()],
        },
        move |_payload, _security| {
            Ok(serde_json::json!({
                "plugin": "process_uptime",
                "uptimeSeconds": started.elapsed().as_secs_f64(),
                "measurementScope": "sinceBuiltinPluginRegistration"
            }))
        },
    )?;

    println!("[PluginEngine] 🔌  Built-in plugins registered (system_memory, file_checksum, process_uptime)");
    Ok(())
}
