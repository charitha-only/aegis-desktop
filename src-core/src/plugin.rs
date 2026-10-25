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

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// ─── Plugin Handler Trait ──────────────────────────────────────────────────

/// A plugin handler is any function that accepts a JSON payload and returns
/// a JSON value result. Plugins are stateless by design.
pub type PluginHandlerFn = Box<dyn Fn(&Value) -> Result<Value, String> + Send + Sync>;

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
        F: Fn(&Value) -> Result<Value, String> + Send + Sync + 'static,
    {
        let mut map = self.plugins.write().unwrap();
        map.insert(descriptor.name.clone(), (descriptor, Box::new(handler)));
        println!("[PluginEngine] ✅  Plugin registered: '{}'", map.len());
    }

    /// Dispatch a call to a registered plugin by name.
    pub fn dispatch(&self, name: &str, payload: &Value) -> Result<Value, String> {
        let map = self.plugins.read().unwrap();
        match map.get(name) {
            Some((_, handler)) => handler(payload),
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
            description: "Reports available and total system memory statistics.".to_string(),
            author: "Aegis Core".to_string(),
            capabilities: vec!["system_info".to_string()],
        },
        |_payload| {
            // In a real plugin, this would use platform APIs (winapi, libc, etc.)
            Ok(serde_json::json!({
                "plugin": "system_memory",
                "totalMB": 16384,
                "availableMB": 8192,
                "usagePercent": 50.0,
                "note": "Simulated — integrate with winapi/libc for real values"
            }))
        },
    );

    // Plugin 2: File Checksum Verifier
    registry.register(
        PluginDescriptor {
            name: "file_checksum".to_string(),
            version: "1.0.0".to_string(),
            description: "Computes SHA-256 checksum of any sandboxed file for integrity verification.".to_string(),
            author: "Aegis Core".to_string(),
            capabilities: vec!["fs_read".to_string(), "crypto".to_string()],
        },
        |payload| {
            use sha2::{Digest, Sha256};
            let filename = payload
                .get("filename")
                .and_then(|f| f.as_str())
                .ok_or("Missing 'filename' parameter")?;

            // Read file content from sandbox
            let path = std::path::Path::new("./sandbox").join(filename);
            let content = std::fs::read(&path)
                .map_err(|e| format!("Cannot read file '{}': {}", filename, e))?;

            let mut hasher = Sha256::new();
            hasher.update(&content);
            let hash = format!("{:x}", hasher.finalize());

            Ok(serde_json::json!({
                "plugin": "file_checksum",
                "filename": filename,
                "algorithm": "SHA-256",
                "checksum": hash,
                "sizeBytes": content.len()
            }))
        },
    );

    // Plugin 3: Process Uptime Reporter
    registry.register(
        PluginDescriptor {
            name: "process_uptime".to_string(),
            version: "1.0.0".to_string(),
            description: "Returns the current Aegis Core process uptime in seconds.".to_string(),
            author: "Aegis Core".to_string(),
            capabilities: vec!["system_info".to_string()],
        },
        |_payload| {
            use std::time::{SystemTime, UNIX_EPOCH};
            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            Ok(serde_json::json!({
                "plugin": "process_uptime",
                "currentTimestampUnix": ts,
                "note": "Track startup timestamp to compute real uptime"
            }))
        },
    );

    println!("[PluginEngine] 🔌  Built-in plugins registered (system_memory, file_checksum, process_uptime)");
}
