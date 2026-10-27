mod tempfile {
    pub struct TempDir(std::path::PathBuf);
    impl TempDir {
        pub fn path(&self) -> &std::path::Path {
            &self.0
        }
    }
    pub fn tempdir() -> std::io::Result<TempDir> {
        let mut bytes = [0u8; 16];
        getrandom::getrandom(&mut bytes).expect("secure random");
        let p = std::env::temp_dir().join(format!("aegis-regression-{}", hex::encode(bytes)));
        std::fs::create_dir(&p)?;
        Ok(TempDir(p))
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            if self.0.parent() == Some(std::env::temp_dir().as_path())
                && self
                    .0
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("aegis-regression-")
            {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
    }
}

#[test]
fn portable_bundles_use_the_native_binary_and_reject_other_targets() {
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(d.path().join("src-core/target/release")).unwrap();
    std::fs::create_dir_all(d.path().join("src-ui/dist")).unwrap();
    let executable = if cfg!(windows) {
        "aegis-core.exe"
    } else {
        "aegis-core"
    };
    std::fs::write(
        d.path().join("src-core/target/release").join(executable),
        b"fixture executable",
    )
    .unwrap();
    std::fs::write(d.path().join("src-ui/dist/index.html"), "fixture UI").unwrap();
    std::fs::write(d.path().join("aegis.config.json"), "{}").unwrap();
    let mut manifest = crate::bundler::BundleManifest::default();
    manifest.target_platforms = vec!["unsupported-target".into()];
    assert!(
        crate::bundler::BundleEngine::bundle(d.path(), &manifest).is_err(),
        "Bundler must not label a host binary as a different target"
    );
    manifest.target_platforms = vec![std::env::consts::OS.into()];
    std::fs::write(
        d.path().join("aegis.config.json"),
        r#"{"appId":"../outside"}"#,
    )
    .unwrap();
    assert!(
        crate::bundler::BundleEngine::bundle(d.path(), &manifest).is_err(),
        "Bundling must reject invalid explicit app identity before writing output"
    );
    assert!(!d.path().join("dist-bundle").exists());
    std::fs::write(d.path().join("aegis.config.json"), "{}").unwrap();
    let report = crate::bundler::BundleEngine::bundle(d.path(), &manifest).unwrap();
    assert!(report.success);
    assert_eq!(
        std::fs::read(std::path::Path::new(&report.output_dir).join(executable)).unwrap(),
        b"fixture executable"
    );
    assert_eq!(report.installer_script_path.is_some(), cfg!(windows));
}
use super::*;
#[test]
fn hardware_providers_fail_closed() {
    let h = HardwareKeyEngine::new();
    assert!(h.list_keys().is_empty());
    assert!(h.enroll_key("fake", "anything").is_err());
    assert!(h.generate_challenge("untrusted.example").is_err());
    assert!(h.verify_signature("id", "pin", "{}", "00", "00").is_err());
    let h = HsmPkcs11Engine::new();
    assert!(h.list_slots().is_empty());
    assert!(h.login_token(0, "anything").is_err());
    assert!(h.sign_payload(0, "fake", "payload").is_err());
    assert!(TpmAttestationEngine::new()
        .generate_quote("nonce", &[0, 7])
        .is_err());
    assert!(BiometricEnclave::new().authenticate("login").is_err());
}
#[test]
fn marketplace_never_claims_fake_installs() {
    let m = crate::p2p_market::P2PPluginMarketplace::new();
    assert!(m.list_plugins().is_empty());
    assert!(m.list_peers().is_empty());
    assert!(m.sign_plugin("plugin", "trusted_did").is_err());
    assert!(m.install_plugin("plugin").is_err());
}
#[test]
fn paths_and_quotas() {
    let dir = tempfile::tempdir().unwrap();
    let s = SecurityEngine::new(CapabilityManifest::default(), dir.path());
    for p in [
        "../outside",
        "/absolute",
        "C:/outside",
        "sub/../../outside",
        "sub\\outside",
        "file:stream",
    ] {
        assert!(s.check_fs_write(p).is_err(), "{p}");
    }
    let p = s.check_fs_write("sub/note.txt").unwrap();
    std::fs::write(&p, "1234").unwrap();
    assert!(s.check_write_quota("sub/note.txt", 5, 4).is_err());
    assert!(s.check_write_quota("sub/note.txt", 4, 4).is_ok());
    let mut denied = CapabilityManifest::default();
    denied.fs.allow_write.clear();
    assert!(SecurityEngine::new(denied, dir.path())
        .check_fs_write("x")
        .is_err());
}
#[test]
fn profiles_survive_restart_and_do_not_collide() {
    let d = tempfile::tempdir().unwrap();
    let p = crate::profile::ProfileManager::new(d.path());
    assert!(p.create_profile("bad/name", "bad", "").is_err());
    p.create_profile("work", "Work", "").unwrap();
    let original = p.get_active_sandbox_path();
    p.switch_profile("work").unwrap();
    assert_ne!(original, p.get_active_sandbox_path());
    drop(p);
    assert!(crate::profile::ProfileManager::new(d.path())
        .list_profiles()
        .iter()
        .any(|p| p.id == "work"));
}
#[test]
fn secure_session_tokens() {
    let a = HandshakeSession::new();
    let b = HandshakeSession::new();
    assert_ne!(a.get_token(), b.get_token());
    assert!(a.validate_nonce(&a.get_token()));
    assert!(!a.validate_nonce(&b.get_token()));
}
#[test]
fn manifest_camel_case_is_enforced() {
    let v = serde_json::json!({"fs":{"allowRead":[],"allowWrite":[],"denyPaths":[]},"system":{"allowSystemInfo":false,"allowDialogs":false,"allowShellExecution":false},"crypto":{"allowHashing":false,"allowRandomGeneration":false}});
    let d = tempfile::tempdir().unwrap();
    let s = SecurityEngine::new(serde_json::from_value(v).unwrap(), d.path());
    assert!(s.check_system_info().is_err());
    assert!(s.check_crypto().is_err());
}
#[test]
fn incomplete_and_cross_platform_bundles_fail() {
    let d = tempfile::tempdir().unwrap();
    assert!(crate::bundler::BundleEngine::bundle(
        d.path(),
        &crate::bundler::BundleManifest::default()
    )
    .is_err());
    assert!(crate::package_pipeline::MultiOsPipeline::stage_packages(
        d.path(),
        &crate::package_pipeline::MultiOsPackageConfig::default(),
        &["macos".into()]
    )
    .is_err());
}
#[test]
fn secrets_authenticate_password_and_have_no_defaults() {
    let s = crate::secrets::SecretManager::new();
    assert!(s.list_secrets().is_empty());
    s.store_secret("k", "value", "passphrase123").unwrap();
    assert_eq!(s.retrieve_secret("k", "passphrase123").unwrap(), "value");
    assert!(s.retrieve_secret("k", "wrong").is_err());
}

#[test]
fn configured_read_scope_is_enforced() {
    let d = tempfile::tempdir().unwrap();
    let mut manifest = CapabilityManifest::default();
    manifest.fs.allow_read = vec!["./sandbox/permitted/**".into()];
    let s = SecurityEngine::new(manifest, d.path());
    let a = s.check_fs_write("permitted/a.txt").unwrap();
    std::fs::write(a, "a").unwrap();
    let b = s.check_fs_write("other.txt").unwrap();
    std::fs::write(b, "b").unwrap();
    assert!(s.check_fs_read("permitted/a.txt").is_ok());
    assert!(s.check_fs_read("other.txt").is_err());
}

#[test]
fn atomic_vault_backup_and_authenticated_overwrite() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("test.vault");
    let first = VaultEngine::encrypt("first", "passphrase123").unwrap();
    let second = VaultEngine::encrypt("second", "passphrase123").unwrap();
    crate::storage::atomic_vault_write(&p, first.as_bytes(), "passphrase123").unwrap();
    assert!(crate::storage::atomic_vault_write(&p, second.as_bytes(), "wrong").is_err());
    assert_eq!(std::fs::read_to_string(&p).unwrap(), first);
    crate::storage::atomic_vault_write(&p, second.as_bytes(), "passphrase123").unwrap();
    assert_eq!(
        std::fs::read_to_string(crate::storage::backup_path(&p)).unwrap(),
        first
    );
    assert_eq!(std::fs::read_to_string(&p).unwrap(), second);
    assert!(!std::fs::read_dir(d.path()).unwrap().any(|e| e
        .unwrap()
        .file_name()
        .to_string_lossy()
        .starts_with(".aegis-tmp-")));
}
#[test]
fn storage_lock_prevents_concurrent_writes() {
    let d = tempfile::tempdir().unwrap();
    let lock = std::fs::OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .truncate(false)
        .open(d.path().join(".aegis-write.lock"))
        .unwrap();
    lock.try_lock().unwrap();
    let p = d.path().join("file");
    assert!(crate::storage::atomic_write(&p, b"x", false).is_err());
    assert!(!p.exists());
    drop(lock);
    crate::storage::atomic_write(&p, b"x", false).unwrap();
}
#[test]
fn ipc_debug_output_is_redacted() {
    use crate::ipc::protocol::{IpcRequest, IpcResponse};
    let req = IpcRequest {
        id: "1".into(),
        command: "vault".into(),
        nonce: "secret_nonce".into(),
        payload: serde_json::json!({"password":"master_secret"}),
    };
    let debug = format!("{req:?}");
    assert!(!debug.contains("master_secret"));
    assert!(!debug.contains("secret_nonce"));
    let response = IpcResponse::ok("1", serde_json::json!({"plaintext":"secret_value"}));
    assert!(!format!("{response:?}").contains("secret_value"));
}
#[test]
fn reserved_names_are_not_storage_paths() {
    let d = tempfile::tempdir().unwrap();
    let s = SecurityEngine::new(CapabilityManifest::default(), d.path());
    for p in ["CON.txt", "NUL", "sub/COM1.log", ".aegis-write.lock"] {
        assert!(s.check_fs_write(p).is_err(), "{p}");
    }
}
#[test]
fn secrets_require_password_to_replace_and_clear() {
    let s = crate::secrets::SecretManager::new();
    s.store_secret("k", "first", "passphrase123").unwrap();
    assert!(s.store_secret("k", "second", "incorrectpass").is_err());
    assert_eq!(s.retrieve_secret("k", "passphrase123").unwrap(), "first");
    s.clear();
    assert!(s.retrieve_secret("k", "passphrase123").is_err());
}
#[test]
fn authentication_failure_limit_resets() {
    let l = RateLimiter::new(20, 60, 60);
    for _ in 0..5 {
        l.record_nonce_failure("sensitive");
    }
    assert!(l.is_locked_out("sensitive"));
    l.clear_nonce_failures("sensitive");
    assert!(!l.is_locked_out("sensitive"));
}
#[test]
fn bounded_storage_read_rejects_large_file() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("large");
    let f = std::fs::File::create(&p).unwrap();
    f.set_len(crate::storage::MAX_FILE_BYTES as u64 + 1)
        .unwrap();
    assert!(crate::storage::read_bounded(&p).is_err());
}

