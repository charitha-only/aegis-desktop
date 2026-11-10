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
use sha2::Digest;
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

// ─── Sandboxed Plugin Manifest & Capability Contracts (v0.30.9 / v1.0.0-rc1) ───

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginTrustTier {
    NativeHost,
    SandboxedGuest,
}

impl std::fmt::Display for PluginTrustTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NativeHost => write!(f, "native_host"),
            Self::SandboxedGuest => write!(f, "sandboxed_guest"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginPermission {
    #[serde(rename = "system:info")]
    SystemInfo,
    #[serde(rename = "crypto:sha256")]
    CryptoSha256,
    #[serde(rename = "storage:read")]
    StorageRead,
    #[serde(rename = "storage:write")]
    StorageWrite,
    #[serde(rename = "ui:dialog")]
    UiDialog,
    #[serde(rename = "net:fetch")]
    NetFetch,
}

impl PluginPermission {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SystemInfo => "system:info",
            Self::CryptoSha256 => "crypto:sha256",
            Self::StorageRead => "storage:read",
            Self::StorageWrite => "storage:write",
            Self::UiDialog => "ui:dialog",
            Self::NetFetch => "net:fetch",
        }
    }

    pub fn from_str_strict(s: &str) -> Result<Self, String> {
        match s {
            "system:info" | "system_info" => Ok(Self::SystemInfo),
            "crypto:sha256" | "crypto" => Ok(Self::CryptoSha256),
            "storage:read" | "fs_read" => Ok(Self::StorageRead),
            "storage:write" | "fs_write" => Ok(Self::StorageWrite),
            "ui:dialog" | "dialog" => Ok(Self::UiDialog),
            "net:fetch" | "net" => Ok(Self::NetFetch),
            unknown => Err(format!("Unknown or unsupported permission grant: '{unknown}'")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginResourceLimits {
    pub max_memory_mb: u32,
    pub max_fuel: u64,
}

impl Default for PluginResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 16,
            max_fuel: 100_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub trust_tier: PluginTrustTier,
    pub permissions: Vec<String>,
    #[serde(default)]
    pub resource_limits: PluginResourceLimits,
    #[serde(default)]
    pub entrypoint: Option<String>,
}

impl PluginManifest {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty()
            || self.id.len() > 128
            || !self
                .id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        {
            return Err("Plugin manifest ID must be 1-128 ASCII letters, digits, dots, underscores or hyphens".into());
        }
        if self.name.is_empty() || self.name.len() > 128 || self.name.chars().any(char::is_control) {
            return Err("Plugin name must be 1-128 characters without control codes".into());
        }
        if self.version.is_empty() || self.version.len() > 64 || self.version.chars().any(char::is_control) {
            return Err("Plugin version must be 1-64 characters without control codes".into());
        }
        if self.description.len() > 1024 || self.description.chars().any(char::is_control) {
            return Err("Plugin description cannot exceed 1024 characters or contain control codes".into());
        }
        if self.author.len() > 256 || self.author.chars().any(char::is_control) {
            return Err("Plugin author cannot exceed 256 characters or contain control codes".into());
        }
        if self.resource_limits.max_memory_mb == 0 || self.resource_limits.max_memory_mb > 64 {
            return Err("Plugin memory limit must be between 1 MB and 64 MB".into());
        }
        if self.resource_limits.max_fuel < 1_000 || self.resource_limits.max_fuel > 10_000_000 {
            return Err("Plugin instruction fuel limit must be between 1,000 and 10,000,000 instructions".into());
        }

        let mut seen = std::collections::HashSet::new();
        for perm in &self.permissions {
            PluginPermission::from_str_strict(perm)?;
            if !seen.insert(perm.clone()) {
                return Err(format!("Duplicate permission declared in manifest: '{perm}'"));
            }
        }
        Ok(())
    }

    pub fn has_permission(&self, perm: &str) -> bool {
        let normalized = match perm {
            "system:info" | "system_info" => "system:info",
            "crypto:sha256" | "crypto" => "crypto:sha256",
            "storage:read" | "fs_read" => "storage:read",
            "storage:write" | "fs_write" => "storage:write",
            "ui:dialog" | "dialog" => "ui:dialog",
            "net:fetch" | "net" => "net:fetch",
            other => other,
        };
        self.permissions.iter().any(|p| {
            let p_norm = match p.as_str() {
                "system:info" | "system_info" => "system:info",
                "crypto:sha256" | "crypto" => "crypto:sha256",
                "storage:read" | "fs_read" => "storage:read",
                "storage:write" | "fs_write" => "storage:write",
                "ui:dialog" | "dialog" => "ui:dialog",
                "net:fetch" | "net" => "net:fetch",
                other => other,
            };
            p_norm == normalized
        })
    }
}

pub struct SandboxedPluginManager {
    manifests: RwLock<HashMap<String, PluginManifest>>,
    faulted: RwLock<HashMap<String, bool>>,
}

impl SandboxedPluginManager {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            manifests: RwLock::new(HashMap::new()),
            faulted: RwLock::new(HashMap::new()),
        })
    }

    pub fn register_manifest(&self, manifest: PluginManifest) -> Result<(), String> {
        manifest.validate()?;
        let mut map = self.manifests.write().map_err(|_| "Plugin manifest lock poisoned")?;
        if map.len() >= 64 && !map.contains_key(&manifest.id) {
            return Err("Sandboxed plugin registry quota reached (64 plugins)".into());
        }
        map.insert(manifest.id.clone(), manifest);
        Ok(())
    }

    pub fn get_manifest(&self, plugin_id: &str) -> Option<PluginManifest> {
        self.manifests.read().ok()?.get(plugin_id).cloned()
    }

    pub fn list_manifests(&self) -> Vec<PluginManifest> {
        let mut list: Vec<PluginManifest> = self
            .manifests
            .read()
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default();
        list.sort_by(|a, b| a.id.cmp(&b.id));
        list
    }

    pub fn is_faulted(&self, plugin_id: &str) -> bool {
        self.faulted
            .read()
            .ok()
            .and_then(|f| f.get(plugin_id).copied())
            .unwrap_or(false)
    }

    pub fn execute_sandboxed_action(
        &self,
        plugin_id: &str,
        action: &str,
        payload: &Value,
        security: &SecurityEngine,
        wasm: &crate::wasm::WasmSandbox,
    ) -> Result<Value, String> {
        let manifest = self
            .get_manifest(plugin_id)
            .ok_or_else(|| format!("Sandboxed plugin '{plugin_id}' is not registered"))?;

        if self.is_faulted(plugin_id) {
            return Err(format!("Sandboxed plugin '{plugin_id}' is disabled due to a previous crash/trap"));
        }

        match action {
            "crypto_sha256" | "hash" => {
                if !manifest.has_permission("crypto:sha256") {
                    return Err(format!(
                        "PERMISSION_DENIED: Plugin '{}' requires declared permission 'crypto:sha256'",
                        plugin_id
                    ));
                }
                security.check_crypto()?;
                let input_str = payload
                    .get("data")
                    .and_then(|d| d.as_str())
                    .ok_or("Missing 'data' string parameter in payload")?;
                use sha2::{Digest, Sha256};
                let hash = hex::encode(Sha256::digest(input_str.as_bytes()));
                Ok(serde_json::json!({
                    "pluginId": plugin_id,
                    "action": action,
                    "algorithm": "SHA-256",
                    "hash": hash,
                    "bytesHashed": input_str.len()
                }))
            }
            "system_info" => {
                if !manifest.has_permission("system:info") {
                    return Err(format!(
                        "PERMISSION_DENIED: Plugin '{}' requires declared permission 'system:info'",
                        plugin_id
                    ));
                }
                security.check_system_info()?;
                Ok(serde_json::json!({
                    "pluginId": plugin_id,
                    "action": action,
                    "tier": manifest.trust_tier.to_string(),
                    "status": "AUTHORIZED",
                    "runtime": "aegis-sandbox"
                }))
            }
            "storage_read" | "read_file" => {
                if !manifest.has_permission("storage:read") {
                    return Err(format!(
                        "PERMISSION_DENIED: Plugin '{}' requires declared permission 'storage:read'",
                        plugin_id
                    ));
                }
                let filename = payload
                    .get("filename")
                    .and_then(|f| f.as_str())
                    .ok_or("Missing 'filename' parameter")?;
                let path = security.check_fs_read(filename)?;
                let meta = std::fs::symlink_metadata(&path).map_err(|e| format!("Cannot inspect file: {e}"))?;
                if !meta.is_file() || meta.len() > crate::storage::MAX_FILE_BYTES as u64 {
                    return Err("Storage read requires a regular file within storage size limits".into());
                }
                let bytes = std::fs::read(&path).map_err(|e| format!("Failed to read file: {e}"))?;
                Ok(serde_json::json!({
                    "pluginId": plugin_id,
                    "action": action,
                    "filename": filename,
                    "bytesRead": bytes.len(),
                    "sha256": hex::encode(sha2::Sha256::digest(&bytes))
                }))
            }
            "wasm_compute" | "execute_guest" => {
                let module_id = payload
                    .get("moduleId")
                    .and_then(|m| m.as_str())
                    .unwrap_or(plugin_id);
                let guest_input = payload.get("input").unwrap_or(&Value::Null);

                match wasm.execute_module_with_tier(module_id, guest_input, crate::wasm::ExecutionTier::Auto) {
                    Ok(exec_res) => Ok(serde_json::to_value(exec_res).unwrap_or(Value::Null)),
                    Err(trap_err) => {
                        if let Ok(mut faulted_map) = self.faulted.write() {
                            faulted_map.insert(plugin_id.to_string(), true);
                        }
                        Err(format!("GUEST_TRAP_CONTAINED: Wasm execution faulted: {trap_err}"))
                    }
                }
            }
            unknown => Err(format!(
                "UNSUPPORTED_ACTION: Sandboxed plugin action '{unknown}' is not supported or requires undeclared capability"
            )),
        }
    }
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
    sandboxed: Arc<SandboxedPluginManager>,
}

