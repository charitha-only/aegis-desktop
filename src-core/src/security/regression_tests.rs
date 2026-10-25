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