#[test]
fn legacy_v2_parameters_and_short_password_remain_readable() {
    use chacha20poly1305::{
        aead::{Aead, Payload},
        KeyInit, XChaCha20Poly1305, XNonce,
    };
    let password = "old-pass";
    let salt = [7u8; 16];
    let nonce = [9u8; 24];
    let mut key = [0u8; 32];
    argon2::Argon2::default()
        .hash_password_into(password.as_bytes(), &salt, &mut key)
        .unwrap();
    let data = XChaCha20Poly1305::new((&key).into())
        .encrypt(
            XNonce::from_slice(&nonce),
            Payload {
                msg: b"v0.19 fixture",
                aad: b"AEGIS_VAULT_V2",
            },
        )
        .unwrap();
    let envelope = format!(
        "AEGIS_VAULT_V2:{}:{}:{}",
        hex::encode(salt),
        hex::encode(nonce),
        hex::encode(data)
    );
    assert_eq!(
        VaultEngine::decrypt(&envelope, password).unwrap(),
        "v0.19 fixture"
    );
    assert!(VaultEngine::encrypt("new", password).is_err());
}

#[test]
fn failed_backup_replacement_preserves_active_vault() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("recovery.vault");
    let password = "recovery-test-password";
    let first = VaultEngine::encrypt("original", password).unwrap();
    let second = VaultEngine::encrypt("replacement", password).unwrap();
    crate::storage::atomic_vault_write(&path, first.as_bytes(), password).unwrap();
    // A non-file backup target forces failure before the active generation changes.
    std::fs::create_dir(crate::storage::backup_path(&path)).unwrap();
    assert!(crate::storage::atomic_vault_write(&path, second.as_bytes(), password).is_err());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), first);
    assert!(!std::fs::read_dir(d.path()).unwrap().any(|e| e
        .unwrap()
        .file_name()
        .to_string_lossy()
        .starts_with(".aegis-tmp-")));
}

#[test]
fn corrupted_active_vault_does_not_destroy_recovery_backup() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("recovery.vault");
    let password = "recovery-test-password";
    let first = VaultEngine::encrypt("original", password).unwrap();
    let second = VaultEngine::encrypt("replacement", password).unwrap();
    crate::storage::atomic_vault_write(&path, first.as_bytes(), password).unwrap();
    crate::storage::atomic_vault_write(&path, second.as_bytes(), password).unwrap();
    std::fs::write(&path, "AEGIS_VAULT_V2:truncated").unwrap();
    assert!(crate::storage::atomic_vault_write(&path, second.as_bytes(), password).is_err());
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "AEGIS_VAULT_V2:truncated"
    );
    let backup = crate::storage::read_bounded(&crate::storage::backup_path(&path)).unwrap();
    assert_eq!(backup, first);
    assert_eq!(VaultEngine::decrypt(&backup, password).unwrap(), "original");
}

#[test]
fn oversized_replacement_preserves_both_generations() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("recovery.vault");
    let password = "recovery-test-password";
    let first = VaultEngine::encrypt("original", password).unwrap();
    let second = VaultEngine::encrypt("replacement", password).unwrap();
    crate::storage::atomic_vault_write(&path, first.as_bytes(), password).unwrap();
    crate::storage::atomic_vault_write(&path, second.as_bytes(), password).unwrap();
    let oversized = vec![b'x'; crate::storage::MAX_FILE_BYTES + 1];
    assert!(crate::storage::atomic_vault_write(&path, &oversized, password).is_err());
    assert_eq!(crate::storage::read_bounded(&path).unwrap(), second);
    assert_eq!(
        crate::storage::read_bounded(&crate::storage::backup_path(&path)).unwrap(),
        first
    );
}

#[test]
fn vault_session_authenticates_and_lock_revokes_access() {
    let secrets = std::sync::Arc::new(crate::secrets::SecretManager::new());
    let sessions = super::vault_session::VaultSessions::new(secrets.clone());
    let envelope = VaultEngine::encrypt("session plaintext", "session-test-password").unwrap();
    assert!(sessions
        .unlock("one.vault", &envelope, "incorrect-password")
        .is_err());
    let token = sessions
        .unlock("one.vault", &envelope, "session-test-password")
        .unwrap();
    assert!(sessions.read("forged", "one.vault", &envelope).is_err());
    assert!(sessions.read(&token, "two.vault", &envelope).is_err());
    assert_eq!(
        &*sessions.read(&token, "one.vault", &envelope).unwrap(),
        "session plaintext"
    );
    secrets
        .store_secret("temporary", "sensitive", "session-test-password")
        .unwrap();
    sessions.lock();
    assert!(sessions.read(&token, "one.vault", &envelope).is_err());
    assert!(secrets.list_secrets().is_empty());
}

