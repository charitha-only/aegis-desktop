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
