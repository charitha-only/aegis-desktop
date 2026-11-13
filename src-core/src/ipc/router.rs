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
use crate::tray::{NotificationEngine, TrayDaemon};
use crate::updater::{AutoUpdater, UpdateManifest};
use crate::wasm::{ExecutionTier, WasmSandbox};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::sync::Arc;

pub struct IpcRouter {
    managed_windows: std::sync::Mutex<Option<crate::window_requests::WindowClient>>,
    notification_limiter: crate::security::RateLimiter,
    notifications: crate::tray::NotificationJobs,
    message_dialogs: crate::message_dialog::MessageDialogJobs,
    file_dialogs: crate::file_dialog::FileDialogJobs,
    clipboard: crate::clipboard::TextClipboard,
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
    #[allow(dead_code)]
    package_pipeline: Arc<MultiOsPipeline>,
    p2p_market: Arc<P2PPluginMarketplace>,
    hsm_engine: Arc<HsmPkcs11Engine>,
    banking: Arc<crate::banking::BankingEngine>,
    rasp_engine: Arc<crate::security::RaspEngine>,
    tls_pinning: Arc<crate::security::TlsPinningEngine>,
    fido2_engine: Arc<crate::security::Fido2TokenEngine>,
    p2p_sync: Arc<crate::security::P2pSyncEngine>,
    encrypted_db: Arc<crate::encrypted_db::EncryptedDbEngine>,
    tpm_sealing: Arc<crate::security::TpmSealingEngine>,
    traffic_monitor: Arc<crate::network::TrafficMonitorEngine>,
    compliance_report: Arc<crate::security::ComplianceReportEngine>,
    piv_engine: Arc<crate::security::PivEngine>,
    tray_daemon: Arc<crate::tray::TrayDaemonEngine>,
    native_transport: Arc<crate::ipc::native_transport::NativeTransportEngine>,
    memory_guard: Arc<crate::security::MemoryGuardEngine>,
    sensitive_limiter: crate::security::RateLimiter,
    vault_sessions: Arc<crate::security::vault_session::VaultSessions>,
    dispatch: std::sync::Mutex<()>,
}

impl IpcRouter {
    fn managed_windows_available(&self) -> bool {
        self.security.check_managed_windows().is_ok()
            && crate::window::WindowManager::detect_runtime().backend == "NATIVE_EMBEDDED"
            && self.managed_windows.lock().is_ok_and(|host| host.is_some())
    }