#[test]
fn vault_session_expiry_and_reunlock_revoke_tokens() {
    use std::{sync::Arc, time::Duration};
    let secret = Arc::new(crate::secrets::SecretManager::new());
    let envelope = VaultEngine::encrypt("expires", "session-test-password").unwrap();
    for (idle, absolute) in [
        (Duration::ZERO, Duration::from_secs(300)),
        (Duration::from_secs(60), Duration::ZERO),
    ] {
        let sessions =
            super::vault_session::VaultSessions::with_limits(secret.clone(), idle, absolute);
        let token = sessions
            .unlock("one.vault", &envelope, "session-test-password")
            .unwrap();
        assert!(sessions.read(&token, "one.vault", &envelope).is_err());
    }
    let sessions = super::vault_session::VaultSessions::new(secret);
    let old = sessions
        .unlock("one.vault", &envelope, "session-test-password")
        .unwrap();
    let new = sessions
        .unlock("one.vault", &envelope, "session-test-password")
        .unwrap();
    assert_ne!(&*old, &*new);
    assert!(sessions.read(&old, "one.vault", &envelope).is_err());
    assert!(sessions.read(&new, "one.vault", &envelope).is_ok());
}

#[test]
fn ipc_vault_sessions_reject_unauthorized_and_locked_requests() {
    use std::sync::Arc;
    let d = tempfile::tempdir().unwrap();
    let security = Arc::new(SecurityEngine::new(CapabilityManifest::default(), d.path()));
    let handshake = Arc::new(HandshakeSession::new());
    let plugins = crate::plugin::PluginRegistry::new();
    let router = crate::ipc::IpcRouter::new(
        security,
        handshake.clone(),
        plugins,
        Arc::new(crate::updater::AutoUpdater::new("test", d.path())),
        Arc::new(HardwareKeyEngine::new()),
        Arc::new(crate::profile::ProfileManager::new(d.path())),
        Arc::new(crate::wasm::WasmSandbox::new()),
        Arc::new(BiometricEnclave::new()),
        Arc::new(crate::crash::CrashReporter::new(d.path())),
        Arc::new(crate::secrets::SecretManager::new()),
        Arc::new(crate::network::NetworkPolicyEngine::new()),
        Arc::new(SignatureVerifier::new()),
        Arc::new(TpmAttestationEngine::new()),
        Arc::new(crate::package_pipeline::MultiOsPipeline),
        Arc::new(crate::p2p_market::P2PPluginMarketplace::new()),
        Arc::new(HsmPkcs11Engine::new()),
    );
    let call = |command: &str, payload: serde_json::Value| {
        router.handle_request(crate::ipc::protocol::IpcRequest {
            id: "session-test".into(),
            command: command.into(),
            nonce: handshake.get_token(),
            payload,
        })
    };
    let password = "ipc-session-test-password";
    assert!(
        call(
            "vault_encrypt_store",
            serde_json::json!({"filename":"one.vault","content":"PRIVATE","passphrase":password})
        )
        .success
    );
    assert!(
        !call(
            "vault_session_read",
            serde_json::json!({"filename":"one.vault","sessionToken":"forged"})
        )
        .success
    );
    assert!(
        !call(
            "vault_session_unlock",
            serde_json::json!({"filename":"one.vault","passphrase":"wrong-password"})
        )
        .success
    );
    let unlocked = call(
        "vault_session_unlock",
        serde_json::json!({"filename":"one.vault","passphrase":password}),
    );
    assert!(unlocked.success, "{:?}", unlocked.error);
    let token = unlocked.data.as_ref().unwrap()["sessionToken"]
        .as_str()
        .unwrap();
    assert!(
        !call(
            "vault_session_read",
            serde_json::json!({"filename":"two.vault","sessionToken":token})
        )
        .success
    );
    let read = call(
        "vault_session_read",
        serde_json::json!({"filename":"one.vault","sessionToken":token}),
    );
    assert_eq!(read.data.as_ref().unwrap()["decryptedContent"], "PRIVATE");
    assert!(call("vault_session_lock", serde_json::json!({})).success);
    assert!(
        !call(
            "vault_session_read",
            serde_json::json!({"filename":"one.vault","sessionToken":token})
        )
        .success
    );
    let reopened = call(
        "vault_session_unlock",
        serde_json::json!({"filename":"one.vault","passphrase":password}),
    );
    let token = reopened.data.as_ref().unwrap()["sessionToken"]
        .as_str()
        .unwrap();
    assert!(
        call(
            "profile_switch",
            serde_json::json!({"profileId":"private_vault"})
        )
        .success
    );
    assert!(call("profile_switch", serde_json::json!({"profileId":"default"})).success);
    assert!(
        !call(
            "vault_session_read",
            serde_json::json!({"filename":"one.vault","sessionToken":token})
        )
        .success
    );
    for _ in 0..25 {
        call(
            "vault_session_read",
            serde_json::json!({"filename":"one.vault","sessionToken":token}),
        );
    }
    assert!(
        call("vault_session_lock", serde_json::json!({})).success,
        "Lock must bypass the sensitive-operation budget"
    );
}

#[test]
fn expired_session_background_cleanup_clears_ephemeral_secrets() {
    use std::{
        sync::Arc,
        time::{Duration, Instant},
    };
    let secrets = Arc::new(crate::secrets::SecretManager::new());
    let sessions = super::vault_session::VaultSessions::with_limits(
        secrets.clone(),
        Duration::from_secs(3),
        Duration::from_secs(300),
    );
    let envelope = VaultEngine::encrypt("expires", "session-test-password").unwrap();
    let _token = sessions
        .unlock("one.vault", &envelope, "session-test-password")
        .unwrap();
    secrets
        .store_secret("temporary", "private", "session-test-password")
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(8);
    while !secrets.list_secrets().is_empty() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(
        secrets.list_secrets().is_empty(),
        "Cleanup must run without a new IPC request"
    );
}

#[test]
fn system_lock_revokes_session_and_requires_fresh_authentication() {
    use std::sync::Arc;
    let sessions =
        super::vault_session::VaultSessions::new(Arc::new(crate::secrets::SecretManager::new()));
    let envelope = VaultEngine::encrypt("PRIVATE", "platform-test-password").unwrap();
    let token = sessions
        .unlock("one.vault", &envelope, "platform-test-password")
        .unwrap();
    sessions.system_lock();
    assert!(sessions.read(&token, "one.vault", &envelope).is_err());
    let fresh = sessions
        .unlock("one.vault", &envelope, "platform-test-password")
        .unwrap();
    assert!(sessions.read(&fresh, "one.vault", &envelope).is_ok());
    sessions.require_platform_monitor();
    assert!(sessions.read(&fresh, "one.vault", &envelope).is_err());
    assert!(sessions
        .unlock("one.vault", &envelope, "platform-test-password")
        .is_err());
}

#[test]
fn unavailable_platform_read_clears_ephemeral_secrets() {
    use std::sync::Arc;
    let secrets = Arc::new(crate::secrets::SecretManager::new());
    let sessions = super::vault_session::VaultSessions::new(secrets.clone());
    sessions.require_platform_monitor();
    secrets
        .store_secret("temporary", "private", "platform-test-password")
        .unwrap();
    assert!(sessions.read("invalid", "one.vault", "invalid").is_err());
    assert!(secrets.list_secrets().is_empty(), "Unavailable OS session must clear secrets immediately, without waiting for monitor callbacks");
    secrets
        .store_secret("temporary", "private", "platform-test-password")
        .unwrap();
    assert!(sessions
        .unlock("one.vault", "invalid", "platform-test-password")
        .is_err());
    assert!(
        secrets.list_secrets().is_empty(),
        "Denied unlock must also clear ephemeral secrets"
    );
}

#[test]
fn lifecycle_observer_revokes_access_without_granting_platform_authority() {
    use super::lifecycle::LifecycleObserver;
    use std::sync::Arc;
    let secrets = Arc::new(crate::secrets::SecretManager::new());
    let sessions = Arc::new(super::vault_session::VaultSessions::new(secrets.clone()));
    let envelope = VaultEngine::encrypt("PRIVATE", "platform-test-password").unwrap();
    let token = sessions
        .unlock("one.vault", &envelope, "platform-test-password")
        .unwrap();
    let observer = LifecycleObserver::new(Arc::downgrade(&sessions));
    secrets
        .store_secret("temporary", "private", "platform-test-password")
        .unwrap();
    observer.revoke();
    assert!(sessions.read(&token, "one.vault", &envelope).is_err());
    assert!(secrets.list_secrets().is_empty());
    assert!(
        !sessions.platform_monitor_active(),
        "A revoke-only observer must not authorize retained sessions"
    );
}

