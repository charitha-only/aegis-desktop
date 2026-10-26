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
///     ↓ dispatch(name, payload) → IpcResponse
/// IpcRouter → plugin_dispatch IPC command
/// ```
use crate::security::SecurityEngine;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

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

// ─── Plugin Registry ──────────────────────────────────────────────────────

pub struct PluginRegistry {
    plugins: RwLock<HashMap<String, (PluginDescriptor, PluginHandlerFn)>>,
}

impl PluginRegistry {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            plugins: RwLock::new(HashMap::new()),
        })
    }

    /// Register a plugin with a name, descriptor, and handler function.
    pub fn register<F>(&self, descriptor: PluginDescriptor, handler: F)
    where
        F: Fn(&Value, &SecurityEngine) -> Result<Value, String> + Send + Sync + 'static,
    {
        let mut map = self.plugins.write().unwrap();
        map.insert(descriptor.name.clone(), (descriptor, Box::new(handler)));
        println!("[PluginEngine] ✅  Plugin registered: '{}'", map.len());
    }

    /// Dispatch a call to a registered plugin by name.
    pub fn dispatch(
        &self,
        name: &str,
        payload: &Value,
        security: &SecurityEngine,
    ) -> Result<Value, String> {
        let map = self.plugins.read().unwrap();
        match map.get(name) {
            Some((descriptor, handler)) => {
                for capability in &descriptor.capabilities {
                    match capability.as_str() {
                        "system_info" => security.check_system_info()?,
                        "crypto" => security.check_crypto()?,
                        "fs_read" if security.actual_capabilities()["fsRead"] == true => {}
                        _ => return Err("Plugin capability denied or unsupported".into()),
                    }
                }
                handler(payload, security)
            }
            None => Err(format!(
                "Plugin '{}' not found. Registered plugins: [{}]",
                name,
                map.keys().cloned().collect::<Vec<_>>().join(", ")
            )),
        }
    }

    /// List all registered plugin descriptors.
    pub fn list_plugins(&self) -> Vec<PluginDescriptor> {
        let map = self.plugins.read().unwrap();
        map.values().map(|(desc, _)| desc.clone()).collect()
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
pub fn register_builtin_plugins(registry: &Arc<PluginRegistry>) {
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
    );

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
    );

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
    );

    println!("[PluginEngine] 🔌  Built-in plugins registered (system_memory, file_checksum, process_uptime)");
}
