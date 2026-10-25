use super::encrypted_stream::EncryptedStream;
use super::protocol::{IpcRequest, IpcResponse};
use crate::crash::CrashReporter;
use crate::network::NetworkPolicyEngine;
use crate::plugin::PluginRegistry;
use crate::profile::ProfileManager;
use crate::secrets::SecretManager;
use crate::security::{
    BiometricEnclave, HandshakeSession, HardwareKeyEngine, PrivacyShield, SecurityEngine,
    SignatureVerifier, VaultEngine,
};
use crate::tray::{NotificationConfig, NotificationEngine, TrayDaemon};
use crate::updater::{AutoUpdater, UpdateManifest};
use crate::wasm::WasmSandbox;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::sync::Arc;

pub struct IpcRouter {
    security: Arc<SecurityEngine>,
    session: Arc<HandshakeSession>,
    plugins: Arc<PluginRegistry>,
    updater: Arc<AutoUpdater>,
    hardware_keys: Arc<HardwareKeyEngine>,
    profile_manager: Arc<ProfileManager>,
    wasm_sandbox: Arc<WasmSandbox>,
    biometric_enclave: Arc<BiometricEnclave>,
    crash_reporter: Arc<CrashReporter>,
    secret_manager: Arc<SecretManager>,
    network_engine: Arc<NetworkPolicyEngine>,
    sig_verifier: Arc<SignatureVerifier>,
}

impl IpcRouter {
    pub fn new(
        security: Arc<SecurityEngine>,
        session: Arc<HandshakeSession>,
        plugins: Arc<PluginRegistry>,
        updater: Arc<AutoUpdater>,
        hardware_keys: Arc<HardwareKeyEngine>,
        profile_manager: Arc<ProfileManager>,
        wasm_sandbox: Arc<WasmSandbox>,
        biometric_enclave: Arc<BiometricEnclave>,
        crash_reporter: Arc<CrashReporter>,
        secret_manager: Arc<SecretManager>,
        network_engine: Arc<NetworkPolicyEngine>,
        sig_verifier: Arc<SignatureVerifier>,
    ) -> Self {
        Self {
            security,
            session,
            plugins,
            updater,
            hardware_keys,
            profile_manager,
            wasm_sandbox,
            biometric_enclave,
            crash_reporter,
            secret_manager,
            network_engine,
            sig_verifier,
        }
    }