#[test]
fn ipc_profiles_reject_shared_storage_identities() {
    use std::sync::Arc;
    let d = tempfile::tempdir().unwrap();
    let security = Arc::new(SecurityEngine::new(CapabilityManifest::default(), d.path()));
    let handshake = Arc::new(HandshakeSession::new());
    let plugins = crate::plugin::PluginRegistry::new();
    let router = crate::ipc::IpcRouter::new(
        security,
        handshake.clone(),
        plugins,
        Arc::new(crate::updater::AutoUpdater::new("test", d.path())),
        Arc::new(HardwareKeyEngine::new()),
        Arc::new(crate::profile::ProfileManager::new(d.path())),
        Arc::new(crate::wasm::WasmSandbox::new()),
        Arc::new(BiometricEnclave::new()),
        Arc::new(crate::crash::CrashReporter::new(d.path())),
        Arc::new(crate::secrets::SecretManager::new()),
        Arc::new(crate::network::NetworkPolicyEngine::new()),
        Arc::new(SignatureVerifier::new()),
        Arc::new(TpmAttestationEngine::new()),
        Arc::new(crate::package_pipeline::MultiOsPipeline),
        Arc::new(crate::p2p_market::P2PPluginMarketplace::new()),
        Arc::new(HsmPkcs11Engine::new()),
    );
    let call = |command: &str, payload: serde_json::Value| {
        router.handle_request(crate::ipc::protocol::IpcRequest {
            id: "session-test".into(),
            command: command.into(),
            nonce: handshake.get_token(),
            payload,
        })
    };
    assert!(
        call(
            "profile_switch",
            serde_json::json!({"profileId":"private_vault"})
        )
        .success
    );
    assert!(
        call(
            "fs_write_file",
            serde_json::json!({"path":"private.txt","content":"private profile data"})
        )
        .success
    );
    std::fs::write(d.path().join("sandbox_blocked"), b"TEST_ONLY blocker").unwrap();
    assert!(
        !call("profile_create", serde_json::json!({"profileId":"blocked"})).success,
        "A failed profile directory creation must not report successful registration"
    );
    let listed = call("profile_list", serde_json::json!({}));
    assert!(!listed.data.as_ref().unwrap()["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .any(|profile| profile["id"] == "blocked"));
    let collision = call("profile_create", serde_json::json!({"profileId":"vault"}));
    assert!(
        !collision.success,
        "A custom profile must not alias the built-in private vault directory"
    );
    assert!(call("profile_create", serde_json::json!({"profileId":"Work"})).success);
    assert!(
        !call("profile_create", serde_json::json!({"profileId":"work"})).success,
        "Profile identities must remain distinct on case-insensitive filesystems"
    );
    std::fs::remove_dir(d.path().join("sandbox_Work")).unwrap();
    std::fs::write(
        d.path().join("sandbox_Work"),
        b"TEST_ONLY blocked destination",
    )
    .unwrap();
    assert!(
        !call("profile_switch", serde_json::json!({"profileId":"Work"})).success,
        "An unusable profile directory must not become active"
    );
    let listed = call("profile_list", serde_json::json!({}));
    assert_eq!(
        listed.data.as_ref().unwrap()["activeProfile"]["id"],
        "private_vault"
    );
    assert!(
        call("fs_read_file", serde_json::json!({"path":"private.txt"})).success,
        "Failed switching must preserve the prior storage root"
    );
    std::fs::remove_file(d.path().join("sandbox_blocked")).unwrap();
    assert!(
        call("profile_create", serde_json::json!({"profileId":"blocked"})).success,
        "A failed creation must not reserve its profile ID"
    );
    std::fs::remove_file(d.path().join("sandbox_Work")).unwrap();
    assert!(
        call("profile_switch", serde_json::json!({"profileId":"Work"})).success,
        "Switching must recover once the destination is usable"
    );
    assert!(!call("fs_read_file", serde_json::json!({"path":"private.txt"})).success);
}

#[test]
fn ipc_restart_ignores_colliding_saved_profiles() {
    use std::sync::Arc;
    let d = tempfile::tempdir().unwrap();
    let saved = serde_json::json!([
        {"id":"vault","name":"Legacy collision","description":"","sandbox_dir":"sandbox_vault","created_at":0,"strict_isolation":true,"max_storage_bytes":1000},
        {"id":"Work","name":"Work","description":"","sandbox_dir":"sandbox_Work","created_at":0,"strict_isolation":true,"max_storage_bytes":1000},
        {"id":"work","name":"Alias","description":"","sandbox_dir":"sandbox_work","created_at":0,"strict_isolation":true,"max_storage_bytes":1000}
    ]);
    std::fs::write(
        d.path().join("profiles.json"),
        serde_json::to_vec(&saved).unwrap(),
    )
    .unwrap();
    let security = Arc::new(SecurityEngine::new(CapabilityManifest::default(), d.path()));
    let handshake = Arc::new(HandshakeSession::new());
    let plugins = crate::plugin::PluginRegistry::new();
    let router = crate::ipc::IpcRouter::new(
        security,
        handshake.clone(),
        plugins,
        Arc::new(crate::updater::AutoUpdater::new("test", d.path())),
        Arc::new(HardwareKeyEngine::new()),
        Arc::new(crate::profile::ProfileManager::new(d.path())),
        Arc::new(crate::wasm::WasmSandbox::new()),
        Arc::new(BiometricEnclave::new()),
        Arc::new(crate::crash::CrashReporter::new(d.path())),
        Arc::new(crate::secrets::SecretManager::new()),
        Arc::new(crate::network::NetworkPolicyEngine::new()),
        Arc::new(SignatureVerifier::new()),
        Arc::new(TpmAttestationEngine::new()),
        Arc::new(crate::package_pipeline::MultiOsPipeline),
        Arc::new(crate::p2p_market::P2PPluginMarketplace::new()),
        Arc::new(HsmPkcs11Engine::new()),
    );
    let call = |command: &str, payload: serde_json::Value| {
        router.handle_request(crate::ipc::protocol::IpcRequest {
            id: "session-test".into(),
            command: command.into(),
            nonce: handshake.get_token(),
            payload,
        })
    };
    let listed = call("profile_list", serde_json::json!({}));
    assert!(listed.success);
    let profiles = listed.data.as_ref().unwrap()["profiles"]
        .as_array()
        .unwrap();
    assert_eq!(
        profiles.len(),
        3,
        "Two built-in profiles and one distinct custom profile must survive"
    );
    assert!(!call("profile_switch", serde_json::json!({"profileId":"vault"})).success);
    assert!(
        call(
            "profile_switch",
            serde_json::json!({"profileId":"private_vault"})
        )
        .success
    );
    assert!(call("profile_switch", serde_json::json!({"profileId":"Work"})).success);
}

#[test]
fn ipc_window_capabilities_do_not_advertise_unimplemented_controls() {
    use std::sync::Arc;
    let d = tempfile::tempdir().unwrap();
    let security = Arc::new(SecurityEngine::new(CapabilityManifest::default(), d.path()));
    let handshake = Arc::new(HandshakeSession::new());
    let plugins = crate::plugin::PluginRegistry::new();
    let router = crate::ipc::IpcRouter::new(
        security,
        handshake.clone(),
        plugins,
        Arc::new(crate::updater::AutoUpdater::new("test", d.path())),
        Arc::new(HardwareKeyEngine::new()),
        Arc::new(crate::profile::ProfileManager::new(d.path())),
        Arc::new(crate::wasm::WasmSandbox::new()),
        Arc::new(BiometricEnclave::new()),
        Arc::new(crate::crash::CrashReporter::new(d.path())),
        Arc::new(crate::secrets::SecretManager::new()),
        Arc::new(crate::network::NetworkPolicyEngine::new()),
        Arc::new(SignatureVerifier::new()),
        Arc::new(TpmAttestationEngine::new()),
        Arc::new(crate::package_pipeline::MultiOsPipeline),
        Arc::new(crate::p2p_market::P2PPluginMarketplace::new()),
        Arc::new(HsmPkcs11Engine::new()),
    );
    let call = |command: &str, payload: serde_json::Value| {
        router.handle_request(crate::ipc::protocol::IpcRequest {
            id: "session-test".into(),
            command: command.into(),
            nonce: handshake.get_token(),
            payload,
        })
    };
    let response = call("window_list", serde_json::json!({}));
    assert!(response.success);
    let capabilities = &response.data.as_ref().unwrap()["capabilities"];
    for unsupported in ["modal", "kiosk", "alwaysOnTop", "customSize"] {
        assert_eq!(
            capabilities[unsupported], false,
            "{unsupported} is not implemented by the external-browser backend"
        );
    }
    // This remote URL is always rejected, so the regression cannot open a desktop
    // even when a broken parser silently accepts the malformed options.
    for options in [
        serde_json::json!({"width":4294968096u64}),
        serde_json::json!({"height":"600"}),
        serde_json::json!({"modal":"true"}),
        serde_json::json!({"title":null}),
        serde_json::json!({"unexpectedOption":true}),
    ] {
        let mut options = options;
        options["url"] = serde_json::json!("https://blocked.invalid/");
        let result = call("window_spawn", options);
        assert!(!result.success);
        assert!(result.error.as_ref().unwrap().contains("Invalid window options"),
            "Malformed options must be rejected before launch validation, not truncated or defaulted: {:?}", result.error);
    }
}

#[test]
fn ipc_dialog_permission_denial_precedes_native_helper_work() {
    use std::sync::Arc;
    let d = tempfile::tempdir().unwrap();
    let mut manifest = CapabilityManifest::default();
    manifest.system.allow_dialogs = false;
    let security = Arc::new(SecurityEngine::new(manifest, d.path()));
    let handshake = Arc::new(HandshakeSession::new());
    let plugins = crate::plugin::PluginRegistry::new();
    let router = crate::ipc::IpcRouter::new(
        security,
        handshake.clone(),
        plugins,
        Arc::new(crate::updater::AutoUpdater::new("test", d.path())),
        Arc::new(HardwareKeyEngine::new()),
        Arc::new(crate::profile::ProfileManager::new(d.path())),
        Arc::new(crate::wasm::WasmSandbox::new()),
        Arc::new(BiometricEnclave::new()),
        Arc::new(crate::crash::CrashReporter::new(d.path())),
        Arc::new(crate::secrets::SecretManager::new()),
        Arc::new(crate::network::NetworkPolicyEngine::new()),
        Arc::new(SignatureVerifier::new()),
        Arc::new(TpmAttestationEngine::new()),
        Arc::new(crate::package_pipeline::MultiOsPipeline),
        Arc::new(crate::p2p_market::P2PPluginMarketplace::new()),
        Arc::new(HsmPkcs11Engine::new()),
    );
    let call = |command: &str, payload: serde_json::Value| {
        router.handle_request(crate::ipc::protocol::IpcRequest {
            id: "session-test".into(),
            command: command.into(),
            nonce: handshake.get_token(),
            payload,
        })
    };
    let response = call(
        "window_dialog",
        serde_json::json!({"title":"x".repeat(1025),"message":"fixture","type":"info"}),
    );
    assert!(!response.success);
    assert!(response
        .error
        .as_deref()
        .unwrap()
        .contains("'dialogs' capability is disabled"));
}

#[test]
fn ipc_clipboard_is_denied_by_default_before_provider_access() {
    use std::sync::Arc;
    let d = tempfile::tempdir().unwrap();
    let manifest = CapabilityManifest::default();

    let security = Arc::new(SecurityEngine::new(manifest, d.path()));
    let handshake = Arc::new(HandshakeSession::new());
    let plugins = crate::plugin::PluginRegistry::new();
    let router = crate::ipc::IpcRouter::new(
        security,
        handshake.clone(),
        plugins,
        Arc::new(crate::updater::AutoUpdater::new("test", d.path())),
        Arc::new(HardwareKeyEngine::new()),
        Arc::new(crate::profile::ProfileManager::new(d.path())),
        Arc::new(crate::wasm::WasmSandbox::new()),
        Arc::new(BiometricEnclave::new()),
        Arc::new(crate::crash::CrashReporter::new(d.path())),
        Arc::new(crate::secrets::SecretManager::new()),
        Arc::new(crate::network::NetworkPolicyEngine::new()),
        Arc::new(SignatureVerifier::new()),
        Arc::new(TpmAttestationEngine::new()),
        Arc::new(crate::package_pipeline::MultiOsPipeline),
        Arc::new(crate::p2p_market::P2PPluginMarketplace::new()),
        Arc::new(HsmPkcs11Engine::new()),
    );
    let call = |command: &str, payload: serde_json::Value| {
        router.handle_request(crate::ipc::protocol::IpcRequest {
            id: "session-test".into(),
            command: command.into(),
            nonce: handshake.get_token(),
            payload,
        })
    };
    let response = call(
        "clipboard_write_text",
        serde_json::json!({"text":"x".repeat(65537)}),
    );
    assert!(!response.success);
    assert!(response
        .error
        .as_deref()
        .unwrap()
        .contains("'clipboard_write' capability is disabled"));
}

#[test]
fn ipc_clipboard_rejects_oversized_text_before_provider_access() {
    use std::sync::Arc;
    let d = tempfile::tempdir().unwrap();
    let mut manifest = CapabilityManifest::default();
    manifest.system.allow_clipboard_write = true;

    let security = Arc::new(SecurityEngine::new(manifest, d.path()));
    let handshake = Arc::new(HandshakeSession::new());
    let plugins = crate::plugin::PluginRegistry::new();
    let router = crate::ipc::IpcRouter::new(
        security,
        handshake.clone(),
        plugins,
        Arc::new(crate::updater::AutoUpdater::new("test", d.path())),
        Arc::new(HardwareKeyEngine::new()),
        Arc::new(crate::profile::ProfileManager::new(d.path())),
        Arc::new(crate::wasm::WasmSandbox::new()),
        Arc::new(BiometricEnclave::new()),
        Arc::new(crate::crash::CrashReporter::new(d.path())),
        Arc::new(crate::secrets::SecretManager::new()),
        Arc::new(crate::network::NetworkPolicyEngine::new()),
        Arc::new(SignatureVerifier::new()),
        Arc::new(TpmAttestationEngine::new()),
        Arc::new(crate::package_pipeline::MultiOsPipeline),
        Arc::new(crate::p2p_market::P2PPluginMarketplace::new()),
        Arc::new(HsmPkcs11Engine::new()),
    );
    let call = |command: &str, payload: serde_json::Value| {
        router.handle_request(crate::ipc::protocol::IpcRequest {
            id: "session-test".into(),
            command: command.into(),
            nonce: handshake.get_token(),
            payload,
        })
    };
    let response = call(
        "clipboard_write_text",
        serde_json::json!({"text":"x".repeat(65537)}),
    );
    assert!(!response.success);
    assert!(response
        .error
        .as_deref()
        .unwrap()
        .contains("Clipboard text exceeds 65536 bytes"));
}

#[test]
#[ignore = "writes clipboard only on disposable hosted CI"]
fn ipc_clipboard_roundtrip_on_disposable_runner() {
    assert_eq!(std::env::var("GITHUB_ACTIONS").as_deref(), Ok("true"));
    assert_eq!(
        std::env::var("RUNNER_ENVIRONMENT").as_deref(),
        Ok("github-hosted")
    );
    use std::sync::Arc;
    let d = tempfile::tempdir().unwrap();
    let mut manifest = CapabilityManifest::default();
    manifest.system.allow_clipboard_write = true;

    let security = Arc::new(SecurityEngine::new(manifest, d.path()));
    let handshake = Arc::new(HandshakeSession::new());
    let plugins = crate::plugin::PluginRegistry::new();
    let router = crate::ipc::IpcRouter::new(
        security,
        handshake.clone(),
        plugins,
        Arc::new(crate::updater::AutoUpdater::new("test", d.path())),
        Arc::new(HardwareKeyEngine::new()),
        Arc::new(crate::profile::ProfileManager::new(d.path())),
        Arc::new(crate::wasm::WasmSandbox::new()),
        Arc::new(BiometricEnclave::new()),
        Arc::new(crate::crash::CrashReporter::new(d.path())),
        Arc::new(crate::secrets::SecretManager::new()),
        Arc::new(crate::network::NetworkPolicyEngine::new()),
        Arc::new(SignatureVerifier::new()),
        Arc::new(TpmAttestationEngine::new()),
        Arc::new(crate::package_pipeline::MultiOsPipeline),
        Arc::new(crate::p2p_market::P2PPluginMarketplace::new()),
        Arc::new(HsmPkcs11Engine::new()),
    );
    let call = |command: &str, payload: serde_json::Value| {
        router.handle_request(crate::ipc::protocol::IpcRequest {
            id: "session-test".into(),
            command: command.into(),
            nonce: handshake.get_token(),
            payload,
        })
    };
    let response = call(
        "clipboard_write_text",
        serde_json::json!({"text":"Aegis clipboard fixture \u{0d85} \u{1f512}\nline two"}),
    );
    assert!(response.success, "{:?}", response.error);
    assert_eq!(response.data.as_ref().unwrap()["written"], true);
    let mut observer = arboard::Clipboard::new().unwrap();
    assert_eq!(
        observer.get_text().unwrap(),
        "Aegis clipboard fixture \u{0d85} \u{1f512}\nline two"
    );
    observer.clear().unwrap();
}

#[test]
fn clipboard_rejects_nul_without_accessing_the_os() {
    assert_eq!(
        crate::clipboard::TextClipboard::validate_text("a\0b").unwrap_err(),
        "Clipboard text contains NUL"
    );
}

#[test]
fn ipc_file_dialog_is_denied_by_default_before_provider_access() {
    use std::sync::Arc;
    let d = tempfile::tempdir().unwrap();
    let manifest = CapabilityManifest::default();

    let security = Arc::new(SecurityEngine::new(manifest, d.path()));
    let handshake = Arc::new(HandshakeSession::new());
    let plugins = crate::plugin::PluginRegistry::new();
    let router = crate::ipc::IpcRouter::new(
        security,
        handshake.clone(),
        plugins,
        Arc::new(crate::updater::AutoUpdater::new("test", d.path())),
        Arc::new(HardwareKeyEngine::new()),
        Arc::new(crate::profile::ProfileManager::new(d.path())),
        Arc::new(crate::wasm::WasmSandbox::new()),
        Arc::new(BiometricEnclave::new()),
        Arc::new(crate::crash::CrashReporter::new(d.path())),
        Arc::new(crate::secrets::SecretManager::new()),
        Arc::new(crate::network::NetworkPolicyEngine::new()),
        Arc::new(SignatureVerifier::new()),
        Arc::new(TpmAttestationEngine::new()),
        Arc::new(crate::package_pipeline::MultiOsPipeline),
        Arc::new(crate::p2p_market::P2PPluginMarketplace::new()),
        Arc::new(HsmPkcs11Engine::new()),
    );
    let call = |command: &str, payload: serde_json::Value| {
        router.handle_request(crate::ipc::protocol::IpcRequest {
            id: "session-test".into(),
            command: command.into(),
            nonce: handshake.get_token(),
            payload,
        })
    };
    let response = call(
        "file_dialog_open",
        serde_json::json!({"title":"x".repeat(1025)}),
    );
    assert!(!response.success);
    assert!(response
        .error
        .as_deref()
        .unwrap()
        .contains("'file_dialogs' capability is disabled"));
}

#[test]
fn ipc_notifications_are_denied_by_default_before_provider_access() {
    use std::sync::Arc;
    let d = tempfile::tempdir().unwrap();
    let manifest = CapabilityManifest::default();

    let security = Arc::new(SecurityEngine::new(manifest, d.path()));
    let handshake = Arc::new(HandshakeSession::new());
    let plugins = crate::plugin::PluginRegistry::new();
    let router = crate::ipc::IpcRouter::new(
        security,
        handshake.clone(),
        plugins,
        Arc::new(crate::updater::AutoUpdater::new("test", d.path())),
        Arc::new(HardwareKeyEngine::new()),
        Arc::new(crate::profile::ProfileManager::new(d.path())),
        Arc::new(crate::wasm::WasmSandbox::new()),
        Arc::new(BiometricEnclave::new()),
        Arc::new(crate::crash::CrashReporter::new(d.path())),
        Arc::new(crate::secrets::SecretManager::new()),
        Arc::new(crate::network::NetworkPolicyEngine::new()),
        Arc::new(SignatureVerifier::new()),
        Arc::new(TpmAttestationEngine::new()),
        Arc::new(crate::package_pipeline::MultiOsPipeline),
        Arc::new(crate::p2p_market::P2PPluginMarketplace::new()),
        Arc::new(HsmPkcs11Engine::new()),
    );
    let call = |command: &str, payload: serde_json::Value| {
        router.handle_request(crate::ipc::protocol::IpcRequest {
            id: "session-test".into(),
            command: command.into(),
            nonce: handshake.get_token(),
            payload,
        })
    };
    let response = call(
        "notify_send",
        serde_json::json!({"title":"x".repeat(1025),"body":"fixture"}),
    );
    assert!(!response.success);
    assert!(response
        .error
        .as_deref()
        .unwrap()
        .contains("'notifications' capability is disabled"));
}

#[test]
fn ipc_notification_submission_quota_is_runtime_wide() {
    use std::sync::Arc;
    let d = tempfile::tempdir().unwrap();
    let mut manifest = CapabilityManifest::default();
    manifest.system.allow_notifications = true;

    let security = Arc::new(SecurityEngine::new(manifest, d.path()));
    let handshake = Arc::new(HandshakeSession::new());
    let plugins = crate::plugin::PluginRegistry::new();
    let router = crate::ipc::IpcRouter::new(
        security,
        handshake.clone(),
        plugins,
        Arc::new(crate::updater::AutoUpdater::new("test", d.path())),
        Arc::new(HardwareKeyEngine::new()),
        Arc::new(crate::profile::ProfileManager::new(d.path())),
        Arc::new(crate::wasm::WasmSandbox::new()),
        Arc::new(BiometricEnclave::new()),
        Arc::new(crate::crash::CrashReporter::new(d.path())),
        Arc::new(crate::secrets::SecretManager::new()),
        Arc::new(crate::network::NetworkPolicyEngine::new()),
        Arc::new(SignatureVerifier::new()),
        Arc::new(TpmAttestationEngine::new()),
        Arc::new(crate::package_pipeline::MultiOsPipeline),
        Arc::new(crate::p2p_market::P2PPluginMarketplace::new()),
        Arc::new(HsmPkcs11Engine::new()),
    );
    let call = |command: &str, payload: serde_json::Value| {
        router.handle_request(crate::ipc::protocol::IpcRequest {
            id: "session-test".into(),
            command: command.into(),
            nonce: handshake.get_token(),
            payload,
        })
    };
    for _ in 0..30 {
        let response = call(
            "notify_send",
            serde_json::json!({"title":"x".repeat(1025),"body":"fixture"}),
        );
        assert!(response
            .error
            .as_deref()
            .unwrap()
            .contains("Notification text exceeds limits"));
    }
    assert!(call("profile_switch", serde_json::json!({"profileId":"default"})).success);
    let response = call(
        "notify_send",
        serde_json::json!({"title":"x".repeat(1025),"body":"fixture"}),
    );
    assert!(response
        .error
        .as_deref()
        .unwrap()
        .contains("Notification submission limit reached"));
}

#[test]
fn ipc_plugin_dispatch_enforces_declared_capabilities() {
    use std::sync::Arc;
    let d = tempfile::tempdir().unwrap();
    let mut manifest = CapabilityManifest::default();
    manifest.system.allow_system_info = false;
    let security = Arc::new(SecurityEngine::new(manifest, d.path()));
    let handshake = Arc::new(HandshakeSession::new());
    let plugins = crate::plugin::PluginRegistry::new();
    crate::plugin::register_builtin_plugins(&plugins);
    let router = crate::ipc::IpcRouter::new(
        security,
        handshake.clone(),
        plugins,
        Arc::new(crate::updater::AutoUpdater::new("test", d.path())),
        Arc::new(HardwareKeyEngine::new()),
        Arc::new(crate::profile::ProfileManager::new(d.path())),
        Arc::new(crate::wasm::WasmSandbox::new()),
        Arc::new(BiometricEnclave::new()),
        Arc::new(crate::crash::CrashReporter::new(d.path())),
        Arc::new(crate::secrets::SecretManager::new()),
        Arc::new(crate::network::NetworkPolicyEngine::new()),
        Arc::new(SignatureVerifier::new()),
        Arc::new(TpmAttestationEngine::new()),
        Arc::new(crate::package_pipeline::MultiOsPipeline),
        Arc::new(crate::p2p_market::P2PPluginMarketplace::new()),
        Arc::new(HsmPkcs11Engine::new()),
    );
    let response = router.handle_request(crate::ipc::protocol::IpcRequest {
        id: "plugin-permission".into(),
        command: "plugin_dispatch".into(),
        nonce: handshake.get_token(),
        payload: serde_json::json!({"plugin":"process_uptime","payload":{}}),
    });
    assert!(
        !response.success,
        "Plugin descriptors must enforce system_info permission before handler execution"
    );
}

#[test]
fn checksum_plugin_rejects_outside_files_and_uses_active_profile() {
    let d = tempfile::tempdir().unwrap();
    let security = SecurityEngine::new(CapabilityManifest::default(), d.path());
    let registry = crate::plugin::PluginRegistry::new();
    crate::plugin::register_builtin_plugins(&registry);
    let outside = d.path().join("outside.txt");
    std::fs::write(&outside, "private outside data").unwrap();
    assert!(
        registry
            .dispatch(
                "file_checksum",
                &serde_json::json!({"filename":outside.to_str().unwrap()}),
                &security
            )
            .is_err(),
        "Checksum handler must not read an absolute file outside the active sandbox"
    );
    let profile = d.path().join("another-profile");
    security.set_sandbox_root(profile.clone());
    std::fs::write(profile.join("data.bin"), b"abc").unwrap();
    let result = registry
        .dispatch(
            "file_checksum",
            &serde_json::json!({"filename":"data.bin"}),
            &security,
        )
        .unwrap();
    assert_eq!(
        result["checksum"],
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(result["sizeBytes"], 3);
    let large = std::fs::File::create(profile.join("oversized.bin")).unwrap();
    large
        .set_len(crate::storage::MAX_FILE_BYTES as u64 + 1)
        .unwrap();
    assert!(registry
        .dispatch(
            "file_checksum",
            &serde_json::json!({"filename":"oversized.bin"}),
            &security
        )
        .is_err());
}

#[test]
fn plugins_do_not_report_fabricated_memory_statistics() {
    let d = tempfile::tempdir().unwrap();
    let security = SecurityEngine::new(CapabilityManifest::default(), d.path());
    let registry = crate::plugin::PluginRegistry::new();
    crate::plugin::register_builtin_plugins(&registry);
    assert!(
        registry
            .dispatch("system_memory", &serde_json::json!({}), &security)
            .is_err(),
        "No provider must return unavailable rather than hard-coded memory figures"
    );
}

#[test]
fn checksum_plugin_requires_read_and_hashing_grants() {
    let d = tempfile::tempdir().unwrap();
    let registry = crate::plugin::PluginRegistry::new();
    crate::plugin::register_builtin_plugins(&registry);
    for deny_read in [true, false] {
        let mut manifest = CapabilityManifest::default();
        if deny_read {
            manifest.fs.allow_read.clear();
        } else {
            manifest.crypto.allow_hashing = false;
        }
        let security = SecurityEngine::new(manifest, d.path());
        std::fs::write(security.get_sandbox_root().join("data.bin"), b"abc").unwrap();
        assert!(registry
            .dispatch(
                "file_checksum",
                &serde_json::json!({"filename":"data.bin"}),
                &security
            )
            .is_err());
    }
}

#[test]
fn uptime_plugin_reports_elapsed_time_instead_of_wall_clock() {
    let d = tempfile::tempdir().unwrap();
    let security = SecurityEngine::new(CapabilityManifest::default(), d.path());
    let registry = crate::plugin::PluginRegistry::new();
    crate::plugin::register_builtin_plugins(&registry);
    let value = registry
        .dispatch("process_uptime", &serde_json::json!({}), &security)
        .unwrap();
    let elapsed = value["uptimeSeconds"]
        .as_f64()
        .expect("Plugin must expose elapsed seconds");
    assert!((0.0..60.0).contains(&elapsed));
    assert_eq!(value["measurementScope"], "sinceBuiltinPluginRegistration");
}

#[test]
fn ipc_plugin_panic_revokes_sessions_and_preserves_other_commands() {
    use std::sync::Arc;
    let d = tempfile::tempdir().unwrap();
    let security = Arc::new(SecurityEngine::new(CapabilityManifest::default(), d.path()));
    let handshake = Arc::new(HandshakeSession::new());
    let secrets = Arc::new(crate::secrets::SecretManager::new());
    let plugins = crate::plugin::PluginRegistry::new();
    let first = std::sync::atomic::AtomicBool::new(true);
    plugins.register(
        crate::plugin::PluginDescriptor {
            name: "fault_fixture".into(),
            version: "1".into(),
            description: "Test only".into(),
            author: "Test".into(),
            capabilities: vec![],
        },
        move |_, _| {
            if first.swap(false, std::sync::atomic::Ordering::SeqCst) {
                panic!("TEST_ONLY panic payload");
            }
            Ok(serde_json::json!({"unexpectedRetry":true}))
        },
    );
    let router = crate::ipc::IpcRouter::new(
        security,
        handshake.clone(),
        plugins,
        Arc::new(crate::updater::AutoUpdater::new("test", d.path())),
        Arc::new(HardwareKeyEngine::new()),
        Arc::new(crate::profile::ProfileManager::new(d.path())),
        Arc::new(crate::wasm::WasmSandbox::new()),
        Arc::new(BiometricEnclave::new()),
        Arc::new(crate::crash::CrashReporter::new(d.path())),
        secrets.clone(),
        Arc::new(crate::network::NetworkPolicyEngine::new()),
        Arc::new(SignatureVerifier::new()),
        Arc::new(TpmAttestationEngine::new()),
        Arc::new(crate::package_pipeline::MultiOsPipeline),
        Arc::new(crate::p2p_market::P2PPluginMarketplace::new()),
        Arc::new(HsmPkcs11Engine::new()),
    );
    let call = |command: &str, payload| {
        router.handle_request(crate::ipc::protocol::IpcRequest {
            id: "fault-test".into(),
            command: command.into(),
            nonce: handshake.get_token(),
            payload,
        })
    };
    let password = "plugin-panic-test-password";
    assert!(
        call(
            "vault_encrypt_store",
            serde_json::json!({"filename":"one.vault","content":"PRIVATE","passphrase":password})
        )
        .success
    );
    let unlocked = call(
        "vault_session_unlock",
        serde_json::json!({"filename":"one.vault","passphrase":password}),
    );
    assert!(unlocked.success);
    let token = unlocked.data.as_ref().unwrap()["sessionToken"]
        .as_str()
        .unwrap();
    secrets
        .store_secret("temporary", "PRIVATE", password)
        .unwrap();
    let response = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        call(
            "plugin_dispatch",
            serde_json::json!({"plugin":"fault_fixture"}),
        )
    }));
    assert!(
        response.is_ok(),
        "A plugin unwind must not escape the IPC router"
    );
    let response = response.unwrap();
    assert!(!response.success);
    assert!(!response.error.as_deref().unwrap().contains("TEST_ONLY"));
    assert!(secrets.list_secrets().is_empty());
    assert!(
        !call(
            "vault_session_read",
            serde_json::json!({"filename":"one.vault","sessionToken":token})
        )
        .success
    );
    assert!(
        call("sys_info", serde_json::json!({})).success,
        "Other commands must survive a plugin fault"
    );
    assert!(
        !call(
            "plugin_dispatch",
            serde_json::json!({"plugin":"fault_fixture"})
        )
        .success,
        "Faulted handler must not be retried"
    );
    let listed = call("plugin_list", serde_json::json!({}));
    assert_eq!(
        listed.data.as_ref().unwrap()["disabledPlugins"],
        serde_json::json!(["fault_fixture"])
    );
}