impl PluginRegistry {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            plugins: RwLock::new(HashMap::new()),
            sandboxed: SandboxedPluginManager::new(),
        })
    }

    pub fn sandboxed(&self) -> &Arc<SandboxedPluginManager> {
        &self.sandboxed
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
    #[allow(dead_code)]
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

    // Register Built-in Sandboxed Plugin Manifests (Zero-Trust Model)
    let _ = registry.sandboxed().register_manifest(PluginManifest {
        id: "demo_sha256".to_string(),
        name: "Demo SHA-256 Hasher".to_string(),
        version: "1.0.0".to_string(),
        description: "Sandboxed guest plugin with declared crypto:sha256 permission.".to_string(),
        author: "Aegis Core".to_string(),
        trust_tier: PluginTrustTier::SandboxedGuest,
        permissions: vec!["crypto:sha256".to_string()],
        resource_limits: PluginResourceLimits::default(),
        entrypoint: None,
    });

    let _ = registry.sandboxed().register_manifest(PluginManifest {
        id: "demo_restricted".to_string(),
        name: "Restricted Zero-Trust Plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "Sandboxed guest plugin with zero declared permissions (denies all privileged host actions).".to_string(),
        author: "Aegis Core".to_string(),
        trust_tier: PluginTrustTier::SandboxedGuest,
        permissions: vec![],
        resource_limits: PluginResourceLimits::default(),
        entrypoint: None,
    });

    println!("[PluginEngine] 🔌  Built-in plugins registered (system_memory, file_checksum, process_uptime, demo_sha256, demo_restricted)");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandboxed_plugin_manifest_validation() {
        let mut manifest = PluginManifest {
            id: "valid_id.1-a".to_string(),
            name: "Valid Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A valid plugin manifest".to_string(),
            author: "Author".to_string(),
            trust_tier: PluginTrustTier::SandboxedGuest,
            permissions: vec!["crypto:sha256".to_string(), "system:info".to_string()],
            resource_limits: PluginResourceLimits::default(),
            entrypoint: None,
        };
        assert!(manifest.validate().is_ok());

        // Test invalid ID with spaces
        manifest.id = "invalid id with spaces".to_string();
        assert!(manifest.validate().is_err());
        manifest.id = "valid_id".to_string();

        // Test duplicate permission
        manifest.permissions = vec!["crypto:sha256".to_string(), "crypto:sha256".to_string()];
        assert!(manifest.validate().is_err());

        // Test unknown permission
        manifest.permissions = vec!["unknown:super_privilege".to_string()];
        assert!(manifest.validate().is_err());

        // Test invalid resource limits
        manifest.permissions = vec!["crypto:sha256".to_string()];
        manifest.resource_limits.max_memory_mb = 128; // exceeds 64 MB
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_sandboxed_plugin_permission_denial() {
        let registry = PluginRegistry::new();
        let _ = register_builtin_plugins(&registry);

        let manager = registry.sandboxed();
        let temp_dir = std::env::temp_dir();
        let sec = SecurityEngine::new(
            crate::security::capabilities::CapabilityManifest::default(),
            &temp_dir,
        );
        let wasm = crate::wasm::WasmSandbox::new();

        // demo_restricted has NO declared permissions
        let denied = manager.execute_sandboxed_action(
            "demo_restricted",
            "crypto_sha256",
            &serde_json::json!({ "data": "secret" }),
            &sec,
            &wasm,
        );
        assert!(denied.is_err());
        let err_msg = denied.unwrap_err();
        assert!(err_msg.contains("PERMISSION_DENIED"), "Expected permission denial, got: {err_msg}");

        // demo_sha256 HAS declared crypto:sha256 permission
        let allowed = manager.execute_sandboxed_action(
            "demo_sha256",
            "crypto_sha256",
            &serde_json::json!({ "data": "hello aegis" }),
            &sec,
            &wasm,
        );
        assert!(allowed.is_ok(), "Expected success, got: {:?}", allowed.err());
        let res = allowed.unwrap();
        assert_eq!(res["algorithm"], "SHA-256");
        assert_eq!(
            res["hash"],
            "b934e58605f631fb166f4c10189c01df9bae28966340038839d84c70fa0b2df0"
        );
    }

    #[test]
    fn test_sandboxed_plugin_guest_trap_containment() {
        let registry = PluginRegistry::new();
        let _ = register_builtin_plugins(&registry);
        let manager = registry.sandboxed();
        let temp_dir = std::env::temp_dir();
        let sec = SecurityEngine::new(
            crate::security::capabilities::CapabilityManifest::default(),
            &temp_dir,
        );
        let wasm = crate::wasm::WasmSandbox::new();

        let trap_manifest = PluginManifest {
            id: "faulting_guest".to_string(),
            name: "Faulting Guest".to_string(),
            version: "1.0.0".to_string(),
            description: "Simulates a faulting Wasm guest".to_string(),
            author: "Tester".to_string(),
            trust_tier: PluginTrustTier::SandboxedGuest,
            permissions: vec![],
            resource_limits: PluginResourceLimits::default(),
            entrypoint: None,
        };
        assert!(manager.register_manifest(trap_manifest).is_ok());

        let res = manager.execute_sandboxed_action(
            "faulting_guest",
            "wasm_compute",
            &serde_json::json!({ "moduleId": "nonexistent-module" }),
            &sec,
            &wasm,
        );
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("GUEST_TRAP_CONTAINED"));

        let follow_up = manager.execute_sandboxed_action(
            "faulting_guest",
            "wasm_compute",
            &serde_json::json!({ "moduleId": "nonexistent-module" }),
            &sec,
            &wasm,
        );
        assert!(follow_up.is_err());
        assert!(follow_up
            .unwrap_err()
            .contains("disabled due to a previous crash/trap"));
    }
}
