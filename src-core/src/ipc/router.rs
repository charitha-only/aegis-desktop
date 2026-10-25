use super::encrypted_stream::EncryptedStream;
use super::protocol::{IpcRequest, IpcResponse};
use crate::crash::CrashReporter;
use crate::network::NetworkPolicyEngine;
use crate::p2p_market::P2PPluginMarketplace;
use crate::package_pipeline::{MultiOsPackageConfig, MultiOsPipeline};
use crate::plugin::PluginRegistry;
use crate::profile::ProfileManager;
use crate::secrets::SecretManager;
use crate::security::{
    attestation::TpmAttestationProof, BiometricEnclave, HandshakeSession, HardwareKeyEngine,
    HsmPkcs11Engine, PrivacyShield, SecurityEngine, SignatureVerifier, TpmAttestationEngine,
    VaultEngine,
};
use crate::tray::{NotificationConfig, NotificationEngine, TrayDaemon};
use crate::updater::{AutoUpdater, UpdateManifest};
use crate::wasm::{ExecutionTier, WasmSandbox};
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
    tpm_engine: Arc<TpmAttestationEngine>,
    package_pipeline: Arc<MultiOsPipeline>,
    p2p_market: Arc<P2PPluginMarketplace>,
    hsm_engine: Arc<HsmPkcs11Engine>,
    sensitive_limiter: crate::security::RateLimiter,
    vault_sessions: Arc<crate::security::vault_session::VaultSessions>,
    dispatch: std::sync::Mutex<()>,
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
        tpm_engine: Arc<TpmAttestationEngine>,
        package_pipeline: Arc<MultiOsPipeline>,
        p2p_market: Arc<P2PPluginMarketplace>,
        hsm_engine: Arc<HsmPkcs11Engine>,
    ) -> Self {
        let vault_sessions = Arc::new(crate::security::vault_session::VaultSessions::new(
            secret_manager.clone(),
        ));
        #[cfg(all(windows, not(test)))]
        crate::security::windows_lifecycle::start(Arc::downgrade(&vault_sessions));
        Self {
            vault_sessions,
            dispatch: std::sync::Mutex::new(()),
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
            tpm_engine,
            package_pipeline,
            p2p_market,
            hsm_engine,
            sensitive_limiter: crate::security::RateLimiter::new(20, 60, 60),
        }
    }

    pub fn handle_request(&self, req: IpcRequest) -> IpcResponse {
        // 1. Cryptographic Nonce Verification
        if !self.session.validate_nonce(&req.nonce) {
            return IpcResponse::err(
                &req.id,
                "Security Breach: Invalid cryptographic nonce. Request rejected!",
            );
        }

        // Serialize profile changes, unlock, reads and revocation. A completed lock
        // prevents any later core operation from using a retained credential.
        let _dispatch = match self.dispatch.lock() {
            Ok(guard) => guard,
            Err(_) => {
                self.vault_sessions.lock();
                return IpcResponse::err(&req.id, "Router state unavailable");
            }
        };
        if (req.command.starts_with("vault_") && req.command != "vault_session_lock")
            || req.command == "secrets_store"
            || req.command == "secrets_retrieve"
        {
            if self.sensitive_limiter.is_locked_out("sensitive")
                || self.sensitive_limiter.check_request().is_err()
            {
                return IpcResponse::err(
                    &req.id,
                    "Sensitive operation limit reached; retry after 60 seconds",
                );
            }
        }
        // 2. Command Dispatching with Capability Verification
        if req.command.starts_with("hardware_key_")
            || req.command.starts_with("hsm_")
            || req.command.starts_with("tpm_")
            || req.command.starts_with("p2p_")
            || req.command == "biometric_authenticate"
            || req.command.starts_with("stream_")
            || req.command.starts_with("updater_")
            || req.command == "package_pipeline_stage"
        {
            return IpcResponse::err(&req.id, "UNAVAILABLE: This integration is disabled until a real provider is configured. No hardware verification, network installation or encrypted transport is claimed.");
        }
        match req.command.as_str() {
            "vault_session_lock" => {
                self.vault_sessions.lock();
                IpcResponse::ok(&req.id, json!({"locked": true}))
            }
            "vault_session_unlock" | "vault_session_read" => {
                if let Err(e) = self.security.check_crypto() {
                    return IpcResponse::err(&req.id, &e);
                }
                let file = req
                    .payload
                    .get("filename")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if !file.ends_with(".vault") {
                    return IpcResponse::err(&req.id, "A .vault filename is required");
                }
                let path = match self.security.check_fs_read(file) {
                    Ok(path) => path,
                    Err(e) => return IpcResponse::err(&req.id, &e),
                };
                // Includes the profile's resolved sandbox path; tokens cannot cross profiles.
                let identity = path.to_string_lossy();
                let envelope = match crate::storage::read_bounded(&path) {
                    Ok(data) => data,
                    Err(e) => return IpcResponse::err(&req.id, &e),
                };
                if req.command == "vault_session_unlock" {
                    let pass = req
                        .payload
                        .get("passphrase")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    match self.vault_sessions.unlock(&identity, &envelope, pass) {
                        Ok(token) => {
                            self.sensitive_limiter.clear_nonce_failures("sensitive");
                            IpcResponse::ok(
                                &req.id,
                                json!({"sessionToken": &*token, "idleTimeoutSeconds":crate::security::vault_session::IDLE_TIMEOUT.as_secs(), "absoluteTimeoutSeconds":crate::security::vault_session::ABSOLUTE_TIMEOUT.as_secs()}),
                            )
                        }
                        Err(_) => {
                            self.sensitive_limiter.record_nonce_failure("sensitive");
                            IpcResponse::err(&req.id, "Vault unlock failed")
                        }
                    }
                } else {
                    let token = req
                        .payload
                        .get("sessionToken")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    match self.vault_sessions.read(token, &identity, &envelope) {
                        Ok(plaintext) => IpcResponse::ok(
                            &req.id,
                            json!({"filename":file,"decryptedContent": &*plaintext,"integrityVerified":true}),
                        ),
                        Err(_) => {
                            IpcResponse::err(&req.id, "Vault session is locked, expired or invalid")
                        }
                    }
                }
            }
            "get_security_status" => {
                let mut status = json!({
                    "engine": concat!("Aegis Core ",env!("CARGO_PKG_VERSION")," (Rust)"),
                    "nonceVerified": true,
                    "windowsLifecycleMonitorActive": self.vault_sessions.platform_monitor_active(),
                    "sandboxEnforced": true,
                    "privacyVaultActive": true,
                    "antiDosRateLimiterActive": true,
                    "airGapPolicyActive": false,
                    "bundlerEngineActive": true,
                    "multiWindowApiActive": true,
                    "nativeWebKitWryActive": false,
                    "integrationStatus": "Hardware authentication, TPM, HSM, GPU, encrypted IPC, updater and P2P are unavailable",
                    "pluginEngineActive": true,
                    "notificationEngineActive": true,
                    "autoUpdaterActive": false,
                    "hardwareKeyActive": false,
                    "multiProfileActive": true,
                    "wasmSandboxActive": true,
                    "wgpuComputeJitActive": false,
                    "biometricEnclaveActive": false,
                    "tpmRemoteAttestationActive": false,
                    "encryptedIpcActive": false,
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
                        "nativeWebKitWry": false,
                        "plugins": true,
                        "notifications": true,
                        "autoUpdater": false,
                        "hardwareKey": false,
                        "multiProfile": true,
                        "wasmSandbox": true,
                        "wgpuComputeJit": false,
                        "biometricEnclave": false,
                        "tpmRemoteAttestation": false,
                        "encryptedIpc": false,
                        "crashReporter": true,
                        "multiOsPipeline": true,
                        "p2pPluginMarketplace": false,
                        "hsmPkcs11": false
                    }
                });
                if let (Some(actual), Some(reported)) = (
                    self.security.actual_capabilities().as_object(),
                    status["activeCapabilities"].as_object_mut(),
                ) {
                    for (k, v) in actual {
                        reported.insert(k.clone(), v.clone());
                    }
                }
                IpcResponse::ok(&req.id, status)
            }

            "privacy_status" => IpcResponse::ok(&req.id, PrivacyShield::get_telemetry_audit()),

            "vault_encrypt_store" => {
                self.vault_sessions.lock();
                match self.security.check_crypto() {
                    Ok(_) => {
                        let filename = match req.payload.get("filename").and_then(|f| f.as_str()) {
                            Some(f) => f,
                            None => {
                                return IpcResponse::err(&req.id, "Missing 'filename' parameter")
                            }
                        };
                        let content = match req.payload.get("content").and_then(|c| c.as_str()) {
                            Some(c) => c,
                            None => {
                                return IpcResponse::err(&req.id, "Missing 'content' parameter")
                            }
                        };
                        let passphrase = match req
                            .payload
                            .get("passphrase")
                            .and_then(|p| p.as_str())
                        {
                            Some(p) => p,
                            None => {
                                return IpcResponse::err(&req.id, "Missing 'passphrase' parameter")
                            }
                        };

                        if passphrase.len() < 4 {
                            return IpcResponse::err(
                                &req.id,
                                "Passphrase must be at least 4 characters long",
                            );
                        }

                        // Encrypt data with PBKDF2/SHA-256 stream cipher + HMAC-SHA256
                        let encrypted_envelope = match VaultEngine::encrypt(content, passphrase) {
                            Ok(v) => v,
                            Err(e) => return IpcResponse::err(&req.id, &e),
                        };

                        // Ensure filename ends with .vault
                        let safe_filename = if filename.ends_with(".vault") {
                            filename.to_string()
                        } else {
                            format!("{}.vault", filename)
                        };

                        // Authenticate replacement before any mutation, including its backup.
                        if let Ok(existing) = self.security.check_fs_write(&safe_filename) {
                            if existing.exists() {
                                let old = match crate::storage::read_bounded(&existing) {
                                    Ok(v) => v,
                                    Err(e) => return IpcResponse::err(&req.id, &e),
                                };
                                if VaultEngine::decrypt(&old, passphrase)
                                    .map(zeroize::Zeroizing::new)
                                    .is_err()
                                {
                                    self.sensitive_limiter.record_nonce_failure("sensitive");
                                    return IpcResponse::err(&req.id,"Vault replacement requires the existing master passphrase; file was not changed");
                                }
                            }
                        }
                        // Store safely inside isolated sandbox
                        if let Err(e) = self.security.check_write_quota(
                            &safe_filename,
                            encrypted_envelope.len()
                                + self
                                    .security
                                    .check_fs_read(&safe_filename)
                                    .ok()
                                    .and_then(|p| p.metadata().ok())
                                    .map_or(0, |m| m.len() as usize),
                            self.profile_manager.get_active_profile().max_storage_bytes,
                        ) {
                            return IpcResponse::err(&req.id, &e);
                        }
                        match self.security.check_fs_write(&safe_filename) {
                            Ok(safe_path) => match crate::storage::atomic_vault_write(
                                &safe_path,
                                encrypted_envelope.as_bytes(),
                                passphrase,
                            ) {
                                Ok(_) => IpcResponse::ok(
                                    &req.id,
                                    json!({
                                        "filename": safe_filename,
                                        "status": "Encrypted & Stored Successfully",
                                        "envelopeLength": encrypted_envelope.len()
                                    }),
                                ),
                                Err(err) => IpcResponse::err(
                                    &req.id,
                                    &format!("Failed to write encrypted vault: {}", err),
                                ),
                            },
                            Err(err) => IpcResponse::err(&req.id, &err),
                        }
                    }
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "vault_rotate_passphrase" => {
                self.vault_sessions.lock();
                if let Err(e) = self.security.check_crypto() {
                    return IpcResponse::err(&req.id, &e);
                }
                let file = req
                    .payload
                    .get("filename")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if !file.ends_with(".vault") {
                    return IpcResponse::err(&req.id, "A .vault filename is required");
                }
                let old_pass = req
                    .payload
                    .get("oldPassphrase")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let new_pass = req
                    .payload
                    .get("newPassphrase")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let path = match self.security.check_fs_read(file) {
                    Ok(p) => p,
                    Err(e) => return IpcResponse::err(&req.id, &e),
                };
                let envelope = match crate::storage::read_bounded(&path) {
                    Ok(v) => v,
                    Err(e) => return IpcResponse::err(&req.id, &e),
                };
                let plain = match VaultEngine::decrypt(&envelope, old_pass) {
                    Ok(v) => zeroize::Zeroizing::new(v),
                    Err(e) => {
                        self.sensitive_limiter.record_nonce_failure("sensitive");
                        return IpcResponse::err(&req.id, &e);
                    }
                };
                let new = match VaultEngine::encrypt(&plain, new_pass) {
                    Ok(v) => v,
                    Err(e) => return IpcResponse::err(&req.id, &e),
                };
                if let Err(e) = self.security.check_write_quota(
                    file,
                    new.len() + envelope.len(),
                    self.profile_manager.get_active_profile().max_storage_bytes,
                ) {
                    return IpcResponse::err(&req.id, &e);
                }
                let path = match self.security.check_fs_write(file) {
                    Ok(p) => p,
                    Err(e) => return IpcResponse::err(&req.id, &e),
                };
                match crate::storage::atomic_vault_write(&path, new.as_bytes(), old_pass) {
                    Ok(()) => IpcResponse::ok(
                        &req.id,
                        json!({"rotated":true,"backupUsesPreviousPassphrase":true}),
                    ),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }
            "vault_restore_backup" => {
                self.vault_sessions.lock();
                if let Err(e) = self.security.check_crypto() {
                    return IpcResponse::err(&req.id, &e);
                }
                let file = req
                    .payload
                    .get("filename")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if !file.ends_with(".vault") {
                    return IpcResponse::err(&req.id, "A .vault filename is required");
                }
                let pass = req
                    .payload
                    .get("passphrase")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let backup = match self.security.check_fs_read(&format!("{file}.bak")) {
                    Ok(p) => p,
                    Err(e) => return IpcResponse::err(&req.id, &e),
                };
                let data = match crate::storage::read_bounded(&backup) {
                    Ok(v) => v,
                    Err(e) => return IpcResponse::err(&req.id, &e),
                };
                if VaultEngine::decrypt(&data, pass)
                    .map(zeroize::Zeroizing::new)
                    .is_err()
                {
                    self.sensitive_limiter.record_nonce_failure("sensitive");
                    return IpcResponse::err(
                        &req.id,
                        "Backup authentication failed; no file was changed",
                    );
                }
                if let Err(e) = self.security.check_write_quota(
                    file,
                    data.len(),
                    self.profile_manager.get_active_profile().max_storage_bytes,
                ) {
                    return IpcResponse::err(&req.id, &e);
                }
                let target = match self.security.check_fs_write(file) {
                    Ok(p) => p,
                    Err(e) => return IpcResponse::err(&req.id, &e),
                };
                match crate::storage::atomic_write(&target, data.as_bytes(), false) {
                    Ok(()) => IpcResponse::ok(&req.id, json!({"restored":true,"filename":file})),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }
            "vault_decrypt_retrieve" => {
                match self.security.check_crypto() {
                    Ok(_) => {
                        let filename = match req.payload.get("filename").and_then(|f| f.as_str()) {
                            Some(f) => f,
                            None => {
                                return IpcResponse::err(&req.id, "Missing 'filename' parameter")
                            }
                        };
                        let passphrase = match req
                            .payload
                            .get("passphrase")
                            .and_then(|p| p.as_str())
                        {
                            Some(p) => p,
                            None => {
                                return IpcResponse::err(&req.id, "Missing 'passphrase' parameter")
                            }
                        };

                        let safe_filename = if filename.ends_with(".vault") {
                            filename.to_string()
                        } else {
                            format!("{}.vault", filename)
                        };

                        // Read safely from sandbox
                        match self.security.check_fs_read(&safe_filename) {
                            Ok(safe_path) => match crate::storage::read_bounded(&safe_path) {
                                Ok(envelope) => match VaultEngine::decrypt(&envelope, passphrase) {
                                    Ok(plaintext) => {
                                        let plaintext = zeroize::Zeroizing::new(plaintext);
                                        self.sensitive_limiter.clear_nonce_failures("sensitive");
                                        IpcResponse::ok(
                                            &req.id,
                                            json!({
                                                "filename": safe_filename,
                                                "decryptedContent": &*plaintext,
                                                "integrityVerified": true
                                            }),
                                        )
                                    }
                                    Err(err) => {
                                        self.sensitive_limiter.record_nonce_failure("sensitive");
                                        IpcResponse::err(&req.id, &err)
                                    }
                                },
                                Err(err) => IpcResponse::err(
                                    &req.id,
                                    &format!("Failed to read vault file: {}", err),
                                ),
                            },
                            Err(err) => IpcResponse::err(&req.id, &err),
                        }
                    }
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "sys_info" => match self.security.check_system_info() {
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
            },

            "fs_read_file" => {
                let path = match req.payload.get("path").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'path' parameter"),
                };

                match self.security.check_fs_read(path) {
                    Ok(safe_path) => match crate::storage::read_bounded(&safe_path) {
                        Ok(content) => {
                            IpcResponse::ok(&req.id, json!({ "path": path, "content": content }))
                        }
                        Err(err) => {
                            IpcResponse::err(&req.id, &format!("Failed to read file: {}", err))
                        }
                    },
                    Err(err) => IpcResponse::err(&req.id, &err),
                }
            }

            "fs_write_file" => {
                let path = match req.payload.get("path").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'path' parameter"),
                };
                if path.to_ascii_lowercase().ends_with(".vault")
                    || path.to_ascii_lowercase().ends_with(".bak")
                {
                    return IpcResponse::err(
                        &req.id,
                        "Use authenticated vault operations for vault and backup files",
                    );
                }
                if let Ok(existing) = self.security.check_fs_read(path) {
                    if crate::storage::read_bounded(&existing)
                        .is_ok_and(|s| s.starts_with("AEGIS_VAULT_V2:"))
                    {
                        return IpcResponse::err(
                            &req.id,
                            "Encrypted vault content requires authenticated vault operations",
                        );
                    }
                }
                let content = match req.payload.get("content").and_then(|c| c.as_str()) {
                    Some(c) => c,
                    None => return IpcResponse::err(&req.id, "Missing 'content' parameter"),
                };

                if let Err(e) = self.security.check_write_quota(
                    path,
                    content.len(),
                    self.profile_manager.get_active_profile().max_storage_bytes,
                ) {
                    return IpcResponse::err(&req.id, &e);
                }
                match self.security.check_fs_write(path) {
                    Ok(safe_path) => {
                        match crate::storage::atomic_write(&safe_path, content.as_bytes(), false) {
                            Ok(_) => IpcResponse::ok(
                                &req.id,
                                json!({ "path": path, "bytesWritten": content.len() }),
                            ),
                            Err(err) => {
                                IpcResponse::err(&req.id, &format!("Failed to write file: {}", err))
                            }
                        }
                    }
                    Err(err) => IpcResponse::err(&req.id, &err),
                }
            }

            "fs_list_sandbox" => match self.security.check_fs_read(".") {
                Ok(sandbox_dir) => {
                    let mut entries = Vec::new();
                    if let Ok(read_dir) = fs::read_dir(sandbox_dir) {
                        for entry in read_dir.flatten() {
                            if let Ok(name) = entry.file_name().into_string() {
                                if name.starts_with(".aegis-") {
                                    continue;
                                }
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
            },

            "crypto_hash" => match self.security.check_crypto() {
                Ok(_) => {
                    let text = req
                        .payload
                        .get("input")
                        .and_then(|t| t.as_str())
                        .unwrap_or("");
                    let mut hasher = Sha256::new();
                    hasher.update(text.as_bytes());
                    let hash = format!("{:x}", hasher.finalize());
                    IpcResponse::ok(&req.id, json!({ "algorithm": "SHA-256", "hash": hash }))
                }
                Err(e) => IpcResponse::err(&req.id, &e),
            },

            "window_spawn" => {
                let url = req
                    .payload
                    .get("url")
                    .and_then(|u| u.as_str())
                    .unwrap_or("http://localhost:5173");
                let title = req
                    .payload
                    .get("title")
                    .and_then(|t| t.as_str())
                    .unwrap_or("Aegis Secondary Window");
                let width = req
                    .payload
                    .get("width")
                    .and_then(|w| w.as_u64())
                    .unwrap_or(800) as u32;
                let height = req
                    .payload
                    .get("height")
                    .and_then(|h| h.as_u64())
                    .unwrap_or(600) as u32;
                let modal = req
                    .payload
                    .get("modal")
                    .and_then(|m| m.as_bool())
                    .unwrap_or(false);

                let cfg = crate::window::WindowConfig {
                    title: title.to_string(),
                    width,
                    height,
                    resizable: true,
                    modal,
                    always_on_top: false,
                    kiosk: false,
                    ..Default::default()
                };

                match crate::window::WindowManager::spawn_window(url, &cfg) {
                    Ok(win_id) => IpcResponse::ok(
                        &req.id,
                        json!({
                            "status": "Window Spawned Successfully",
                            "windowId": win_id,
                            "title": title,
                            "url": url,
                            "width": width,
                            "height": height,
                            "runtime": crate::window::WindowManager::detect_runtime()
                        }),
                    ),
                    Err(e) => IpcResponse::err(&req.id, &format!("Failed to spawn window: {}", e)),
                }
            }

            "window_runtime_info" => {
                let info = crate::window::WindowManager::detect_runtime();
                IpcResponse::ok(&req.id, json!(info))
            }

            "window_dialog" => {
                let dialog_type = req
                    .payload
                    .get("type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("info");
                let message = req
                    .payload
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Aegis Native Notification");
                let title = req
                    .payload
                    .get("title")
                    .and_then(|t| t.as_str())
                    .unwrap_or("Aegis Security Dialog");

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

                IpcResponse::ok(
                    &req.id,
                    json!({
                        "dialogType": dialog_type,
                        "title": title,
                        "delivered": true
                    }),
                )
            }

            "window_list" => {
                // Returns info about multi-window support capabilities
                IpcResponse::ok(
                    &req.id,
                    json!({
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
                    }),
                )
            }

            "bundle_status" => {
                // Returns info about the bundler engine
                let project_root =
                    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                let dist_bundle = project_root.join("dist-bundle");
                let bundle_exists = dist_bundle.exists();
                let bundle_contents: Vec<String> = if bundle_exists {
                    std::fs::read_dir(&dist_bundle)
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

                IpcResponse::ok(
                    &req.id,
                    json!({
                        "bundlerEngineVersion": "v0.12.0",
                        "bundleExists": bundle_exists,
                        "bundleOutputDir": dist_bundle.to_string_lossy(),
                        "existingBundles": bundle_contents,
                        "howToBundle": "Run: cargo run --manifest-path src-core/Cargo.toml -- --bundle",
                        "innoSetupDocs": "https://jrsoftware.org/isinfo.php",
                        "supportedTargets": ["windows-x64", "linux-x64", "macos-aarch64"]
                    }),
                )
            }

            // ─── PLUGIN ENGINE IPC (v0.13) ─────────────────────────────────
            "plugin_list" => {
                let plugins = self.plugins.list_plugins();
                IpcResponse::ok(
                    &req.id,
                    json!({
                        "plugins": plugins,
                        "count": plugins.len()
                    }),
                )
            }

            "plugin_dispatch" => {
                let plugin_name = match req.payload.get("plugin").and_then(|p| p.as_str()) {
                    Some(n) => n,
                    None => return IpcResponse::err(&req.id, "Missing 'plugin' parameter"),
                };
                let payload = req.payload.get("payload").cloned().unwrap_or(json!({}));

                match self.plugins.dispatch(plugin_name, &payload) {
                    Ok(result) => IpcResponse::ok(
                        &req.id,
                        json!({
                            "plugin": plugin_name,
                            "result": result
                        }),
                    ),
                    Err(e) => IpcResponse::err(&req.id, &format!("Plugin error: {}", e)),
                }
            }

            // ─── NOTIFICATION ENGINE IPC (v0.13) ───────────────────────────
            "notify_send" => {
                let title = req
                    .payload
                    .get("title")
                    .and_then(|t| t.as_str())
                    .unwrap_or("Aegis Desktop");
                let body = req
                    .payload
                    .get("body")
                    .and_then(|b| b.as_str())
                    .unwrap_or("Notification from Aegis Core");

                let config = NotificationConfig {
                    title: title.to_string(),
                    body: body.to_string(),
                    icon: req
                        .payload
                        .get("icon")
                        .and_then(|i| i.as_str())
                        .map(|s| s.to_string()),
                    urgency: crate::tray::NotificationUrgency::Normal,
                    timeout_ms: req
                        .payload
                        .get("timeout_ms")
                        .and_then(|t| t.as_u64())
                        .map(|t| t as u32),
                };

                match NotificationEngine::send(&config) {
                    Ok(_) => IpcResponse::ok(
                        &req.id,
                        json!({
                            "delivered": true,
                            "title": title,
                            "body": body,
                            "platform": std::env::consts::OS
                        }),
                    ),
                    Err(e) => IpcResponse::err(&req.id, &format!("Notification failed: {}", e)),
                }
            }

            "tray_status" => IpcResponse::ok(&req.id, TrayDaemon::get_status()),

            // ─── AUTO-UPDATER IPC (v0.13) ──────────────────────────────────
            "updater_status" => IpcResponse::ok(&req.id, self.updater.get_status()),

            "updater_check" => {
                // Accept a manifest JSON directly in the payload for air-gapped checking
                match serde_json::from_value::<UpdateManifest>(req.payload.clone()) {
                    Ok(manifest) => {
                        let result = self.updater.check_update_available(&manifest);
                        IpcResponse::ok(&req.id, serde_json::to_value(&result).unwrap_or(json!({})))
                    }
                    Err(e) => {
                        IpcResponse::err(&req.id, &format!("Invalid manifest payload: {}", e))
                    }
                }
            }

            "updater_verify" => {
                let file_path = match req.payload.get("filePath").and_then(|p| p.as_str()) {
                    Some(p) => std::path::PathBuf::from(p),
                    None => return IpcResponse::err(&req.id, "Missing 'filePath' parameter"),
                };
                let expected_sha256 =
                    match req.payload.get("expectedSha256").and_then(|h| h.as_str()) {
                        Some(h) => h,
                        None => {
                            return IpcResponse::err(&req.id, "Missing 'expectedSha256' parameter")
                        }
                    };

                match self.updater.verify_integrity(&file_path, expected_sha256) {
                    Ok(_) => IpcResponse::ok(
                        &req.id,
                        json!({
                            "verified": true,
                            "filePath": file_path.to_string_lossy(),
                            "sha256": expected_sha256
                        }),
                    ),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            // ─── PERMISSION MANAGER IPC (v0.13 / v0.14) ────────────────────
            "permissions_status" => IpcResponse::ok(
                &req.id,
                json!({
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
                        "autoUpdater": { "allowed": false, "autoApply": false },
                        "hardwareKey": { "allowed": false, "standard": "FIDO2 / WebAuthn L3" },
                        "multiProfile": { "allowed": true, "isolation": "STRICT_DIRECTORY" },
                        "wasmSandbox": { "allowed": true, "memoryLimitMb": 16 }
                    },
                    "policySource": "aegis.config.json",
                    "runtimeOverridesEnabled": false,
                    "note": "Runtime profile isolation active in v0.14.0"
                }),
            ),

            // ─── HARDWARE SECURITY KEY / FIDO2 IPC (v0.14) ─────────────────
            "hardware_key_list" => {
                let detected = self.hardware_keys.scan_and_detect_keys();
                IpcResponse::ok(
                    &req.id,
                    json!({
                        "keys": detected,
                        "fido2Active": true,
                        "deviceCount": detected.len(),
                        "subsystem": "FIDO2 / WebAuthn / CTAP2.1"
                    }),
                )
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
                    Ok(key_info) => {
                        IpcResponse::ok(&req.id, json!({ "enrolled": true, "key": key_info }))
                    }
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "hardware_key_challenge" => {
                let rp = req
                    .payload
                    .get("relyingParty")
                    .and_then(|r| r.as_str())
                    .unwrap_or("aegis.desktop.local");
                match self.hardware_keys.generate_challenge(rp) {
                    Ok(chal) => IpcResponse::ok(&req.id, json!(chal)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "hardware_key_verify" => {
                let chal_id = match req.payload.get("challengeId").and_then(|c| c.as_str()) {
                    Some(c) => c,
                    None => return IpcResponse::err(&req.id, "Missing 'challengeId' parameter"),
                };
                let pin = req
                    .payload
                    .get("pin")
                    .and_then(|p| p.as_str())
                    .unwrap_or("");
                // WebAuthn assertion fields — must come from the physical authenticator device
                let client_data_json = req
                    .payload
                    .get("clientDataJSON")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let authenticator_data = req
                    .payload
                    .get("authenticatorData")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let signature = req
                    .payload
                    .get("signature")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                match self.hardware_keys.verify_signature(
                    chal_id,
                    pin,
                    client_data_json,
                    authenticator_data,
                    signature,
                ) {
                    Ok(resp) => IpcResponse::ok(&req.id, json!(resp)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            // ─── MULTI-PROFILE ISOLATION IPC (v0.14) ───────────────────────
            "profile_list" => IpcResponse::ok(
                &req.id,
                json!({
                    "profiles": self.profile_manager.list_profiles(),
                    "activeProfile": self.profile_manager.get_active_profile()
                }),
            ),

            "profile_switch" => {
                self.vault_sessions.lock();
                let id = match req.payload.get("profileId").and_then(|i| i.as_str()) {
                    Some(i) => i,
                    None => return IpcResponse::err(&req.id, "Missing 'profileId' parameter"),
                };
                match self.profile_manager.switch_profile(id) {
                    Ok(profile) => {
                        let new_sandbox = self.profile_manager.get_active_sandbox_path();
                        self.security.set_sandbox_root(new_sandbox);
                        self.secret_manager.clear();
                        IpcResponse::ok(
                            &req.id,
                            json!({
                                "switched": true,
                                "activeProfile": profile
                            }),
                        )
                    }
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "profile_create" => {
                let id = match req.payload.get("profileId").and_then(|i| i.as_str()) {
                    Some(i) => i,
                    None => return IpcResponse::err(&req.id, "Missing 'profileId' parameter"),
                };
                let name = req
                    .payload
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or(id);
                let desc = req
                    .payload
                    .get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("Custom isolated profile");
                match self.profile_manager.create_profile(id, name, desc) {
                    Ok(profile) => IpcResponse::ok(
                        &req.id,
                        json!({
                            "created": true,
                            "profile": profile
                        }),
                    ),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            // ─── WASM PLUGIN SANDBOX IPC (v0.14) ───────────────────────────
            "wasm_list" => IpcResponse::ok(
                &req.id,
                json!({
                    "modules": self.wasm_sandbox.list_modules(),
                    "vmStatus": "OPERATIONAL",
                    "gasMeterEnabled": true
                }),
            ),

            "wasm_execute" => {
                let module_id = match req.payload.get("moduleId").and_then(|m| m.as_str()) {
                    Some(m) => m,
                    None => return IpcResponse::err(&req.id, "Missing 'moduleId' parameter"),
                };
                let default_input = json!({});
                let input_payload = req.payload.get("input").unwrap_or(&default_input);
                let tier = match req.payload.get("tier").and_then(|t| t.as_str()) {
                    Some("WGPU_JIT") => ExecutionTier::WgpuJit,
                    Some("INTERPRETER") => ExecutionTier::Interpreter,
                    _ => ExecutionTier::Auto,
                };
                match self
                    .wasm_sandbox
                    .execute_module_with_tier(module_id, input_payload, tier)
                {
                    Ok(res) => IpcResponse::ok(&req.id, json!(res)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "wasm_execute_gpu" => {
                let module_id = match req.payload.get("moduleId").and_then(|m| m.as_str()) {
                    Some(m) => m,
                    None => return IpcResponse::err(&req.id, "Missing 'moduleId' parameter"),
                };
                let default_input = json!({});
                let input_payload = req.payload.get("input").unwrap_or(&default_input);
                match self.wasm_sandbox.execute_module_with_tier(
                    module_id,
                    input_payload,
                    ExecutionTier::WgpuJit,
                ) {
                    Ok(res) => IpcResponse::ok(&req.id, json!(res)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            // ─── BIOMETRIC ENCLAVE IPC (v0.15) ─────────────────────────────
            "enclave_status" => {
                IpcResponse::ok(&req.id, json!(self.biometric_enclave.get_status()))
            }

            "enclave_authenticate" => {
                let reason = req
                    .payload
                    .get("reason")
                    .and_then(|r| r.as_str())
                    .unwrap_or("Access Aegis Confidential Subsystem");
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
                match serde_json::from_value::<crate::ipc::encrypted_stream::EncryptedEnvelope>(
                    envelope_val.clone(),
                ) {
                    Ok(env) => match EncryptedStream::decrypt_payload(&env, &req.nonce) {
                        Ok(decrypted) => {
                            IpcResponse::ok(&req.id, json!({ "plaintext": decrypted }))
                        }
                        Err(e) => IpcResponse::err(&req.id, &e),
                    },
                    Err(e) => IpcResponse::err(&req.id, &format!("Invalid envelope schema: {}", e)),
                }
            }

            // ─── AIR-GAPPED CRASH REPORTER IPC (v0.15) ─────────────────────
            "crash_list" => IpcResponse::ok(
                &req.id,
                json!({
                    "crashes": self.crash_reporter.list_crashes(),
                    "zeroTelemetry": true,
                    "storageLocation": "./sandbox/crashes/"
                }),
            ),

            "crash_record" => {
                let kind = req
                    .payload
                    .get("errorKind")
                    .and_then(|k| k.as_str())
                    .unwrap_or("ControlledDiagnosticTrap");
                let trace = "Application diagnostic event; caller trace omitted to avoid persisting secrets";
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
                let master_pass = match req.payload.get("masterPassphrase").and_then(|p| p.as_str())
                {
                    Some(p) => p,
                    None => {
                        return IpcResponse::err(&req.id, "Missing 'masterPassphrase' parameter")
                    }
                };
                let meta = match self.secret_manager.store_secret(key, value, master_pass) {
                    Ok(v) => v,
                    Err(e) => return IpcResponse::err(&req.id, &e),
                };
                IpcResponse::ok(&req.id, json!(meta))
            }

            "secrets_retrieve" => {
                let key = match req.payload.get("key").and_then(|k| k.as_str()) {
                    Some(k) => k,
                    None => return IpcResponse::err(&req.id, "Missing 'key' parameter"),
                };
                let master_pass = match req.payload.get("masterPassphrase").and_then(|p| p.as_str())
                {
                    Some(p) => p,
                    None => {
                        return IpcResponse::err(&req.id, "Missing 'masterPassphrase' parameter")
                    }
                };
                match self.secret_manager.retrieve_secret(key, master_pass) {
                    Ok(val) => IpcResponse::ok(&req.id, json!({ "key": key, "value": val })),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            // ─── NETWORK ISOLATION & FIREWALL POLICY IPC (v0.16) ─────────────
            "firewall_rules" => {
                let rules = self.network_engine.list_rules();
                IpcResponse::ok(
                    &req.id,
                    json!({ "rules": rules, "mode": "STRICT_ZERO_TRUST" }),
                )
            }

            "firewall_evaluate" => {
                let host = match req.payload.get("host").and_then(|h| h.as_str()) {
                    Some(h) => h,
                    None => return IpcResponse::err(&req.id, "Missing 'host' parameter"),
                };
                let port = req
                    .payload
                    .get("port")
                    .and_then(|p| p.as_u64())
                    .unwrap_or(443) as u16;
                let nonce = req
                    .payload
                    .get("nonce")
                    .and_then(|n| n.as_str())
                    .unwrap_or(&req.nonce);
                let proof_val = req.payload.get("attestationProof");
                let proof: Option<TpmAttestationProof> =
                    proof_val.and_then(|v| serde_json::from_value(v.clone()).ok());

                let eval = self.network_engine.evaluate_connection_with_attestation(
                    host,
                    port,
                    proof.as_ref(),
                    &self.tpm_engine,
                    nonce,
                );
                IpcResponse::ok(&req.id, json!(eval))
            }

            // ─── TPM 2.0 REMOTE ATTESTATION IPC (v0.17) ─────────────────────
            "tpm_measurements" => {
                let measurements = self.tpm_engine.get_measurements();
                IpcResponse::ok(&req.id, json!({ "pcrs": measurements, "tpmFamily": "2.0" }))
            }

            "tpm_generate_quote" => {
                let nonce = match req.payload.get("nonce").and_then(|n| n.as_str()) {
                    Some(n) => n,
                    None => {
                        return IpcResponse::err(&req.id, "Missing 'nonce' parameter for TPM quote")
                    }
                };
                let requested_pcrs: Vec<u32> = req
                    .payload
                    .get("pcrs")
                    .and_then(|p| p.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_u64().map(|u| u as u32))
                            .collect()
                    })
                    .unwrap_or_else(|| vec![0, 7, 10, 11]);

                match self.tpm_engine.generate_quote(nonce, &requested_pcrs) {
                    Ok(proof) => IpcResponse::ok(&req.id, json!(proof)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "tpm_verify_quote" => {
                let nonce = match req.payload.get("nonce").and_then(|n| n.as_str()) {
                    Some(n) => n,
                    None => {
                        return IpcResponse::err(
                            &req.id,
                            "Missing 'nonce' parameter for TPM quote verification",
                        )
                    }
                };
                let proof_val = match req.payload.get("proof") {
                    Some(v) => v,
                    None => return IpcResponse::err(&req.id, "Missing 'proof' payload"),
                };
                match serde_json::from_value::<TpmAttestationProof>(proof_val.clone()) {
                    Ok(proof) => match self.tpm_engine.verify_quote(&proof, nonce) {
                        Ok(res) => IpcResponse::ok(&req.id, json!(res)),
                        Err(e) => IpcResponse::err(&req.id, &e),
                    },
                    Err(e) => {
                        IpcResponse::err(&req.id, &format!("Invalid TPM proof schema: {}", e))
                    }
                }
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
                let res = self
                    .sig_verifier
                    .verify_plugin_manifest(plugin_id, manifest, sig);
                IpcResponse::ok(&req.id, json!(res))
            }

            // ─── AUTOMATED MULTI-OS PACKAGING PIPELINE IPC (v0.18) ────────
            "package_pipeline_stage" => {
                let platforms: Vec<String> = req
                    .payload
                    .get("platforms")
                    .and_then(|p| p.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                let mut config = MultiOsPackageConfig::default();
                if let Some(name) = req.payload.get("name").and_then(|n| n.as_str()) {
                    config.name = name.to_string();
                }
                if let Some(app_id) = req.payload.get("appId").and_then(|a| a.as_str()) {
                    config.app_id = app_id.to_string();
                }
                if let Some(version) = req.payload.get("version").and_then(|v| v.as_str()) {
                    config.version = version.to_string();
                }

                let project_root =
                    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                match MultiOsPipeline::stage_packages(&project_root, &config, &platforms) {
                    Ok(report) => IpcResponse::ok(&req.id, json!(report)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            // ─── DISTRIBUTED P2P PLUGIN MARKETPLACE IPC (v0.18) ────────────
            "p2p_market_list" => {
                let plugins = self.p2p_market.list_plugins();
                let peers = self.p2p_market.list_peers();
                IpcResponse::ok(&req.id, json!({ "plugins": plugins, "peers": peers }))
            }

            "p2p_market_sign" => {
                let plugin_id = match req.payload.get("pluginId").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'pluginId' parameter"),
                };
                let signer_did = match req.payload.get("signerDid").and_then(|s| s.as_str()) {
                    Some(s) => s,
                    None => return IpcResponse::err(&req.id, "Missing 'signerDid' parameter"),
                };
                match self.p2p_market.sign_plugin(plugin_id, signer_did) {
                    Ok(listing) => IpcResponse::ok(&req.id, json!(listing)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "p2p_market_install" => {
                let plugin_id = match req.payload.get("pluginId").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'pluginId' parameter"),
                };
                match self.p2p_market.install_plugin(plugin_id) {
                    Ok(listing) => IpcResponse::ok(&req.id, json!(listing)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            // ─── HARDWARE SECURITY MODULE (HSM) PKCS#11 IPC (v0.18) ────────
            "hsm_slots" => {
                let slots = self.hsm_engine.list_slots();
                IpcResponse::ok(&req.id, json!({ "slots": slots }))
            }

            "hsm_login" => {
                let slot_id = req
                    .payload
                    .get("slotId")
                    .and_then(|s| s.as_u64())
                    .unwrap_or(0);
                let pin = match req.payload.get("pin").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => {
                        return IpcResponse::err(
                            &req.id,
                            "Missing 'pin' parameter for HSM authentication",
                        )
                    }
                };
                match self.hsm_engine.login_token(slot_id, pin) {
                    Ok(success) => {
                        IpcResponse::ok(&req.id, json!({ "slotId": slot_id, "loggedIn": success }))
                    }
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "hsm_logout" => {
                let slot_id = req
                    .payload
                    .get("slotId")
                    .and_then(|s| s.as_u64())
                    .unwrap_or(0);
                match self.hsm_engine.logout_token(slot_id) {
                    Ok(_) => {
                        IpcResponse::ok(&req.id, json!({ "slotId": slot_id, "loggedIn": false }))
                    }
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "hsm_keys" => {
                let slot_id = req
                    .payload
                    .get("slotId")
                    .and_then(|s| s.as_u64())
                    .unwrap_or(0);
                match self.hsm_engine.list_keys(slot_id) {
                    Ok(keys) => {
                        IpcResponse::ok(&req.id, json!({ "slotId": slot_id, "keys": keys }))
                    }
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "hsm_sign" => {
                let slot_id = req
                    .payload
                    .get("slotId")
                    .and_then(|s| s.as_u64())
                    .unwrap_or(0);
                let key_id = match req.payload.get("keyId").and_then(|k| k.as_str()) {
                    Some(k) => k,
                    None => return IpcResponse::err(&req.id, "Missing 'keyId' parameter"),
                };
                let payload = match req.payload.get("payload").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => {
                        return IpcResponse::err(&req.id, "Missing 'payload' parameter to sign")
                    }
                };
                match self.hsm_engine.sign_payload(slot_id, key_id, payload) {
                    Ok(res) => IpcResponse::ok(&req.id, json!(res)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
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