#[test]
fn plugin_handler_can_register_another_plugin_without_deadlock() {
    use std::sync::Arc;
    let d = tempfile::tempdir().unwrap();
    let security = Arc::new(SecurityEngine::new(CapabilityManifest::default(), d.path()));
    let registry = crate::plugin::PluginRegistry::new();
    let weak = Arc::downgrade(&registry);
    let descriptor = |name: &str| crate::plugin::PluginDescriptor {
        name: name.into(),
        version: "1".into(),
        description: "Fixture".into(),
        author: "Test".into(),
        capabilities: vec![],
    };
    registry.register(descriptor("registering"), move |_, _| {
        weak.upgrade()
            .unwrap()
            .register(descriptor("new_fixture"), |_, _| Ok(serde_json::json!(42)));
        Ok(serde_json::json!("registered"))
    });
    let (tx, rx) = std::sync::mpsc::channel();
    let worker_registry = registry.clone();
    let worker_security = security.clone();
    let worker = std::thread::spawn(move || {
        tx.send(worker_registry.dispatch("registering", &serde_json::json!({}), &worker_security))
            .unwrap();
    });
    let result = rx
        .recv_timeout(std::time::Duration::from_secs(3))
        .expect("Plugin registration must not deadlock inside a handler");
    assert_eq!(result.unwrap(), "registered");
    worker.join().unwrap();
    assert_eq!(
        registry
            .dispatch("new_fixture", &serde_json::json!({}), &security)
            .unwrap(),
        42
    );
}