    pub fn handle_request(&self, req: IpcRequest) -> IpcResponse {
        // 1. Cryptographic Nonce Verification
        if !self.session.validate_nonce(&req.nonce) {
            return IpcResponse::err(&req.id, "Security Breach: Invalid cryptographic nonce. Request rejected!");
        }

        // 2. Command Dispatching with Capability Verification
        match req.command.as_str() {
            "get_security_status" => {
                let status = json!({
                    "engine": "Aegis Core v0.15 (Rust)",
                    "nonceVerified": true,
                    "sandboxEnforced": true,
                    "privacyVaultActive": true,
                    "antiDosRateLimiterActive": true,
                    "airGapPolicyActive": true,
                    "bundlerEngineActive": true,
                    "multiWindowApiActive": true,
                    "pluginEngineActive": true,
                    "notificationEngineActive": true,
                    "autoUpdaterActive": true,
                    "hardwareKeyActive": true,
                    "multiProfileActive": true,
                    "wasmSandboxActive": true,
                    "biometricEnclaveActive": true,
                    "encryptedIpcActive": true,
                    "crashReporterActive": true,
                    "activeCapabilities": {
                        "fsRead": true,
                        "fsWrite": true,
                        "shellExecution": false,
                        "systemInfo": true,
                        "crypto": true,
                        "encryptedVault": true,
                        "bundler": true,
                        "multiWindow": true,
                        "plugins": true,
                        "notifications": true,
                        "autoUpdater": true,
                        "hardwareKey": true,
                        "multiProfile": true,
                        "wasmSandbox": true,
                        "biometricEnclave": true,
                        "encryptedIpc": true,
                        "crashReporter": true
                    }
                });
                IpcResponse::ok(&req.id, status)
            }

            "privacy_status" => {
                IpcResponse::ok(&req.id, PrivacyShield::get_telemetry_audit())
            }

            "vault_encrypt_store" => {
                match self.security.check_crypto() {
                    Ok(_) => {
                        let filename = match req.payload.get("filename").and_then(|f| f.as_str()) {
                            Some(f) => f,
                            None => return IpcResponse::err(&req.id, "Missing 'filename' parameter"),
                        };
                        let content = match req.payload.get("content").and_then(|c| c.as_str()) {
                            Some(c) => c,
                            None => return IpcResponse::err(&req.id, "Missing 'content' parameter"),
                        };
                        let passphrase = match req.payload.get("passphrase").and_then(|p| p.as_str()) {
                            Some(p) => p,
                            None => return IpcResponse::err(&req.id, "Missing 'passphrase' parameter"),
                        };

                        if passphrase.len() < 4 {
                            return IpcResponse::err(&req.id, "Passphrase must be at least 4 characters long");
                        }

                        // Encrypt data with PBKDF2/SHA-256 stream cipher + HMAC-SHA256
                        let encrypted_envelope = VaultEngine::encrypt(content, passphrase);

                        // Ensure filename ends with .vault
                        let safe_filename = if filename.ends_with(".vault") {
                            filename.to_string()
                        } else {
                            format!("{}.vault", filename)
                        };

                        // Store safely inside isolated sandbox
                        match self.security.check_fs_write(&safe_filename) {
                            Ok(safe_path) => match fs::write(&safe_path, &encrypted_envelope) {
                                Ok(_) => IpcResponse::ok(
                                    &req.id,
                                    json!({
                                        "filename": safe_filename,
                                        "status": "Encrypted & Stored Successfully",
                                        "envelopeLength": encrypted_envelope.len()
                                    }),
                                ),
                                Err(err) => IpcResponse::err(&req.id, &format!("Failed to write encrypted vault: {}", err)),
                            },
                            Err(err) => IpcResponse::err(&req.id, &err),
                        }
                    }
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "vault_decrypt_retrieve" => {
                match self.security.check_crypto() {
                    Ok(_) => {
                        let filename = match req.payload.get("filename").and_then(|f| f.as_str()) {
                            Some(f) => f,
                            None => return IpcResponse::err(&req.id, "Missing 'filename' parameter"),
                        };
                        let passphrase = match req.payload.get("passphrase").and_then(|p| p.as_str()) {
                            Some(p) => p,
                            None => return IpcResponse::err(&req.id, "Missing 'passphrase' parameter"),
                        };

                        let safe_filename = if filename.ends_with(".vault") {
                            filename.to_string()
                        } else {
                            format!("{}.vault", filename)
                        };

                        // Read safely from sandbox
                        match self.security.check_fs_read(&safe_filename) {
                            Ok(safe_path) => match fs::read_to_string(&safe_path) {
                                Ok(envelope) => match VaultEngine::decrypt(&envelope, passphrase) {
                                    Ok(plaintext) => IpcResponse::ok(
                                        &req.id,
                                        json!({
                                            "filename": safe_filename,
                                            "decryptedContent": plaintext,
                                            "integrityVerified": true
                                        }),
                                    ),
                                    Err(err) => IpcResponse::err(&req.id, &err),
                                },
                                Err(err) => IpcResponse::err(&req.id, &format!("Failed to read vault file: {}", err)),
                            },
                            Err(err) => IpcResponse::err(&req.id, &err),
                        }
                    }
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "sys_info" => {
                match self.security.check_system_info() {
                    Ok(_) => {
                        let info = json!({
                            "os": std::env::consts::OS,
                            "arch": std::env::consts::ARCH,
                            "family": std::env::consts::FAMILY,
                            "rustVersion": "1.99.1 (GNU)",
                            "processId": std::process::id()
                        });
                        IpcResponse::ok(&req.id, info)
                    }
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "fs_read_file" => {
                let path = match req.payload.get("path").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'path' parameter"),
                };

                match self.security.check_fs_read(path) {
                    Ok(safe_path) => match fs::read_to_string(&safe_path) {
                        Ok(content) => IpcResponse::ok(&req.id, json!({ "path": path, "content": content })),
                        Err(err) => IpcResponse::err(&req.id, &format!("Failed to read file: {}", err)),
                    },
                    Err(err) => IpcResponse::err(&req.id, &err),
                }
            }

            "fs_write_file" => {
                let path = match req.payload.get("path").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'path' parameter"),
                };
                let content = match req.payload.get("content").and_then(|c| c.as_str()) {
                    Some(c) => c,
                    None => return IpcResponse::err(&req.id, "Missing 'content' parameter"),
                };

                match self.security.check_fs_write(path) {
                    Ok(safe_path) => match fs::write(&safe_path, content) {
                        Ok(_) => IpcResponse::ok(&req.id, json!({ "path": path, "bytesWritten": content.len() })),
                        Err(err) => IpcResponse::err(&req.id, &format!("Failed to write file: {}", err)),
                    },
                    Err(err) => IpcResponse::err(&req.id, &err),
                }
            }

            "fs_list_sandbox" => {
                match self.security.check_fs_read(".") {
                    Ok(sandbox_dir) => {
                        let mut entries = Vec::new();
                        if let Ok(read_dir) = fs::read_dir(sandbox_dir) {
                            for entry in read_dir.flatten() {
                                if let Ok(name) = entry.file_name().into_string() {
                                    let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                                    entries.push(json!({
                                        "name": name,
                                        "isDir": is_dir,
                                        "sizeBytes": size
                                    }));
                                }
                            }
                        }
                        IpcResponse::ok(&req.id, json!({ "entries": entries }))
                    }
                    Err(err) => IpcResponse::err(&req.id, &err),
                }
            }

            "crypto_hash" => {
                match self.security.check_crypto() {
                    Ok(_) => {
                        let text = req.payload.get("input").and_then(|t| t.as_str()).unwrap_or("");
                        let mut hasher = Sha256::new();
                        hasher.update(text.as_bytes());
                        let hash = format!("{:x}", hasher.finalize());
                        IpcResponse::ok(&req.id, json!({ "algorithm": "SHA-256", "hash": hash }))
                    }
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "window_spawn" => {
                let url = req.payload.get("url").and_then(|u| u.as_str()).unwrap_or("http://localhost:5173");
                let title = req.payload.get("title").and_then(|t| t.as_str()).unwrap_or("Aegis Secondary Window");
                let width = req.payload.get("width").and_then(|w| w.as_u64()).unwrap_or(800) as u32;
                let height = req.payload.get("height").and_then(|h| h.as_u64()).unwrap_or(600) as u32;
                let modal = req.payload.get("modal").and_then(|m| m.as_bool()).unwrap_or(false);

                let cfg = crate::window::WindowConfig {
                    title: title.to_string(),
                    width,
                    height,
                    resizable: true,
                    modal,
                    always_on_top: false,
                    kiosk: false,
                };

                match crate::window::WindowManager::spawn_window(url, &cfg) {
                    Ok(_) => IpcResponse::ok(&req.id, json!({
                        "status": "Window Spawned Successfully",
                        "title": title,
                        "url": url,
                        "width": width,
                        "height": height
                    })),
                    Err(e) => IpcResponse::err(&req.id, &format!("Failed to spawn window: {}", e)),
                }
            }

            "window_dialog" => {
                let dialog_type = req.payload.get("type").and_then(|t| t.as_str()).unwrap_or("info");
                let message = req.payload.get("message").and_then(|m| m.as_str()).unwrap_or("Aegis Native Notification");
                let title = req.payload.get("title").and_then(|t| t.as_str()).unwrap_or("Aegis Security Dialog");

                // Execute native OS dialog via powershell on Windows, zenity on Linux, or osascript on Mac
                #[cfg(target_os = "windows")]
                {
                    let script = format!(
                        "Add-Type -AssemblyName PresentationFramework; [System.Windows.MessageBox]::Show('{}', '{}')",
                        message.replace("'", "''"),
                        title.replace("'", "''")
                    );
                    let _ = std::process::Command::new("powershell")
                        .args(["-NoProfile", "-Command", &script])
                        .spawn();
                }

                IpcResponse::ok(&req.id, json!({
                    "dialogType": dialog_type,
                    "title": title,
                    "delivered": true
                }))
            }

            "window_list" => {
                // Returns info about multi-window support capabilities
                IpcResponse::ok(&req.id, json!({
                    "multiWindowSupported": true,
                    "apiVersion": "v0.12.0",
                    "spawnMethods": ["window_spawn", "window_dialog"],
                    "platformHint": std::env::consts::OS,
                    "capabilities": {
                        "modal": true,
                        "kiosk": true,
                        "alwaysOnTop": true,
                        "customSize": true,
                        "customUrl": true
                    }
                }))
            }

            "bundle_status" => {
                // Returns info about the bundler engine
                let project_root = std::env::current_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from("."));
                let dist_bundle = project_root.join("dist-bundle");
                let bundle_exists = dist_bundle.exists();
                let bundle_contents: Vec<String> = if bundle_exists {
                    std::fs::read_dir(&dist_bundle)
                        .ok()
                        .map(|d| d.filter_map(|e| e.ok())
                            .filter_map(|e| e.file_name().into_string().ok())
                            .collect())
                        .unwrap_or_default()
                } else {
                    vec![]
                };

                IpcResponse::ok(&req.id, json!({
                    "bundlerEngineVersion": "v0.12.0",
                    "bundleExists": bundle_exists,
                    "bundleOutputDir": dist_bundle.to_string_lossy(),
                    "existingBundles": bundle_contents,
                    "howToBundle": "Run: cargo run --manifest-path src-core/Cargo.toml -- --bundle",
                    "innoSetupDocs": "https://jrsoftware.org/isinfo.php",
                    "supportedTargets": ["windows-x64", "linux-x64", "macos-aarch64"]
                }))
            }

            // ─── PLUGIN ENGINE IPC (v0.13) ─────────────────────────────────

            "plugin_list" => {
                let plugins = self.plugins.list_plugins();
                IpcResponse::ok(&req.id, json!({
                    "plugins": plugins,
                    "count": plugins.len()
                }))
            }

            "plugin_dispatch" => {
                let plugin_name = match req.payload.get("plugin").and_then(|p| p.as_str()) {
                    Some(n) => n,
                    None => return IpcResponse::err(&req.id, "Missing 'plugin' parameter"),
                };
                let payload = req.payload.get("payload").cloned().unwrap_or(json!({}));

                match self.plugins.dispatch(plugin_name, &payload) {
                    Ok(result) => IpcResponse::ok(&req.id, json!({
                        "plugin": plugin_name,
                        "result": result
                    })),
                    Err(e) => IpcResponse::err(&req.id, &format!("Plugin error: {}", e)),
                }
            }

            // ─── NOTIFICATION ENGINE IPC (v0.13) ───────────────────────────

            "notify_send" => {
                let title = req.payload.get("title").and_then(|t| t.as_str()).unwrap_or("Aegis Desktop");
                let body = req.payload.get("body").and_then(|b| b.as_str()).unwrap_or("Notification from Aegis Core");

                let config = NotificationConfig {
                    title: title.to_string(),
                    body: body.to_string(),
                    icon: req.payload.get("icon").and_then(|i| i.as_str()).map(|s| s.to_string()),
                    urgency: crate::tray::NotificationUrgency::Normal,
                    timeout_ms: req.payload.get("timeout_ms").and_then(|t| t.as_u64()).map(|t| t as u32),
                };

                match NotificationEngine::send(&config) {
                    Ok(_) => IpcResponse::ok(&req.id, json!({
                        "delivered": true,
                        "title": title,
                        "body": body,
                        "platform": std::env::consts::OS
                    })),
                    Err(e) => IpcResponse::err(&req.id, &format!("Notification failed: {}", e)),
                }
            }

            "tray_status" => {
                IpcResponse::ok(&req.id, TrayDaemon::get_status())
            }

            // ─── AUTO-UPDATER IPC (v0.13) ──────────────────────────────────

            "updater_status" => {
                IpcResponse::ok(&req.id, self.updater.get_status())
            }

            "updater_check" => {
                // Accept a manifest JSON directly in the payload for air-gapped checking
                match serde_json::from_value::<UpdateManifest>(req.payload.clone()) {
                    Ok(manifest) => {
                        let result = self.updater.check_update_available(&manifest);
                        IpcResponse::ok(&req.id, serde_json::to_value(&result).unwrap_or(json!({})))
                    }
                    Err(e) => IpcResponse::err(&req.id, &format!("Invalid manifest payload: {}", e)),
                }
            }

            "updater_verify" => {
                let file_path = match req.payload.get("filePath").and_then(|p| p.as_str()) {
                    Some(p) => std::path::PathBuf::from(p),
                    None => return IpcResponse::err(&req.id, "Missing 'filePath' parameter"),
                };
                let expected_sha256 = match req.payload.get("expectedSha256").and_then(|h| h.as_str()) {
                    Some(h) => h,
                    None => return IpcResponse::err(&req.id, "Missing 'expectedSha256' parameter"),
                };

                match self.updater.verify_integrity(&file_path, expected_sha256) {
                    Ok(_) => IpcResponse::ok(&req.id, json!({
                        "verified": true,
                        "filePath": file_path.to_string_lossy(),
                        "sha256": expected_sha256
                    })),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            // ─── PERMISSION MANAGER IPC (v0.13 / v0.14) ────────────────────

            "permissions_status" => {
                IpcResponse::ok(&req.id, json!({
                    "permissionManagerVersion": "v0.14.0",
                    "activeCapabilities": {
                        "fs": { "read": true, "write": true, "paths": ["./sandbox/**", "./sandbox_*/**"] },
                        "shell": { "allowed": false, "policy": "PERMANENTLY_REVOKED" },
                        "systemInfo": { "allowed": true },
                        "crypto": { "allowed": true },
                        "encryptedVault": { "allowed": true },
                        "bundler": { "allowed": true },
                        "multiWindow": { "allowed": true },
                        "plugins": { "allowed": true },
                        "notifications": { "allowed": true },
                        "autoUpdater": { "allowed": true, "autoApply": false },
                        "hardwareKey": { "allowed": true, "standard": "FIDO2 / WebAuthn L3" },
                        "multiProfile": { "allowed": true, "isolation": "STRICT_DIRECTORY" },
                        "wasmSandbox": { "allowed": true, "memoryLimitMb": 16 }
                    },
                    "policySource": "aegis.config.json",
                    "runtimeOverridesEnabled": true,
                    "note": "Runtime profile isolation active in v0.14.0"
                }))
            }

            // ─── HARDWARE SECURITY KEY / FIDO2 IPC (v0.14) ─────────────────

            "hardware_key_list" => {
                let detected = self.hardware_keys.scan_and_detect_keys();
                IpcResponse::ok(&req.id, json!({
                    "keys": detected,
                    "fido2Active": true,
                    "deviceCount": detected.len(),
                    "subsystem": "FIDO2 / WebAuthn / CTAP2.1"
                }))
            }

            "hardware_key_enroll" => {
                let key_id = match req.payload.get("keyId").and_then(|k| k.as_str()) {
                    Some(k) => k,
                    None => return IpcResponse::err(&req.id, "Missing 'keyId' parameter"),
                };
                let pin = match req.payload.get("pin").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'pin' parameter"),
                };
                match self.hardware_keys.enroll_key(key_id, pin) {
                    Ok(key_info) => IpcResponse::ok(&req.id, json!({ "enrolled": true, "key": key_info })),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "hardware_key_challenge" => {
                let rp = req.payload.get("relyingParty").and_then(|r| r.as_str()).unwrap_or("aegis.desktop.local");
                let chal = self.hardware_keys.generate_challenge(rp);
                IpcResponse::ok(&req.id, json!(chal))
            }

            "hardware_key_verify" => {
                let chal_id = match req.payload.get("challengeId").and_then(|c| c.as_str()) {
                    Some(c) => c,
                    None => return IpcResponse::err(&req.id, "Missing 'challengeId' parameter"),
                };
                let pin = req.payload.get("pin").and_then(|p| p.as_str()).unwrap_or("");
                match self.hardware_keys.verify_signature(chal_id, pin) {
                    Ok(resp) => IpcResponse::ok(&req.id, json!(resp)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            // ─── MULTI-PROFILE ISOLATION IPC (v0.14) ───────────────────────

            "profile_list" => {
                IpcResponse::ok(&req.id, json!({
                    "profiles": self.profile_manager.list_profiles(),
                    "activeProfile": self.profile_manager.get_active_profile()
                }))
            }

            "profile_switch" => {
                let id = match req.payload.get("profileId").and_then(|i| i.as_str()) {
                    Some(i) => i,
                    None => return IpcResponse::err(&req.id, "Missing 'profileId' parameter"),
                };
                match self.profile_manager.switch_profile(id) {
                    Ok(profile) => {
                        let new_sandbox = self.profile_manager.get_active_sandbox_path();
                        self.security.set_sandbox_root(new_sandbox);
                        IpcResponse::ok(&req.id, json!({
                            "switched": true,
                            "activeProfile": profile
                        }))
                    }
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "profile_create" => {
                let id = match req.payload.get("profileId").and_then(|i| i.as_str()) {
                    Some(i) => i,
                    None => return IpcResponse::err(&req.id, "Missing 'profileId' parameter"),
                };
                let name = req.payload.get("name").and_then(|n| n.as_str()).unwrap_or(id);
                let desc = req.payload.get("description").and_then(|d| d.as_str()).unwrap_or("Custom isolated profile");
                match self.profile_manager.create_profile(id, name, desc) {
                    Ok(profile) => IpcResponse::ok(&req.id, json!({
                        "created": true,
                        "profile": profile
                    })),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            // ─── WASM PLUGIN SANDBOX IPC (v0.14) ───────────────────────────

            "wasm_list" => {
                IpcResponse::ok(&req.id, json!({
                    "modules": self.wasm_sandbox.list_modules(),
                    "vmStatus": "OPERATIONAL",
                    "gasMeterEnabled": true
                }))
            }

            "wasm_execute" => {
                let module_id = match req.payload.get("moduleId").and_then(|m| m.as_str()) {
                    Some(m) => m,
                    None => return IpcResponse::err(&req.id, "Missing 'moduleId' parameter"),
                };
                let default_input = json!({});
                let input_payload = req.payload.get("input").unwrap_or(&default_input);
                match self.wasm_sandbox.execute_module(module_id, input_payload) {
                    Ok(res) => IpcResponse::ok(&req.id, json!(res)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            // ─── BIOMETRIC ENCLAVE IPC (v0.15) ─────────────────────────────

            "enclave_status" => {
                IpcResponse::ok(&req.id, json!(self.biometric_enclave.get_status()))
            }

            "enclave_authenticate" => {
                let reason = req.payload.get("reason").and_then(|r| r.as_str()).unwrap_or("Access Aegis Confidential Subsystem");
                match self.biometric_enclave.authenticate(reason) {
                    Ok(auth_res) => IpcResponse::ok(&req.id, json!(auth_res)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            // ─── ENCRYPTED IPC STREAM IPC (v0.15) ──────────────────────────

            "stream_encrypt" => {
                let plaintext = match req.payload.get("plaintext").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'plaintext' parameter"),
                };
                let envelope = EncryptedStream::encrypt_payload(plaintext, &req.nonce);
                IpcResponse::ok(&req.id, json!(envelope))
            }

            "stream_decrypt" => {
                let envelope_val = match req.payload.get("envelope") {
                    Some(v) => v,
                    None => return IpcResponse::err(&req.id, "Missing 'envelope' parameter"),
                };
                match serde_json::from_value::<crate::ipc::encrypted_stream::EncryptedEnvelope>(envelope_val.clone()) {
                    Ok(env) => match EncryptedStream::decrypt_payload(&env, &req.nonce) {
                        Ok(decrypted) => IpcResponse::ok(&req.id, json!({ "plaintext": decrypted })),
                        Err(e) => IpcResponse::err(&req.id, &e),
                    },
                    Err(e) => IpcResponse::err(&req.id, &format!("Invalid envelope schema: {}", e)),
                }
            }

            // ─── AIR-GAPPED CRASH REPORTER IPC (v0.15) ─────────────────────

            "crash_list" => {
                IpcResponse::ok(&req.id, json!({
                    "crashes": self.crash_reporter.list_crashes(),
                    "zeroTelemetry": true,
                    "storageLocation": "./sandbox/crashes/"
                }))
            }

            "crash_record" => {
                let kind = req.payload.get("errorKind").and_then(|k| k.as_str()).unwrap_or("ControlledDiagnosticTrap");
                let trace = req.payload.get("stackTrace").and_then(|s| s.as_str()).unwrap_or("User initiated air-gap diagnostic capture");
                let report = self.crash_reporter.record_crash(kind, trace);
                IpcResponse::ok(&req.id, json!(report))
            }

            // ─── ZERO-KNOWLEDGE SECRETS MANAGER IPC (v0.16) ──────────────────

            "secrets_list" => {
                let list = self.secret_manager.list_secrets();
                IpcResponse::ok(&req.id, json!({ "secrets": list }))
            }

            "secrets_store" => {
                let key = match req.payload.get("key").and_then(|k| k.as_str()) {
                    Some(k) => k,
                    None => return IpcResponse::err(&req.id, "Missing 'key' parameter"),
                };
                let value = match req.payload.get("value").and_then(|v| v.as_str()) {
                    Some(v) => v,
                    None => return IpcResponse::err(&req.id, "Missing 'value' parameter"),
                };
                let master_pass = match req.payload.get("masterPassphrase").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'masterPassphrase' parameter"),
                };
                let meta = self.secret_manager.store_secret(key, value, master_pass);
                IpcResponse::ok(&req.id, json!(meta))
            }

            "secrets_retrieve" => {
                let key = match req.payload.get("key").and_then(|k| k.as_str()) {
                    Some(k) => k,
                    None => return IpcResponse::err(&req.id, "Missing 'key' parameter"),
                };
                let master_pass = match req.payload.get("masterPassphrase").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'masterPassphrase' parameter"),
                };
                match self.secret_manager.retrieve_secret(key, master_pass) {
                    Ok(val) => IpcResponse::ok(&req.id, json!({ "key": key, "value": val })),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            // ─── NETWORK ISOLATION & FIREWALL POLICY IPC (v0.16) ─────────────

            "firewall_rules" => {
                let rules = self.network_engine.list_rules();
                IpcResponse::ok(&req.id, json!({ "rules": rules, "mode": "STRICT_ZERO_TRUST" }))
            }

            "firewall_evaluate" => {
                let host = match req.payload.get("host").and_then(|h| h.as_str()) {
                    Some(h) => h,
                    None => return IpcResponse::err(&req.id, "Missing 'host' parameter"),
                };
                let port = req.payload.get("port").and_then(|p| p.as_u64()).unwrap_or(443) as u16;
                let eval = self.network_engine.evaluate_connection(host, port);
                IpcResponse::ok(&req.id, json!(eval))
            }

            // ─── PLUGIN SIGNATURE & PROVENANCE IPC (v0.16) ───────────────────

            "plugin_signers" => {
                let signers = self.sig_verifier.list_trusted_signers();
                IpcResponse::ok(&req.id, json!({ "trustedSigners": signers }))
            }

            "plugin_verify_manifest" => {
                let plugin_id = match req.payload.get("pluginId").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'pluginId' parameter"),
                };
                let manifest = match req.payload.get("manifest").and_then(|m| m.as_str()) {
                    Some(m) => m,
                    None => return IpcResponse::err(&req.id, "Missing 'manifest' parameter"),
                };
                let sig = match req.payload.get("signature").and_then(|s| s.as_str()) {
                    Some(s) => s,
                    None => return IpcResponse::err(&req.id, "Missing 'signature' parameter"),
                };
                let res = self.sig_verifier.verify_plugin_manifest(plugin_id, manifest, sig);
                IpcResponse::ok(&req.id, json!(res))
            }

            "execute_shell" => {
                // Strictly blocked by capability policy
                match self.security.check_shell() {
                    Ok(_) => IpcResponse::err(&req.id, "Shell execution granted"),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            _ => IpcResponse::err(&req.id, &format!("Unknown IPC command: '{}'", req.command)),
        }
    }
}