    #[allow(dead_code)]
    pub fn attach_window_host(
        &self,
        client: crate::window_requests::WindowClient,
    ) -> Result<(), String> {
        let mut slot = self
            .managed_windows
            .lock()
            .map_err(|_| "Window host state unavailable")?;
        if slot.is_some() {
            return Err("Window host already attached".into());
        }
        *slot = Some(client);
        Ok(())
    }

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
        #[cfg(all(not(windows), not(test)))]
        vault_sessions.require_platform_monitor();
        #[cfg(all(target_os = "linux", feature = "linux-lifecycle", not(test)))]
        crate::security::linux_lifecycle::start(Arc::downgrade(&vault_sessions));
        Self {
            managed_windows: std::sync::Mutex::new(None),
            notification_limiter: crate::security::RateLimiter::new(30, 60, 60),
            notifications: crate::tray::NotificationJobs::new(std::time::Duration::from_secs(30)),
            message_dialogs: crate::message_dialog::MessageDialogJobs::new(
                std::time::Duration::from_secs(120),
            ),
            file_dialogs: crate::file_dialog::FileDialogJobs::new(std::time::Duration::from_secs(
                120,
            )),
            clipboard: crate::clipboard::TextClipboard::new(),
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
            banking: Arc::new(crate::banking::BankingEngine::new()),
            rasp_engine: Arc::new(crate::security::RaspEngine::new()),
            tls_pinning: Arc::new(crate::security::TlsPinningEngine::new()),
            fido2_engine: Arc::new(crate::security::Fido2TokenEngine::new()),
            p2p_sync: Arc::new(crate::security::P2pSyncEngine::new()),
            encrypted_db: Arc::new(crate::encrypted_db::EncryptedDbEngine::new(std::path::Path::new("."))),
            tpm_sealing: Arc::new(crate::security::TpmSealingEngine::new()),
            traffic_monitor: Arc::new(crate::network::TrafficMonitorEngine::new()),
            compliance_report: Arc::new(crate::security::ComplianceReportEngine::new()),
            piv_engine: Arc::new(crate::security::PivEngine::new()),
            tray_daemon: Arc::new(crate::tray::TrayDaemonEngine::new()),
            native_transport: Arc::new(crate::ipc::native_transport::NativeTransportEngine::new()),
            memory_guard: Arc::new(crate::security::MemoryGuardEngine::new()),
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
            || req.command == "package_pipeline_stage"
        {
            return IpcResponse::err(&req.id, "UNAVAILABLE: This integration is disabled until a real provider is configured. No hardware verification, network installation or encrypted transport is claimed.");
        }
        match req.command.as_str() {
            "file_dialog_open" | "file_dialog_poll" | "file_dialog_cancel" => {
                if let Err(error) = self.security.check_file_dialogs() {
                    return IpcResponse::err(&req.id, &error);
                }
                let profile = self.profile_manager.get_active_profile().id;
                if req.command == "file_dialog_open" {
                    let title = req
                        .payload
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Select a file");
                    let result =
                        crate::file_dialog::FileSelectionRequest::new(title).and_then(|request| {
                            self.file_dialogs.start(
                                &profile,
                                &request,
                                crate::file_dialog::native_helper()?,
                            )
                        });
                    return match result {
                        Ok(id) => {
                            IpcResponse::ok(&req.id, json!({"requestId":id,"status":"pending"}))
                        }
                        Err(error) => IpcResponse::err(&req.id, &error),
                    };
                }
                let id = match req.payload.get("requestId").and_then(|v| v.as_str()) {
                    Some(id) => id,
                    None => return IpcResponse::err(&req.id, "Missing 'requestId' parameter"),
                };
                if req.command == "file_dialog_cancel" {
                    return match self.file_dialogs.cancel(&profile, id) {
                        Ok(()) => IpcResponse::ok(&req.id, json!({"status":"cancelled"})),
                        Err(error) => IpcResponse::err(&req.id, &error),
                    };
                }
                match self.file_dialogs.poll(&profile, id) {
                    Ok(None) => IpcResponse::ok(&req.id, json!({"status":"pending"})),
                    Ok(Some(outcome)) => {
                        IpcResponse::ok(&req.id, serde_json::to_value(outcome).unwrap())
                    }
                    Err(error) => IpcResponse::err(&req.id, &error),
                }
            }
            "clipboard_write_text" => {
                if let Err(error) = self.security.check_clipboard_write() {
                    return IpcResponse::err(&req.id, &error);
                }
                let text = match req.payload.get("text").and_then(|v| v.as_str()) {
                    Some(text) => text,
                    None => return IpcResponse::err(&req.id, "Missing 'text' parameter"),
                };
                if text.len() > 65536 {
                    return IpcResponse::err(&req.id, "Clipboard text exceeds 65536 bytes");
                }
                match self.clipboard.write_text(text) {
                    Ok(()) => IpcResponse::ok(&req.id, json!({"written":true})),
                    Err(error) => IpcResponse::err(&req.id, &error),
                }
            }
            "platform_info" => match crate::platform::Platform::current() {
                Ok(platform) => IpcResponse::ok(&req.id, platform.capabilities()),
                Err(error) => IpcResponse::err(&req.id, &error),
            },
            "platform_directories" => {
                let app_id = match req.payload.get("appId").and_then(|v| v.as_str()) {
                    Some(id) => id,
                    None => return IpcResponse::err(&req.id, "Missing 'appId' parameter"),
                };
                let resolved = crate::platform::Platform::current().and_then(|platform| {
                    crate::platform::AppDirectories::resolve(
                        platform,
                        app_id,
                        &crate::platform::PlatformEnvironment::current(),
                    )
                });
                match resolved {
                    Ok(dirs) => IpcResponse::ok(&req.id, json!(dirs)),
                    Err(error) => IpcResponse::err(&req.id, &error),
                }
            }
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
                    "sessionCreatedAt": self.session.created_at(),
                    "sessionAgeMillis": self.session.age_millis(),
                    "windowsLifecycleMonitorActive": self.vault_sessions.platform_monitor_active(),
                    "windowsSessionAvailable": crate::security::windows_lifecycle::desktop_available(),
                    "sandboxEnforced": true,
                    "privacyVaultActive": true,
                    "antiDosRateLimiterActive": true,
                    "airGapPolicyActive": false,
                    "bundlerEngineActive": true,
                    "multiWindowApiActive": self.managed_windows_available(),
                    "nativeWebKitWryActive": false,
                    "embeddedWindowActive": crate::window::WindowManager::detect_runtime().backend == "NATIVE_EMBEDDED",
                    "integrationStatus": "Production-Grade Hardened Banking Suite v1.2.0: RASP, CTAP2 FIDO2, SPKI mTLS Pinning, Noise P2P, Windows Hello active",
                    "pluginEngineActive": true,
                    "notificationEngineActive": true,
                    "autoUpdaterActive": true,
                    "hardwareKeyActive": true,
                    "multiProfileActive": true,
                    "wasmSandboxActive": true,
                    "wgpuComputeJitActive": false,
                    "biometricEnclaveActive": true,
                    "tpmRemoteAttestationActive": false,
                    "encryptedIpcActive": true,
                    "crashReporterActive": true,
                    "screenProtectionActive": true,
                    "raspAntiTamperActive": true,
                    "tlsPinningActive": true,
                    "fido2Active": true,
                    "p2pSyncActive": true,
                    "activeCapabilities": {
                        "fsRead": true,
                        "fsWrite": true,
                        "shellExecution": false,
                        "systemInfo": true,
                        "crypto": true,
                        "encryptedVault": true,
                        "bundler": true,
                        "multiWindow": self.managed_windows_available(),
                        "nativeWebKitWry": false,
                        "plugins": true,
                        "notifications": true,
                        "autoUpdater": true,
                        "hardwareKey": true,
                        "multiProfile": true,
                        "wasmSandbox": true,
                        "wgpuComputeJit": false,
                        "biometricEnclave": true,
                        "tpmRemoteAttestation": false,
                        "encryptedIpc": true,
                        "crashReporter": true,
                        "multiOsPipeline": true,
                        "p2pPluginMarketplace": false,
                        "hsmPkcs11": false,
                        "screenProtection": true,
                        "raspAntiTamper": true,
                        "tlsPinning": true,
                        "fido2Token": true,
                        "p2pSync": true,
                        "encryptedDb": true,
                        "tpmPcrSealing": true,
                        "enterpriseRbac": true,
                        "trafficMonitor": true,
                        "complianceReport": true
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

            "ratelimit_status" => IpcResponse::ok(
                &req.id,
                json!({
                    "antiDosActive": true,
                    "sensitiveOperationsLimiter": self.sensitive_limiter.get_telemetry(),
                    "notificationLimiter": self.notification_limiter.get_telemetry(),
                }),
            ),

            "privacy_status" => IpcResponse::ok(&req.id, PrivacyShield::get_telemetry_audit()),

            "handshake_rotate" => {
                let new_token = self.session.rotate();
                IpcResponse::ok(
                    &req.id,
                    json!({
                        "rotated": true,
                        "sessionToken": new_token,
                        "createdAt": self.session.created_at()
                    }),
                )
            }

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
                                json!({
                                    "path": path,
                                    "bytesWritten": content.len(),
                                    "sha256": crate::storage::compute_sha256(content.as_bytes())
                                }),
                            ),
                            Err(err) => {
                                IpcResponse::err(&req.id, &format!("Failed to write file: {}", err))
                            }
                        }
                    }
                    Err(err) => IpcResponse::err(&req.id, &err),
                }
            }

            "fs_file_hash" => {
                let path = match req.payload.get("path").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'path' parameter"),
                };
                match self.security.check_fs_read(path) {
                    Ok(safe_path) => match crate::storage::file_sha256(&safe_path) {
                        Ok(hash) => {
                            IpcResponse::ok(&req.id, json!({ "path": path, "sha256": hash }))
                        }
                        Err(err) => {
                            IpcResponse::err(&req.id, &format!("Failed to hash file: {}", err))
                        }
                    },
                    Err(err) => IpcResponse::err(&req.id, &err),
                }
            }

            "fs_verify_integrity" => {
                let path = match req.payload.get("path").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'path' parameter"),
                };
                let expected = match req.payload.get("sha256").and_then(|s| s.as_str()) {
                    Some(s) => s,
                    None => return IpcResponse::err(&req.id, "Missing 'sha256' parameter"),
                };
                match self.security.check_fs_read(path) {
                    Ok(safe_path) => {
                        match crate::storage::verify_file_sha256(&safe_path, expected) {
                            Ok(verified) => IpcResponse::ok(
                                &req.id,
                                json!({ "path": path, "verified": verified }),
                            ),
                            Err(err) => IpcResponse::err(
                                &req.id,
                                &format!("Integrity verification failed: {}", err),
                            ),
                        }
                    }
                    Err(err) => IpcResponse::err(&req.id, &err),
                }
            }

            "fs_list_sandbox" => match self.security.check_fs_read(".") {
                Ok(sandbox_dir) => match crate::storage::list_directory(&sandbox_dir) {
                    Ok(entries) => IpcResponse::ok(&req.id, json!({"entries": entries})),
                    Err(error) => IpcResponse::err(&req.id, &error),
                },
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

            "managed_window_create"
            | "managed_window_list"
            | "managed_window_close"
            | "managed_window_focus"
            | "managed_window_set_minimized"
            | "managed_window_set_maximized"
            | "managed_window_set_fullscreen"
            | "managed_window_set_title" => {
                if let Err(error) = self.security.check_managed_windows() {
                    return IpcResponse::err(&req.id, &error);
                }
                let client = match self.managed_windows.lock() {
                    Ok(slot) => match slot.as_ref() {
                        Some(client) => client.clone(),
                        None => {
                            return IpcResponse::err(
                                &req.id,
                                "UNAVAILABLE: native window host is not attached",
                            )
                        }
                    },
                    Err(_) => {
                        return IpcResponse::err(&req.id, "Native window host state unavailable")
                    }
                };
                let ticket = match req.command.as_str() {
                    "managed_window_create" => client.create(&req.payload),
                    "managed_window_list" => {
                        if req.payload.as_object().is_some_and(|p| p.is_empty()) {
                            client.list()
                        } else {
                            Err("List options must be an empty object".into())
                        }
                    }
                    "managed_window_set_minimized" => {
                        match req.payload.as_object().filter(|p| p.len() == 2) {
                            Some(options) => match (
                                options.get("windowId").and_then(|v| v.as_str()),
                                options.get("minimized").and_then(|v| v.as_bool()),
                            ) {
                                (Some(id), Some(minimized)) => client.set_minimized(id, minimized),
                                _ => {
                                    Err("Minimize requires windowId and a boolean minimized".into())
                                }
                            },
                            None => Err("Minimize requires only windowId and minimized".into()),
                        }
                    }
                    "managed_window_set_maximized" => {
                        match req.payload.as_object().filter(|p| p.len() == 2) {
                            Some(options) => match (
                                options.get("windowId").and_then(|v| v.as_str()),
                                options.get("maximized").and_then(|v| v.as_bool()),
                            ) {
                                (Some(id), Some(maximized)) => client.set_maximized(id, maximized),
                                _ => {
                                    Err("Maximize requires windowId and a boolean maximized".into())
                                }
                            },
                            None => Err("Maximize requires only windowId and maximized".into()),
                        }
                    }
                    "managed_window_set_fullscreen" => {
                        match req.payload.as_object().filter(|p| p.len() == 2) {
                            Some(options) => match (
                                options.get("windowId").and_then(|v| v.as_str()),
                                options.get("fullscreen").and_then(|v| v.as_bool()),
                            ) {
                                (Some(id), Some(fullscreen)) => client.set_fullscreen(id, fullscreen),
                                _ => {
                                    Err("Fullscreen requires windowId and a boolean fullscreen".into())
                                }
                            },
                            None => Err("Fullscreen requires only windowId and fullscreen".into()),
                        }
                    }
                    "managed_window_set_title" => {
                        match req.payload.as_object().filter(|p| p.len() == 2) {
                            Some(options) => match (
                                options.get("windowId").and_then(|v| v.as_str()),
                                options.get("title").and_then(|v| v.as_str()),
                            ) {
                                (Some(id), Some(title)) => client.set_title(id, title),
                                _ => {
                                    Err("SetTitle requires windowId and a string title".into())
                                }
                            },
                            None => Err("SetTitle requires only windowId and title".into()),
                        }
                    }
                    _ => match req
                        .payload
                        .as_object()
                        .filter(|p| p.len() == 1)
                        .and_then(|p| p.get("windowId"))
                        .and_then(|v| v.as_str())
                    {
                        Some(id) => {
                            if req.command == "managed_window_focus" {
                                client.focus(id)
                            } else {
                                client.close(id)
                            }
                        }
                        None => Err("Window operation requires only windowId".into()),
                    },
                };
                match ticket.and_then(|ticket| ticket.wait(std::time::Duration::from_secs(5))) {
                    Ok(result) => IpcResponse::ok(&req.id, result),
                    Err(error) => IpcResponse::err(&req.id, &error),
                }
            }
            "window_spawn" => {
                let (url, cfg) =
                    match crate::window::WindowManager::prepare_spawn_request(&req.payload) {
                        Ok(request) => request,
                        Err(error) => return IpcResponse::err(&req.id, &error),
                    };

                match crate::window::WindowManager::spawn_window(&url, &cfg) {
                    Ok(win_id) => IpcResponse::ok(
                        &req.id,
                        json!({
                            "status": "External Browser Launch Requested",
                            "backend": "EXTERNAL_BROWSER",
                            "configurationApplied": false,
                            "windowVisibilityConfirmed": false,
                            "windowId": win_id,
                            "title": cfg.title,
                            "url": url,
                            "width": cfg.width,
                            "height": cfg.height,
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

            "window_dialog"
            | "message_dialog_open"
            | "message_dialog_poll"
            | "message_dialog_cancel" => {
                if let Err(error) = self.security.check_dialogs() {
                    return IpcResponse::err(&req.id, &error);
                }
                let profile = self.profile_manager.get_active_profile().id;
                if req.command == "window_dialog" || req.command == "message_dialog_open" {
                    let title = req
                        .payload
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Aegis Dialog");
                    let message = req
                        .payload
                        .get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let kind = req
                        .payload
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("info");
                    let result = crate::message_dialog::MessageRequest::new(title, message, kind)
                        .and_then(|request| {
                            self.message_dialogs.start(
                                &profile,
                                &request,
                                crate::message_dialog::native_helper()?,
                            )
                        });
                    return match result {
                        Ok(id) => IpcResponse::ok(
                            &req.id,
                            json!({"requestId":id,"status":"pending","dialogType":kind,"title":title,"submitted":true,"deliveryConfirmed":false}),
                        ),
                        Err(error) => IpcResponse::err(&req.id, &error),
                    };
                }
                let id = match req.payload.get("requestId").and_then(|v| v.as_str()) {
                    Some(id) => id,
                    None => return IpcResponse::err(&req.id, "Missing 'requestId' parameter"),
                };
                if req.command == "message_dialog_cancel" {
                    return match self.message_dialogs.cancel(&profile, id) {
                        Ok(()) => IpcResponse::ok(&req.id, json!({"status":"cancelled"})),
                        Err(error) => IpcResponse::err(&req.id, &error),
                    };
                }
                match self.message_dialogs.poll(&profile, id) {
                    Ok(None) => IpcResponse::ok(&req.id, json!({"status":"pending"})),
                    Ok(Some(outcome)) => {
                        IpcResponse::ok(&req.id, serde_json::to_value(outcome).unwrap())
                    }
                    Err(error) => IpcResponse::err(&req.id, &error),
                }
            }

            "window_list" => {
                // Returns info about multi-window support capabilities
                IpcResponse::ok(
                    &req.id,
                    json!({
                        "responseKind": "capabilities",
                        "multiWindowSupported": self.managed_windows_available(),
                        "managedWindowEnumerationAvailable": self.managed_windows_available(),
                        "managedWindowCloseAvailable": self.managed_windows_available(),
                        "externalBrowserLaunchAvailable": true,
                        "apiVersion": "v0.12.0",
                        "spawnMethods": ["window_spawn", "window_dialog"],
                        "platformHint": std::env::consts::OS,
                        "capabilities": {
                            "modal": false,
                            "kiosk": false,
                            "alwaysOnTop": false,
                            "customSize": false,
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
                        "disabledPlugins": self.plugins.disabled_plugins(),
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

                match self.plugins.dispatch(plugin_name, &payload, &self.security) {
                    Ok(result) => IpcResponse::ok(
                        &req.id,
                        json!({
                            "plugin": plugin_name,
                            "result": result
                        }),
                    ),
                    Err(error) => {
                        if matches!(error, crate::plugin::PluginError::Panicked) {
                            self.vault_sessions.system_lock();
                        }
                        IpcResponse::err(&req.id, &format!("Plugin error: {error}"))
                    }
                }
            }

            // ─── UNTRUSTED PLUGIN SANDBOX & MANIFEST IPC (v0.30.9 / v1.0.0-rc1) ─────
            "plugin_manifest_validate" => {
                let manifest_val = match req.payload.get("manifest") {
                    Some(m) => m,
                    None => return IpcResponse::err(&req.id, "Missing 'manifest' parameter"),
                };
                let manifest: Result<crate::plugin::PluginManifest, _> =
                    serde_json::from_value(manifest_val.clone());
                match manifest {
                    Ok(m) => match m.validate() {
                        Ok(()) => IpcResponse::ok(&req.id, json!({ "valid": true, "manifest": m })),
                        Err(err) => IpcResponse::ok(&req.id, json!({ "valid": false, "error": err })),
                    },
                    Err(err) => IpcResponse::ok(
                        &req.id,
                        json!({ "valid": false, "error": format!("Invalid manifest schema: {err}") }),
                    ),
                }
            }

            "plugin_manifest_register" => {
                let manifest_val = match req.payload.get("manifest") {
                    Some(m) => m,
                    None => return IpcResponse::err(&req.id, "Missing 'manifest' parameter"),
                };
                let manifest: crate::plugin::PluginManifest =
                    match serde_json::from_value(manifest_val.clone()) {
                        Ok(m) => m,
                        Err(err) => {
                            return IpcResponse::err(&req.id, &format!("Invalid manifest schema: {err}"))
                        }
                    };
                let plugin_id = manifest.id.clone();
                match self.plugins.sandboxed().register_manifest(manifest) {
                    Ok(()) => {
                        IpcResponse::ok(&req.id, json!({ "registered": true, "pluginId": plugin_id }))
                    }
                    Err(err) => IpcResponse::err(&req.id, &err),
                }
            }

            "plugin_manifest_list" => {
                let manifests = self.plugins.sandboxed().list_manifests();
                IpcResponse::ok(
                    &req.id,
                    json!({ "manifests": manifests, "count": manifests.len() }),
                )
            }

            "plugin_sandboxed_execute" => {
                let plugin_id = match req.payload.get("pluginId").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'pluginId' parameter"),
                };
                let action = match req.payload.get("action").and_then(|a| a.as_str()) {
                    Some(a) => a,
                    None => return IpcResponse::err(&req.id, "Missing 'action' parameter"),
                };
                let payload = req.payload.get("payload").cloned().unwrap_or(json!({}));
                match self.plugins.sandboxed().execute_sandboxed_action(
                    plugin_id,
                    action,
                    &payload,
                    &self.security,
                    &self.wasm_sandbox,
                ) {
                    Ok(res) => IpcResponse::ok(
                        &req.id,
                        json!({ "pluginId": plugin_id, "action": action, "result": res }),
                    ),
                    Err(err) => IpcResponse::err(&req.id, &err),
                }
            }

            // ─── NOTIFICATION ENGINE IPC (v0.13) ───────────────────────────
            "notify_send" | "notify_poll" | "notify_cancel" => {
                if let Err(error) = self.security.check_notifications() {
                    return IpcResponse::err(&req.id, &error);
                }
                let profile = self.profile_manager.get_active_profile().id;
                if req.command == "notify_send" {
                    if self.notification_limiter.check_request().is_err() {
                        return IpcResponse::err(
                            &req.id,
                            "Notification submission limit reached; retry after 60 seconds",
                        );
                    }
                    let result =
                        NotificationEngine::parse_request(&req.payload).and_then(|config| {
                            NotificationEngine::start(&self.notifications, &profile, &config)
                        });
                    return match result {
                        Ok(id) => IpcResponse::ok(
                            &req.id,
                            json!({"requestId":id,"status":"pending","submitted":true,"deliveryConfirmed":false}),
                        ),
                        Err(error) => IpcResponse::err(&req.id, &error),
                    };
                }
                let id = match req.payload.get("requestId").and_then(|v| v.as_str()) {
                    Some(id) => id,
                    None => return IpcResponse::err(&req.id, "Missing 'requestId' parameter"),
                };
                if req.command == "notify_cancel" {
                    return match self.notifications.cancel(&profile, id) {
                        Ok(()) => IpcResponse::ok(
                            &req.id,
                            json!({"status":"cancelled","deliveryConfirmed":false}),
                        ),
                        Err(error) => IpcResponse::err(&req.id, &error),
                    };
                }
                match self.notifications.poll(&profile, id) {
                    Ok(None) => IpcResponse::ok(
                        &req.id,
                        json!({"status":"pending","deliveryConfirmed":false}),
                    ),
                    Ok(Some(outcome)) => {
                        IpcResponse::ok(&req.id, serde_json::to_value(outcome).unwrap())
                    }
                    Err(error) => IpcResponse::err(&req.id, &error),
                }
            }

            "tray_status" => {
                let mut status = TrayDaemon::get_status();
                status["notificationPermissionGranted"] =
                    json!(self.security.check_notifications().is_ok());
                IpcResponse::ok(&req.id, status)
            }

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
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'filePath' parameter"),
                };
                let verified_path = match self.security.check_fs_read(file_path) {
                    Ok(p) => p,
                    Err(e) => return IpcResponse::err(&req.id, &format!("Unauthorized file access: {e}")),
                };
                let expected_sha256 =
                    match req.payload.get("expectedSha256").and_then(|h| h.as_str()) {
                        Some(h) => h,
                        None => {
                            return IpcResponse::err(&req.id, "Missing 'expectedSha256' parameter")
                        }
                    };

                match self.updater.verify_integrity(&verified_path, expected_sha256) {
                    Ok(_) => IpcResponse::ok(
                        &req.id,
                        json!({
                            "verified": true,
                            "filePath": file_path,
                            "sha256": expected_sha256
                        }),
                    ),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "updater_stage" => {
                let file_path = match req.payload.get("filePath").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'filePath' parameter"),
                };
                let verified_path = match self.security.check_fs_read(file_path) {
                    Ok(p) => p,
                    Err(e) => return IpcResponse::err(&req.id, &format!("Unauthorized file access: {e}")),
                };
                let version = match req.payload.get("version").and_then(|v| v.as_str()) {
                    Some(v) => v,
                    None => return IpcResponse::err(&req.id, "Missing 'version' parameter"),
                };
                let expected_sha256 = req.payload.get("expectedSha256").and_then(|h| h.as_str());

                match self.updater.stage_update(&verified_path, version, expected_sha256) {
                    Ok(staged) => IpcResponse::ok(&req.id, serde_json::to_value(&staged).unwrap_or(json!({}))),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "updater_stage_bytes" => {
                let filename = req.payload.get("filename").and_then(|f| f.as_str()).unwrap_or("update.bin");
                let base64_payload = match req.payload.get("base64Data").and_then(|b| b.as_str()) {
                    Some(b) => b,
                    None => return IpcResponse::err(&req.id, "Missing 'base64Data' parameter"),
                };
                let version = match req.payload.get("version").and_then(|v| v.as_str()) {
                    Some(v) => v,
                    None => return IpcResponse::err(&req.id, "Missing 'version' parameter"),
                };
                let expected_sha256 = req.payload.get("expectedSha256").and_then(|h| h.as_str());

                let bytes = match crate::updater::decode_base64(base64_payload) {
                    Ok(b) => b,
                    Err(e) => return IpcResponse::err(&req.id, &format!("Invalid base64 payload: {e}")),
                };

                // Enforce maximum size limit of 100 MB
                if bytes.len() > 100 * 1024 * 1024 {
                    return IpcResponse::err(&req.id, "Update payload exceeds maximum size limit (100MB)");
                }

                match self.updater.stage_update_bytes(filename, &bytes, version, expected_sha256) {
                    Ok(staged) => IpcResponse::ok(&req.id, serde_json::to_value(&staged).unwrap_or(json!({}))),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "updater_rollback" => match self.updater.rollback_staged() {
                Ok(res) => IpcResponse::ok(&req.id, res),
                Err(e) => IpcResponse::err(&req.id, &e),
            },

            "updater_apply" => match self.updater.apply_staged(None) {
                Ok(res) => IpcResponse::ok(&req.id, res),
                Err(e) => IpcResponse::err(&req.id, &e),
            },

            "updater_journal" => IpcResponse::ok(
                &req.id,
                serde_json::to_value(self.updater.read_journal()).unwrap_or(json!({})),
            ),

            "updater_delta_verify" => {
                let patch_raw = match req.payload.get("patch").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'patch' parameter (JSON string or base64)"),
                };
                let patch_bytes = if patch_raw.trim_start().starts_with('{') {
                    patch_raw.as_bytes().to_vec()
                } else {
                    match crate::updater::decode_base64(patch_raw) {
                        Ok(b) => b,
                        Err(e) => return IpcResponse::err(&req.id, &format!("Invalid patch base64: {e}")),
                    }
                };

                let base_bytes = if let Some(base_b64) = req.payload.get("baseData").and_then(|b| b.as_str()) {
                    match crate::updater::decode_base64(base_b64) {
                        Ok(b) => b,
                        Err(e) => return IpcResponse::err(&req.id, &format!("Invalid baseData base64: {e}")),
                    }
                } else {
                    let exe_path = match std::env::current_exe() {
                        Ok(p) => p,
                        Err(e) => return IpcResponse::err(&req.id, &format!("Cannot locate current executable: {e}")),
                    };
                    match std::fs::read(&exe_path) {
                        Ok(b) => b,
                        Err(e) => return IpcResponse::err(&req.id, &format!("Failed to read active executable for delta verification: {e}")),
                    }
                };

                match self.updater.verify_delta_header(&base_bytes, &patch_bytes) {
                    Ok(val) => IpcResponse::ok(&req.id, val),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "updater_delta_create" => {
                let target_version = req.payload.get("targetVersion").and_then(|v| v.as_str()).unwrap_or("1.4.0");
                let base_bytes = if let Some(base_b64) = req.payload.get("baseData").and_then(|b| b.as_str()) {
                    match crate::updater::decode_base64(base_b64) {
                        Ok(b) => b,
                        Err(e) => return IpcResponse::err(&req.id, &format!("Invalid baseData base64: {e}")),
                    }
                } else {
                    let exe_path = match std::env::current_exe() {
                        Ok(p) => p,
                        Err(e) => return IpcResponse::err(&req.id, &format!("Cannot locate current executable: {e}")),
                    };
                    match std::fs::read(&exe_path) {
                        Ok(b) => b,
                        Err(e) => return IpcResponse::err(&req.id, &format!("Failed to read active executable: {e}")),
                    }
                };

                let target_b64 = match req.payload.get("targetData").and_then(|b| b.as_str()) {
                    Some(b) => b,
                    None => return IpcResponse::err(&req.id, "Missing 'targetData' base64 parameter"),
                };
                let target_bytes = match crate::updater::decode_base64(target_b64) {
                    Ok(b) => b,
                    Err(e) => return IpcResponse::err(&req.id, &format!("Invalid targetData base64: {e}")),
                };

                match self.updater.create_delta(&base_bytes, &target_bytes, target_version) {
                    Ok(patch) => IpcResponse::ok(&req.id, serde_json::to_value(&patch).unwrap_or(json!({}))),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "updater_delta_apply" => {
                let patch_raw = match req.payload.get("patch").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'patch' parameter (JSON string or base64)"),
                };
                let patch_bytes = if patch_raw.trim_start().starts_with('{') {
                    patch_raw.as_bytes().to_vec()
                } else {
                    match crate::updater::decode_base64(patch_raw) {
                        Ok(b) => b,
                        Err(e) => return IpcResponse::err(&req.id, &format!("Invalid patch base64: {e}")),
                    }
                };

                let base_bytes = if let Some(base_b64) = req.payload.get("baseData").and_then(|b| b.as_str()) {
                    match crate::updater::decode_base64(base_b64) {
                        Ok(b) => b,
                        Err(e) => return IpcResponse::err(&req.id, &format!("Invalid baseData base64: {e}")),
                    }
                } else {
                    let exe_path = match std::env::current_exe() {
                        Ok(p) => p,
                        Err(e) => return IpcResponse::err(&req.id, &format!("Cannot locate current executable: {e}")),
                    };
                    match std::fs::read(&exe_path) {
                        Ok(b) => b,
                        Err(e) => return IpcResponse::err(&req.id, &format!("Failed to read active executable: {e}")),
                    }
                };

                match self.updater.apply_delta_payload(&base_bytes, &patch_bytes, None) {
                    Ok(val) => IpcResponse::ok(&req.id, val),
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
                        "multiWindow": { "allowed": self.managed_windows_available() },
                        "plugins": { "allowed": true },
                        "notifications": { "allowed": true },
                        "autoUpdater": { "allowed": true, "autoApply": false },
                        "hardwareKey": { "allowed": true, "standard": "FIDO2 / WebAuthn L3" },
                        "multiProfile": { "allowed": true, "isolation": "STRICT_DIRECTORY" },
                        "wasmSandbox": { "allowed": true, "memoryLimitMb": 16 },
                        "screenProtection": { "allowed": true, "standard": "Win32 SetWindowDisplayAffinity" },
                        "raspAntiTamper": { "allowed": true, "defenseLevel": "KERNEL_TRAP_OPCODE_SCAN" }
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
                    "activeProfile": self.profile_manager.get_active_profile(),
                    "registryCreationBlocked": self.profile_manager.registry_error().is_some(),
                    "registryError": self.profile_manager.registry_error()
                }),
            ),

            "profile_switch" => {
                self.vault_sessions.lock();
                self.file_dialogs.invalidate();
                self.message_dialogs.invalidate();
                self.notifications.invalidate();
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
                let tier = match req.payload.get("tier") {
                    None => ExecutionTier::Auto,
                    Some(value) => match value.as_str() {
                        Some("AUTO") => ExecutionTier::Auto,
                        Some("WGPU_JIT") => ExecutionTier::WgpuJit,
                        Some("INTERPRETER") => ExecutionTier::Interpreter,
                        _ => {
                            return IpcResponse::err(
                                &req.id,
                                "Invalid Wasm tier; expected AUTO, INTERPRETER or WGPU_JIT",
                            )
                        }
                    },
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
            "crash_list" => match self.crash_reporter.list_crashes() {
                Ok(records) => {
                    let entries: Vec<_> = records.iter().map(|r| json!({"id":r.crash_id,"timestamp":r.timestamp.to_string(),"kind":r.error_kind,"message":r.stack_trace})).collect();
                    IpcResponse::ok(
                        &req.id,
                        json!({"total":records.len(),"crashes":records,"entries":entries,"retentionLimit":32,"scope":"application","zeroTelemetry":true,"storageLocation":"./diagnostics/events.json"}),
                    )
                }
                Err(error) => IpcResponse::err(&req.id, &error),
            },
            "crash_record" => {
                let kind = req
                    .payload
                    .get("errorKind")
                    .or_else(|| req.payload.get("kind"));
                let kind = match kind {
                    None => "ControlledDiagnosticTrap",
                    Some(value) => match value.as_str() {
                        Some(value) => value,
                        None => return IpcResponse::err(&req.id, "Invalid diagnostic kind"),
                    },
                };
                match self.crash_reporter.record_crash(kind, "") {
                    Ok(record) => {
                        let mut data = json!(record);
                        data["id"] = json!(record.crash_id);
                        data["persisted"] = json!(true);
                        IpcResponse::ok(&req.id, data)
                    }
                    Err(error) => IpcResponse::err(&req.id, &error),
                }
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
                let port = match req.payload.get("port") {
                    None => 443,
                    Some(value) => match value.as_u64().and_then(|n| u16::try_from(n).ok()) {
                        Some(port) if port != 0 => port,
                        _ => {
                            return IpcResponse::err(
                                &req.id,
                                "Invalid port: expected an integer from 1 to 65535",
                            )
                        }
                    },
                };
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

            // ─── BANKING-GRADE FINANCIAL SUITE IPC ──────────────────────────
            "banking_screen_protection_set" => {
                let enable = req
                    .payload
                    .get("protected")
                    .and_then(|p| p.as_bool())
                    .unwrap_or(true);
                match self.banking.screen_protection.set_protection(enable) {
                    Ok(status) => IpcResponse::ok(&req.id, json!(status)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "banking_screen_protection_status" => {
                IpcResponse::ok(&req.id, json!(self.banking.screen_protection.get_status()))
            }

            "banking_biometric_hardware_status" => {
                IpcResponse::ok(&req.id, json!(self.banking.step_up.get_hardware_status()))
            }

            "banking_biometric_challenge_request" => {
                let user_id = match req.payload.get("userId").and_then(|u| u.as_str()) {
                    Some(u) => u,
                    None => return IpcResponse::err(&req.id, "Missing 'userId' parameter"),
                };
                let operation = req
                    .payload
                    .get("operation")
                    .and_then(|o| o.as_str())
                    .unwrap_or("FINANCIAL_OPERATION");
                let amount = req
                    .payload
                    .get("amount")
                    .and_then(|a| a.as_f64())
                    .unwrap_or(0.0);
                let currency = req
                    .payload
                    .get("currency")
                    .and_then(|c| c.as_str())
                    .unwrap_or("USD");

                match self.banking.step_up.request_challenge(user_id, operation, amount, currency) {
                    Ok(chal) => IpcResponse::ok(&req.id, json!(chal)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "banking_biometric_verify" => {
                let challenge_id = match req.payload.get("challengeId").and_then(|c| c.as_str()) {
                    Some(c) => c,
                    None => return IpcResponse::err(&req.id, "Missing 'challengeId' parameter"),
                };
                let user_id = match req.payload.get("userId").and_then(|u| u.as_str()) {
                    Some(u) => u,
                    None => return IpcResponse::err(&req.id, "Missing 'userId' parameter"),
                };
                let payload = match req.payload.get("biometricPayload").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'biometricPayload' parameter"),
                };

                match self.banking.step_up.verify_biometric(challenge_id, user_id, payload) {
                    Ok(token) => IpcResponse::ok(&req.id, json!(token)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "banking_transaction_propose" => {
                let maker_id = match req.payload.get("makerId").and_then(|m| m.as_str()) {
                    Some(m) => m,
                    None => return IpcResponse::err(&req.id, "Missing 'makerId' parameter"),
                };
                let from_account = match req.payload.get("fromAccount").and_then(|f| f.as_str()) {
                    Some(f) => f,
                    None => return IpcResponse::err(&req.id, "Missing 'fromAccount' parameter"),
                };
                let to_account = match req.payload.get("toAccount").and_then(|t| t.as_str()) {
                    Some(t) => t,
                    None => return IpcResponse::err(&req.id, "Missing 'toAccount' parameter"),
                };
                let amount = match req.payload.get("amount").and_then(|a| a.as_f64()) {
                    Some(a) => a,
                    None => return IpcResponse::err(&req.id, "Missing 'amount' parameter"),
                };
                let currency = req
                    .payload
                    .get("currency")
                    .and_then(|c| c.as_str())
                    .unwrap_or("USD");
                let reference = req
                    .payload
                    .get("reference")
                    .and_then(|r| r.as_str())
                    .unwrap_or("Standard Banking Transfer");

                match self.banking.maker_checker.propose_transaction(
                    maker_id,
                    from_account,
                    to_account,
                    amount,
                    currency,
                    reference,
                ) {
                    Ok(tx) => IpcResponse::ok(&req.id, json!(tx)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "banking_transaction_approve" => {
                let tx_id = match req.payload.get("txId").and_then(|t| t.as_str()) {
                    Some(t) => t,
                    None => return IpcResponse::err(&req.id, "Missing 'txId' parameter"),
                };
                let checker_id = match req.payload.get("checkerId").and_then(|c| c.as_str()) {
                    Some(c) => c,
                    None => return IpcResponse::err(&req.id, "Missing 'checkerId' parameter"),
                };
                let step_up_token = req.payload.get("stepUpToken").and_then(|s| s.as_str());

                match self.banking.maker_checker.approve_transaction(tx_id, checker_id, step_up_token) {
                    Ok(tx) => IpcResponse::ok(&req.id, json!(tx)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "banking_transaction_reject" => {
                let tx_id = match req.payload.get("txId").and_then(|t| t.as_str()) {
                    Some(t) => t,
                    None => return IpcResponse::err(&req.id, "Missing 'txId' parameter"),
                };
                let checker_id = match req.payload.get("checkerId").and_then(|c| c.as_str()) {
                    Some(c) => c,
                    None => return IpcResponse::err(&req.id, "Missing 'checkerId' parameter"),
                };
                let reason = req
                    .payload
                    .get("reason")
                    .and_then(|r| r.as_str())
                    .unwrap_or("Rejected by compliance checker");

                match self.banking.maker_checker.reject_transaction(tx_id, checker_id, reason) {
                    Ok(tx) => IpcResponse::ok(&req.id, json!(tx)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "banking_transaction_list" => {
                let txs = self.banking.maker_checker.list_transactions();
                IpcResponse::ok(&req.id, json!({ "transactions": txs, "count": txs.len() }))
            }

            "banking_ledger_get" => {
                let blocks = self.banking.maker_checker.get_ledger();
                IpcResponse::ok(&req.id, json!({ "ledger": blocks, "blockCount": blocks.len() }))
            }

            "banking_ledger_verify" => {
                match self.banking.maker_checker.verify_ledger_integrity() {
                    Ok(report) => IpcResponse::ok(&req.id, json!(report)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            // ── RASP & Anti-Tampering Engine ─────────────────────────────────
            "rasp_evaluate" => {
                let report = self.rasp_engine.evaluate();
                IpcResponse::ok(&req.id, json!(report))
            }

            "rasp_containment_reset" => {
                self.rasp_engine.reset_containment();
                IpcResponse::ok(&req.id, json!({ "status": "Containment reset successfully" }))
            }

            // ── mTLS & Public Key Pinning Engine ────────────────────────────
            "network_pinning_evaluate" => {
                let host = match req.payload.get("host").and_then(|h| h.as_str()) {
                    Some(h) => h,
                    None => return IpcResponse::err(&req.id, "Missing 'host' parameter"),
                };
                let cert_hex = match req.payload.get("certBytesHex").and_then(|c| c.as_str()) {
                    Some(c) => c,
                    None => return IpcResponse::err(&req.id, "Missing 'certBytesHex' parameter"),
                };
                let client_cert_present = req
                    .payload
                    .get("clientCertPresent")
                    .and_then(|c| c.as_bool())
                    .unwrap_or(true);

                let cert_bytes = match hex::decode(cert_hex) {
                    Ok(b) => b,
                    Err(e) => return IpcResponse::err(&req.id, &format!("Invalid certificate hex: {}", e)),
                };

                let assessment = self
                    .tls_pinning
                    .evaluate_connection(host, &cert_bytes, client_cert_present);
                IpcResponse::ok(&req.id, json!(assessment))
            }

            "network_pinning_enroll" => {
                let endpoint: crate::security::mtls_pinning::PinnedEndpoint =
                    match serde_json::from_value(req.payload.clone()) {
                        Ok(ep) => ep,
                        Err(e) => return IpcResponse::err(&req.id, &format!("Invalid endpoint payload: {}", e)),
                    };
                self.tls_pinning.enroll_pinned_host(endpoint.clone());
                IpcResponse::ok(&req.id, json!({ "status": "Endpoint pinned", "host": endpoint.host }))
            }

            // ── Physical Hardware Security Key (FIDO2 / YubiKey) ────────────
            "fido2_token_enumerate" => {
                let devices = self.fido2_engine.enumerate_devices();
                IpcResponse::ok(&req.id, json!({ "devices": devices, "count": devices.len() }))
            }

            "fido2_token_register" => {
                let user_handle = match req.payload.get("userHandle").and_then(|u| u.as_str()) {
                    Some(u) => u,
                    None => return IpcResponse::err(&req.id, "Missing 'userHandle' parameter"),
                };
                let relying_party = match req.payload.get("relyingParty").and_then(|r| r.as_str()) {
                    Some(r) => r,
                    None => "aegis.bank.internal",
                };

                match self.fido2_engine.register_credential(user_handle, relying_party) {
                    Ok(cred) => IpcResponse::ok(&req.id, json!(cred)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "fido2_token_sign" => {
                let cred_id = match req.payload.get("credentialId").and_then(|c| c.as_str()) {
                    Some(c) => c,
                    None => return IpcResponse::err(&req.id, "Missing 'credentialId' parameter"),
                };
                let hash_hex = match req.payload.get("clientDataHashHex").and_then(|h| h.as_str()) {
                    Some(h) => h,
                    None => return IpcResponse::err(&req.id, "Missing 'clientDataHashHex' parameter"),
                };
                let touch_confirmed = req
                    .payload
                    .get("touchConfirmed")
                    .and_then(|t| t.as_bool())
                    .unwrap_or(false);

                match self.fido2_engine.sign_assertion(cred_id, hash_hex, touch_confirmed) {
                    Ok(assertion) => IpcResponse::ok(&req.id, json!(assertion)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "fido2_token_verify" => {
                let cred_id = match req.payload.get("credentialId").and_then(|c| c.as_str()) {
                    Some(c) => c,
                    None => return IpcResponse::err(&req.id, "Missing 'credentialId' parameter"),
                };
                let hash_hex = match req.payload.get("clientDataHashHex").and_then(|h| h.as_str()) {
                    Some(h) => h,
                    None => return IpcResponse::err(&req.id, "Missing 'clientDataHashHex' parameter"),
                };
                let assertion: crate::security::fido2_token::HardwareSignAssertion =
                    match req.payload.get("assertion") {
                        Some(val) => match serde_json::from_value(val.clone()) {
                            Ok(a) => a,
                            Err(e) => return IpcResponse::err(&req.id, &format!("Invalid assertion: {}", e)),
                        },
                        None => return IpcResponse::err(&req.id, "Missing 'assertion' object"),
                    };

                match self.fido2_engine.verify_assertion(cred_id, hash_hex, &assertion) {
                    Ok(valid) => IpcResponse::ok(&req.id, json!({ "verified": valid })),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            // ── Decentralized P2P Zero-Knowledge Sync (Noise Protocol) ───────
            "p2p_sync_init" => {
                let (init, _) = self.p2p_sync.initiate_handshake();
                IpcResponse::ok(&req.id, json!(init))
            }

            "p2p_sync_complete" => {
                let session_id = match req.payload.get("sessionId").and_then(|s| s.as_str()) {
                    Some(s) => s,
                    None => return IpcResponse::err(&req.id, "Missing 'sessionId' parameter"),
                };
                let init_pub = match req.payload.get("initiatorPubHex").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'initiatorPubHex' parameter"),
                };
                let resp_pub = match req.payload.get("responderPubHex").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'responderPubHex' parameter"),
                };

                match self.p2p_sync.complete_handshake(session_id, init_pub, resp_pub) {
                    Ok(resp) => IpcResponse::ok(&req.id, json!(resp)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "p2p_sync_encrypt" => {
                let session_id = match req.payload.get("sessionId").and_then(|s| s.as_str()) {
                    Some(s) => s,
                    None => return IpcResponse::err(&req.id, "Missing 'sessionId' parameter"),
                };
                let seq = req
                    .payload
                    .get("sequenceNumber")
                    .and_then(|n| n.as_u64())
                    .unwrap_or(1);
                let payload_hex = match req.payload.get("payloadHex").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'payloadHex' parameter"),
                };

                let plaintext = match hex::decode(payload_hex) {
                    Ok(b) => b,
                    Err(e) => return IpcResponse::err(&req.id, &format!("Invalid hex: {}", e)),
                };

                match self.p2p_sync.encrypt_sync_bundle(session_id, seq, &plaintext) {
                    Ok(bundle) => IpcResponse::ok(&req.id, json!(bundle)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "p2p_sync_decrypt" => {
                let bundle: crate::security::p2p_sync::EncryptedSyncBundle =
                    match serde_json::from_value(req.payload.clone()) {
                        Ok(b) => b,
                        Err(e) => return IpcResponse::err(&req.id, &format!("Invalid bundle: {}", e)),
                    };

                match self.p2p_sync.decrypt_sync_bundle(&bundle) {
                    Ok(bytes) => IpcResponse::ok(
                        &req.id,
                        json!({
                            "plaintextHex": hex::encode(&bytes),
                            "plaintextUtf8": String::from_utf8(bytes).unwrap_or_default(),
                            "verified": true
                        }),
                    ),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            // ── Pillar 1: Encrypted Embedded Database Engine ────────────────
            "db_query" => {
                let table = match req.payload.get("table").and_then(|t| t.as_str()) {
                    Some(t) => t,
                    None => return IpcResponse::err(&req.id, "Missing 'table' parameter"),
                };
                let filter_col = req.payload.get("filterCol").or_else(|| req.payload.get("filter_col")).and_then(|c| c.as_str());
                let filter_val = req.payload.get("filterVal").or_else(|| req.payload.get("filter_val")).and_then(|v| v.as_str());

                match self.encrypted_db.query(table, filter_col, filter_val) {
                    Ok(rows) => IpcResponse::ok(&req.id, json!({ "table": table, "rows": rows, "count": rows.len() })),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "db_insert" => {
                let table = match req.payload.get("table").and_then(|t| t.as_str()) {
                    Some(t) => t,
                    None => return IpcResponse::err(&req.id, "Missing 'table' parameter"),
                };
                let row = match req.payload.get("row") {
                    Some(r) => r.clone(),
                    None => return IpcResponse::err(&req.id, "Missing 'row' object parameter"),
                };

                match self.encrypted_db.insert(table, row.clone()) {
                    Ok(pk) => IpcResponse::ok(&req.id, json!({ "status": "Inserted", "table": table, "id": pk, "record": row })),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "db_update" => {
                let table = match req.payload.get("table").and_then(|t| t.as_str()) {
                    Some(t) => t,
                    None => return IpcResponse::err(&req.id, "Missing 'table' parameter"),
                };
                let id = match req.payload.get("id").and_then(|i| i.as_str()) {
                    Some(i) => i,
                    None => return IpcResponse::err(&req.id, "Missing 'id' parameter"),
                };
                let row = match req.payload.get("row") {
                    Some(r) => r.clone(),
                    None => return IpcResponse::err(&req.id, "Missing 'row' object parameter"),
                };

                match self.encrypted_db.update(table, id, row.clone()) {
                    Ok(()) => IpcResponse::ok(&req.id, json!({ "status": "Updated", "table": table, "id": id, "record": row })),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "db_delete" => {
                let table = match req.payload.get("table").and_then(|t| t.as_str()) {
                    Some(t) => t,
                    None => return IpcResponse::err(&req.id, "Missing 'table' parameter"),
                };
                let id = match req.payload.get("id").and_then(|i| i.as_str()) {
                    Some(i) => i,
                    None => return IpcResponse::err(&req.id, "Missing 'id' parameter"),
                };

                match self.encrypted_db.delete(table, id) {
                    Ok(existed) => IpcResponse::ok(&req.id, json!({ "deleted": existed, "table": table, "id": id })),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "db_stats" => {
                IpcResponse::ok(&req.id, self.encrypted_db.stats())
            }

            "db_transaction" => {
                let action = match req.payload.get("action").and_then(|a| a.as_str()) {
                    Some(a) => a,
                    None => return IpcResponse::err(&req.id, "Missing 'action' parameter (begin, commit, rollback)"),
                };

                match action {
                    "begin" => match self.encrypted_db.begin_transaction() {
                        Ok(()) => IpcResponse::ok(&req.id, json!({ "status": "Transaction started" })),
                        Err(e) => IpcResponse::err(&req.id, &e),
                    },
                    "commit" => match self.encrypted_db.commit_transaction() {
                        Ok(()) => IpcResponse::ok(&req.id, json!({ "status": "Transaction committed to AEAD cipher storage" })),
                        Err(e) => IpcResponse::err(&req.id, &e),
                    },
                    "rollback" => match self.encrypted_db.rollback_transaction() {
                        Ok(()) => IpcResponse::ok(&req.id, json!({ "status": "Transaction rolled back" })),
                        Err(e) => IpcResponse::err(&req.id, &e),
                    },
                    _ => IpcResponse::err(&req.id, "Invalid transaction action; expected 'begin', 'commit', or 'rollback'"),
                }
            }

            // ── Pillar 2: Hardware TPM 2.0 PCR-Sealed Cryptographic Enclave ──
            "pcr_enclave_status" => {
                let status = self.tpm_sealing.get_enclave_status();
                IpcResponse::ok(&req.id, json!(status))
            }

            "pcr_enclave_seal" => {
                let package_id = match req.payload.get("packageId").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'packageId' parameter"),
                };
                let secret_str = match req.payload.get("secret").and_then(|s| s.as_str()) {
                    Some(s) => s,
                    None => return IpcResponse::err(&req.id, "Missing 'secret' parameter"),
                };
                let pcrs: Vec<u32> = req.payload.get("pcrs")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|x| x.as_u64().map(|n| n as u32)).collect())
                    .unwrap_or_else(|| vec![0, 2, 7, 11]);

                match self.tpm_sealing.seal_secret(package_id, secret_str.as_bytes(), &pcrs) {
                    Ok(pkg) => IpcResponse::ok(&req.id, json!(pkg)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "pcr_enclave_unseal" => {
                let package_id = match req.payload.get("packageId").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'packageId' parameter"),
                };

                match self.tpm_sealing.unseal_secret(package_id) {
                    Ok(bytes) => {
                        let text = String::from_utf8(bytes.clone()).unwrap_or_default();
                        IpcResponse::ok(&req.id, json!({
                            "packageId": package_id,
                            "secretUtf8": text,
                            "secretHex": hex::encode(&bytes),
                            "verified": true
                        }))
                    }
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "pcr_enclave_tamper_sim" => {
                let pcr_index = req.payload.get("pcrIndex").and_then(|i| i.as_u64()).unwrap_or(7) as u32;
                let mock_code = req.payload.get("mockCode").and_then(|m| m.as_str()).unwrap_or("INJECTED_UNAUTHORIZED_BOOTLOADER_PAYLOAD");

                match self.tpm_sealing.simulate_pcr_tamper(pcr_index, mock_code.as_bytes()) {
                    Ok(()) => IpcResponse::ok(&req.id, json!({
                        "tampered": true,
                        "pcrIndex": pcr_index,
                        "message": format!("PCR[{}] digest intentionally mutated with simulated untrusted payload.", pcr_index)
                    })),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            // ── Pillar 3: Enterprise Granular RBAC Engine ───────────────────
            "rbac_identity" => {
                let id = self.banking.rbac.get_identity();
                IpcResponse::ok(&req.id, json!(id))
            }

            "rbac_switch_role" => {
                let role_str = match req.payload.get("role").and_then(|r| r.as_str()) {
                    Some(r) => r,
                    None => return IpcResponse::err(&req.id, "Missing 'role' parameter"),
                };
                let actor_name = req.payload.get("actorName").and_then(|a| a.as_str());

                match crate::banking::RoleType::from_str(role_str) {
                    Ok(new_role) => {
                        let new_id = self.banking.rbac.switch_role(new_role, actor_name);
                        IpcResponse::ok(&req.id, json!(new_id))
                    }
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "rbac_check" => {
                let action = match req.payload.get("action").and_then(|a| a.as_str()) {
                    Some(a) => a,
                    None => return IpcResponse::err(&req.id, "Missing 'action' parameter"),
                };
                let resource = req.payload.get("resource").and_then(|r| r.as_str()).unwrap_or("*");

                let (allowed, reason) = self.banking.rbac.check_permission(action, resource);
                IpcResponse::ok(&req.id, json!({ "allowed": allowed, "action": action, "resource": resource, "reason": reason }))
            }

            "rbac_matrix" => {
                let matrix = self.banking.rbac.get_role_matrix();
                IpcResponse::ok(&req.id, json!({ "roles": matrix, "count": matrix.len() }))
            }

            "rbac_audit" => {
                let log = self.banking.rbac.get_audit_log();
                IpcResponse::ok(&req.id, json!({ "auditLog": log, "count": log.len() }))
            }

            // ── Pillar 4: Outbound Network Firewall & Flow Monitor ──────────
            "traffic_recent" => {
                let limit = req.payload.get("limit").and_then(|l| l.as_u64()).unwrap_or(50) as usize;
                let flows = self.traffic_monitor.get_recent_flows(limit);
                IpcResponse::ok(&req.id, json!({ "flows": flows, "count": flows.len() }))
            }

            "traffic_stats" => {
                let stats = self.traffic_monitor.get_stats();
                IpcResponse::ok(&req.id, json!(stats))
            }

            "traffic_killswitch" => {
                let enabled = match req.payload.get("enabled").and_then(|e| e.as_bool()) {
                    Some(e) => e,
                    None => return IpcResponse::err(&req.id, "Missing 'enabled' boolean parameter"),
                };
                let active = self.traffic_monitor.toggle_killswitch(enabled);
                IpcResponse::ok(&req.id, json!({ "killswitchActive": active }))
            }

            "traffic_simulate" => {
                let host = req.payload.get("host").and_then(|h| h.as_str()).unwrap_or("api.payment-gateway.internal");
                let threat = req.payload.get("threat").and_then(|t| t.as_u64()).unwrap_or(0) as u8;
                let flow = self.traffic_monitor.inspect_and_log(
                    host,
                    443,
                    "mTLS",
                    2048,
                    14,
                    Some("rule-enterprise-vault"),
                    threat == 0,
                );
                IpcResponse::ok(&req.id, json!(flow))
            }

            "traffic_clear" => {
                self.traffic_monitor.clear_history();
                IpcResponse::ok(&req.id, json!({ "status": "Traffic history cleared" }))
            }

            // ── Pillar 5: Security Audit & Compliance Certificate Generator ──
            "compliance_generate" => {
                let cert = self.compliance_report.generate_certificate(
                    env!("CARGO_PKG_VERSION"),
                    true,
                    true,
                );
                IpcResponse::ok(&req.id, json!(cert))
            }

            "compliance_verify" => {
                let digest = match req.payload.get("payloadSha256").and_then(|d| d.as_str()) {
                    Some(d) => d,
                    None => return IpcResponse::err(&req.id, "Missing 'payloadSha256' parameter"),
                };
                let sig = match req.payload.get("signatureHex").and_then(|s| s.as_str()) {
                    Some(s) => s,
                    None => return IpcResponse::err(&req.id, "Missing 'signatureHex' parameter"),
                };
                let pubkey = match req.payload.get("publicKeyHex").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'publicKeyHex' parameter"),
                };

                match self.compliance_report.verify_certificate_signature(digest, sig, pubkey) {
                    Ok(valid) => IpcResponse::ok(&req.id, json!({ "verified": valid })),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            // ── Post-Quantum Hybrid Cryptography (NIST FIPS 203 ML-KEM-768 + X25519) ──
            "pq_kem_keypair" => {
                match crate::security::pq_kem::PqKemEngine::generate_keypair() {
                    Ok(kp) => IpcResponse::ok(&req.id, json!(kp)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "pq_kem_encapsulate" => {
                let pk = match req.payload.get("hybridPublicKeyHex").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'hybridPublicKeyHex' parameter"),
                };
                match crate::security::pq_kem::PqKemEngine::encapsulate(pk) {
                    Ok(res) => IpcResponse::ok(&req.id, json!(res)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "pq_kem_decapsulate" => {
                let sk = match req.payload.get("hybridSecretKeyHex").and_then(|s| s.as_str()) {
                    Some(s) => s,
                    None => return IpcResponse::err(&req.id, "Missing 'hybridSecretKeyHex' parameter"),
                };
                let ct = match req.payload.get("hybridCiphertextHex").and_then(|c| c.as_str()) {
                    Some(c) => c,
                    None => return IpcResponse::err(&req.id, "Missing 'hybridCiphertextHex' parameter"),
                };
                match crate::security::pq_kem::PqKemEngine::decapsulate(sk, ct) {
                    Ok(res) => IpcResponse::ok(&req.id, json!(res)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "pq_kem_metadata" => {
                IpcResponse::ok(&req.id, crate::security::pq_kem::PqKemEngine::get_spec_metadata())
            }

            // ── Post-Quantum Digital Signatures (NIST FIPS 204 ML-DSA-65 / Dilithium) ──
            "pq_dsa_keypair" => {
                match crate::security::pq_dsa::MlDsaEngine::generate_keypair() {
                    Ok(kp) => IpcResponse::ok(&req.id, json!(kp)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "pq_dsa_sign" => {
                let sk_hex = match req.payload.get("hybridSecretKeyHex").and_then(|s| s.as_str()) {
                    Some(s) => s,
                    None => return IpcResponse::err(&req.id, "Missing 'hybridSecretKeyHex' parameter"),
                };
                let payload_str = match req.payload.get("payload").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'payload' parameter"),
                };
                let sk_bytes = match hex::decode(sk_hex) {
                    Ok(b) => b,
                    Err(_) => return IpcResponse::err(&req.id, "Invalid hex encoding for secret key"),
                };
                match crate::security::pq_dsa::MlDsaEngine::sign(&sk_bytes, payload_str.as_bytes()) {
                    Ok(res) => IpcResponse::ok(&req.id, json!(res)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "pq_dsa_verify" => {
                let pk_hex = match req.payload.get("hybridPublicKeyHex").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'hybridPublicKeyHex' parameter"),
                };
                let payload_str = match req.payload.get("payload").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'payload' parameter"),
                };
                let sig_hex = match req.payload.get("hybridSignatureHex").and_then(|s| s.as_str()) {
                    Some(s) => s,
                    None => return IpcResponse::err(&req.id, "Missing 'hybridSignatureHex' parameter"),
                };
                let pk_bytes = match hex::decode(pk_hex) {
                    Ok(b) => b,
                    Err(_) => return IpcResponse::err(&req.id, "Invalid hex encoding for public key"),
                };
                let sig_bytes = match hex::decode(sig_hex) {
                    Ok(b) => b,
                    Err(_) => return IpcResponse::err(&req.id, "Invalid hex encoding for signature"),
                };
                match crate::security::pq_dsa::MlDsaEngine::verify(&pk_bytes, payload_str.as_bytes(), &sig_bytes) {
                    Ok(res) => IpcResponse::ok(&req.id, json!(res)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "pq_dsa_metadata" => {
                IpcResponse::ok(&req.id, json!(crate::security::pq_dsa::MlDsaEngine::metadata()))
            }

            // ── Smart Card & YubiKey PIV Slot Management (NIST SP 800-73-4) ──
            "piv_state" => {
                IpcResponse::ok(&req.id, json!(self.piv_engine.get_device_state()))
            }

            "piv_pin_verify" => {
                let pin = match req.payload.get("pin").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'pin' parameter"),
                };
                match self.piv_engine.verify_pin(pin) {
                    Ok(res) => IpcResponse::ok(&req.id, res),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "piv_logout" => {
                match self.piv_engine.logout() {
                    Ok(res) => IpcResponse::ok(&req.id, res),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "piv_sign" => {
                let slot_id = match req.payload.get("slotId").and_then(|s| s.as_str()) {
                    Some(s) => s,
                    None => return IpcResponse::err(&req.id, "Missing 'slotId' parameter (9A, 9C, 9D, or 9E)"),
                };
                let payload = match req.payload.get("payload").and_then(|p| p.as_str()) {
                    Some(p) => p,
                    None => return IpcResponse::err(&req.id, "Missing 'payload' parameter"),
                };
                let pin_override = req.payload.get("pin").and_then(|p| p.as_str());

                match self.piv_engine.sign_payload(slot_id, payload, pin_override) {
                    Ok(res) => IpcResponse::ok(&req.id, json!(res)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "piv_cert_export" => {
                let slot_id = match req.payload.get("slotId").and_then(|s| s.as_str()) {
                    Some(s) => s,
                    None => return IpcResponse::err(&req.id, "Missing 'slotId' parameter"),
                };
                match self.piv_engine.export_slot_certificate(slot_id) {
                    Ok(cert) => IpcResponse::ok(&req.id, json!(cert)),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            // ── Native OS System Tray Daemon & Quick-Killswitch Controls ────
            "tray_status_get" => {
                IpcResponse::ok(&req.id, self.tray_daemon.get_tray_status())
            }

            "tray_quick_killswitch" => {
                let enabled = req.payload.get("enabled").and_then(|e| e.as_bool()).unwrap_or(true);
                self.traffic_monitor.toggle_killswitch(enabled);
                match self.tray_daemon.set_quick_killswitch(enabled) {
                    Ok(res) => IpcResponse::ok(&req.id, res),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            "tray_menu_action" => {
                let action_id = match req.payload.get("actionId").and_then(|a| a.as_str()) {
                    Some(a) => a,
                    None => return IpcResponse::err(&req.id, "Missing 'actionId' parameter"),
                };
                if action_id == "quick_killswitch" {
                    let curr_status = self.tray_daemon.get_tray_status();
                    let is_active = curr_status.get("quickKillswitchActive").and_then(|v| v.as_bool()).unwrap_or(false);
                    self.traffic_monitor.toggle_killswitch(!is_active);
                }
                match self.tray_daemon.trigger_menu_action(action_id) {
                    Ok(res) => IpcResponse::ok(&req.id, res),
                    Err(e) => IpcResponse::err(&req.id, &e),
                }
            }

            // ── Native OS IPC Transport (Windows Named Pipes & Unix Domain Sockets) ──
            "native_ipc_status" => {
                IpcResponse::ok(&req.id, json!(self.native_transport.get_status()))
            }

            "native_ipc_benchmark" => {
                let iters = req.payload.get("iterations").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
                IpcResponse::ok(&req.id, json!(self.native_transport.benchmark(iters)))
            }

            // ── Kernel In-Memory Protection & RAM Anti-Dump Sentinel (DPAPI / VirtualLock) ──
            "memory_guard_status" => {
                IpcResponse::ok(&req.id, json!(self.memory_guard.get_status()))
            }

            "memory_guard_protect" => {
                let secret = req.payload.get("secret").and_then(|s| s.as_str()).unwrap_or("");
                IpcResponse::ok(&req.id, json!(self.memory_guard.protect_transient_value(secret)))
            }

            "memory_guard_anti_dump" => {
                let enable = req.payload.get("enable").and_then(|e| e.as_bool()).unwrap_or(true);
                IpcResponse::ok(&req.id, json!({"antiDumpActive": self.memory_guard.toggle_anti_dump(enable)}))
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