#[test]
fn ipc_wasm_rejects_unknown_or_malformed_execution_tiers() {
    use std::sync::Arc;
    let d = tempfile::tempdir().unwrap();
    let security = Arc::new(SecurityEngine::new(CapabilityManifest::default(), d.path()));
    let handshake = Arc::new(HandshakeSession::new());
    let plugins = crate::plugin::PluginRegistry::new();
    let router = crate::ipc::IpcRouter::new(
        security,
        handshake.clone(),
        plugins,
        Arc::new(crate::updater::AutoUpdater::new("test", d.path())),
        Arc::new(HardwareKeyEngine::new()),
        Arc::new(crate::profile::ProfileManager::new(d.path())),
        Arc::new(crate::wasm::WasmSandbox::new()),
        Arc::new(BiometricEnclave::new()),
        Arc::new(crate::crash::CrashReporter::new(d.path())),
        Arc::new(crate::secrets::SecretManager::new()),
        Arc::new(crate::network::NetworkPolicyEngine::new()),
        Arc::new(SignatureVerifier::new()),
        Arc::new(TpmAttestationEngine::new()),
        Arc::new(crate::package_pipeline::MultiOsPipeline),
        Arc::new(crate::p2p_market::P2PPluginMarketplace::new()),
        Arc::new(HsmPkcs11Engine::new()),
    );
    for tier in [
        serde_json::json!("UNKNOWN"),
        serde_json::json!(42),
        serde_json::Value::Null,
    ] {
        let response = router.handle_request(crate::ipc::protocol::IpcRequest {
            id: "wasm-tier".into(),
            command: "wasm_execute".into(),
            nonce: handshake.get_token(),
            payload: serde_json::json!({"moduleId":"wasm-schema-validator","input":{},"tier":tier}),
        });
        assert!(
            !response.success,
            "Explicit invalid tier must not silently execute in AUTO mode"
        );
    }
}

#[test]
fn ipc_crash_history_is_bounded() {
    use std::sync::Arc;
    let d = tempfile::tempdir().unwrap();
    let manifest = CapabilityManifest::default();

    let security = Arc::new(SecurityEngine::new(manifest, d.path()));
    let handshake = Arc::new(HandshakeSession::new());
    let create_router = || {
        crate::ipc::IpcRouter::new(
            security.clone(),
            handshake.clone(),
            crate::plugin::PluginRegistry::new(),
            Arc::new(crate::updater::AutoUpdater::new("test", d.path())),
            Arc::new(HardwareKeyEngine::new()),
            Arc::new(crate::profile::ProfileManager::new(d.path())),
            Arc::new(crate::wasm::WasmSandbox::new()),
            Arc::new(BiometricEnclave::new()),
            Arc::new(crate::crash::CrashReporter::new(d.path())),
            Arc::new(crate::secrets::SecretManager::new()),
            Arc::new(crate::network::NetworkPolicyEngine::new()),
            Arc::new(SignatureVerifier::new()),
            Arc::new(TpmAttestationEngine::new()),
            Arc::new(crate::package_pipeline::MultiOsPipeline),
            Arc::new(crate::p2p_market::P2PPluginMarketplace::new()),
            Arc::new(HsmPkcs11Engine::new()),
        )
    };
    let router = create_router();
    let mut ids = std::collections::HashSet::new();
    for _ in 0..40 {
        let response = router.handle_request(crate::ipc::protocol::IpcRequest {
            id: "crash-fixture".into(),
            command: "crash_record".into(),
            nonce: handshake.get_token(),
            payload: serde_json::json!({"errorKind":"ControlledDiagnosticTrap"}),
        });
        assert!(response.success);
        assert!(
            ids.insert(
                response.data.as_ref().unwrap()["crash_id"]
                    .as_str()
                    .unwrap()
                    .to_string()
            ),
            "Every diagnostic event must have a distinct ID"
        );
    }
    let response = router.handle_request(crate::ipc::protocol::IpcRequest {
        id: "crash-list".into(),
        command: "crash_list".into(),
        nonce: handshake.get_token(),
        payload: serde_json::json!({}),
    });
    assert!(response.success);
    assert_eq!(
        response.data.as_ref().unwrap()["crashes"]
            .as_array()
            .unwrap()
            .len(),
        32,
        "Crash history must retain at most 32 events instead of growing for the process lifetime"
    );
    for kind in [
        serde_json::json!("TEST_ONLY_secret-not-a-kind"),
        serde_json::json!("x".repeat(65536)),
        serde_json::Value::Null,
    ] {
        let denied = router.handle_request(crate::ipc::protocol::IpcRequest {
            id: "reject-kind".into(),
            command: "crash_record".into(),
            nonce: handshake.get_token(),
            payload: serde_json::json!({"errorKind":kind,"stackTrace":"TEST_ONLY_hidden-trace"}),
        });
        assert!(
            !denied.success,
            "Diagnostic labels must be allowlisted instead of storing arbitrary payload strings"
        );
    }
    assert!(!serde_json::to_string(&response)
        .unwrap()
        .contains("TEST_ONLY_"));
    drop(router);
    let restarted = create_router();
    let response = restarted.handle_request(crate::ipc::protocol::IpcRequest {
        id: "restart-list".into(),
        command: "crash_list".into(),
        nonce: handshake.get_token(),
        payload: serde_json::json!({}),
    });
    assert_eq!(
        response.data.as_ref().unwrap()["crashes"]
            .as_array()
            .unwrap()
            .len(),
        32,
        "Bounded diagnostics must survive a normal process restart"
    );
    let store = d.path().join("diagnostics/events.json");
    let saved = d.path().join("diagnostics/saved-fixture.json");
    std::fs::rename(&store, &saved).unwrap();
    std::fs::create_dir(&store).unwrap();
    let failed = restarted.handle_request(crate::ipc::protocol::IpcRequest {
        id: "failed-write".into(),
        command: "crash_record".into(),
        nonce: handshake.get_token(),
        payload: serde_json::json!({"errorKind":"panic"}),
    });
    assert!(
        !failed.success,
        "A failed persistence attempt must never report persisted success"
    );
    std::fs::remove_dir(&store).unwrap();
    std::fs::rename(&saved, &store).unwrap();
    let restored = restarted.handle_request(crate::ipc::protocol::IpcRequest {
        id: "restored-list".into(),
        command: "crash_list".into(),
        nonce: handshake.get_token(),
        payload: serde_json::json!({}),
    });
    assert_eq!(restored.data.as_ref().unwrap()["total"], 32);
    std::fs::write(&store, b"{invalid history").unwrap();
    for command in ["crash_record", "crash_list"] {
        let failure = restarted.handle_request(crate::ipc::protocol::IpcRequest {
            id: "corrupt-history".into(),
            command: command.into(),
            nonce: handshake.get_token(),
            payload: serde_json::json!({}),
        });
        assert!(
            !failure.success,
            "Corrupted history must fail closed instead of being silently replaced"
        );
    }
}
