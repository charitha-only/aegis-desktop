#![recursion_limit = "256"]

mod bundler;
mod clipboard;
mod crash;
mod file_dialog;
mod ipc;
#[cfg(feature = "native-desktop")]
mod native_window;
mod network;
mod p2p_market;
mod package_pipeline;
mod platform;
mod plugin;
mod profile;
mod secrets;
mod security;
mod server;
mod storage;
mod tray;
mod updater;
mod wasm;
mod window;

use crash::CrashReporter;
use ipc::IpcRouter;
use network::NetworkPolicyEngine;
use p2p_market::P2PPluginMarketplace;
use package_pipeline::MultiOsPipeline;
use plugin::{register_builtin_plugins, PluginRegistry};
use profile::ProfileManager;
use secrets::SecretManager;
use security::{
    BiometricEnclave, CapabilityManifest, HandshakeSession, HardwareKeyEngine, HsmPkcs11Engine,
    OriginValidator, SecurityEngine, SignatureVerifier, TpmAttestationEngine,
};
use server::CoreServer;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use updater::AutoUpdater;
use wasm::WasmSandbox;
use window::{WindowConfig, WindowManager};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().any(|arg| arg == "--file-picker-helper") {
        return file_dialog::run_helper().map_err(Into::into);
    }
    println!("============================================================");
    println!(
        "     AEGIS DESKTOP CORE v{} - EXPERIMENTAL RUNTIME",
        env!("CARGO_PKG_VERSION")
    );
    println!("============================================================");

    let args: Vec<String> = std::env::args().collect();
    let port: u16 = args
        .iter()
        .find_map(|a| a.strip_prefix("--port="))
        .unwrap_or("48291")
        .parse()?;
    let executable = std::env::current_exe()?;
    let app_dir = executable.parent().ok_or("Missing executable directory")?;
    let packaged = app_dir.join("ui/index.html").is_file();
    if packaged {
        std::env::set_current_dir(app_dir)?;
    }
    // --bundle flag: run the bundler then exit
    if args.iter().any(|arg| arg == "--bundle") {
        println!("[Bundler] 📦  Aegis Bundle Engine v0.12.0 — Packaging application...");
        let project_root = std::env::current_dir()?;
        let config_path = project_root.join("aegis.config.json");
        let bundle_manifest = bundler::load_bundle_manifest(&config_path);
        match bundler::BundleEngine::bundle(&project_root, &bundle_manifest) {
            Ok(report) => {
                println!("[Bundler] ✅  {}", report.message);
                if let Some(iss) = &report.installer_script_path {
                    println!("[Bundler] 📋  InnoSetup installer script: {}", iss);
                    println!("[Bundler] 💡  Compile with Inno Setup Compiler to generate .exe installer.");
                }
                println!("[Bundler] 📂  Bundle output: {}", report.output_dir);
            }
            Err(e) => return Err(e.into()),
        }
        return Ok(());
    }

    // 1. Locate and parse aegis.config.json
    let config_path = Path::new("aegis.config.json");
    let mut allowed_origins = vec![
        "http://localhost:5173".to_string(),
        "http://127.0.0.1:5173".to_string(),
    ];
    let mut csp = "default-src 'self' 'unsafe-inline';".to_string();
    let mut window_config = WindowConfig::default();

    let manifest = if config_path.exists() {
        match fs::read_to_string(config_path) {
            Ok(content) => {
                let parsed: serde_json::Value = serde_json::from_str(&content)?;
                if let Some(origins) = parsed["security"]["allowedOrigins"].as_array() {
                    allowed_origins = origins
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                }
                if let Some(csp_str) = parsed["security"]["contentSecurityPolicy"].as_str() {
                    csp = csp_str.to_string();
                }
                if let Some(window) = parsed.get("window") {
                    window_config = serde_json::from_value(window.clone())?;
                }
                serde_json::from_value::<CapabilityManifest>(
                    parsed["security"]["capabilities"].clone(),
                )?
            }
            Err(e) => return Err(e.into()),
        }
    } else {
        CapabilityManifest::default()
    };

    let base_dir = if packaged {
        let app_name = app_dir.file_name().ok_or("Missing app name")?;
        let dir = platform::app_data_dir(app_name.to_str().ok_or("Invalid application ID")?)?;
        fs::create_dir_all(&dir)?;
        dir
    } else {
        std::env::current_dir()?
    };
    let security_engine = Arc::new(SecurityEngine::new(manifest, &base_dir));
    let handshake_session = Arc::new(HandshakeSession::new());
    allowed_origins.push(format!("http://127.0.0.1:{port}"));
    if packaged {
        allowed_origins = vec![format!("http://127.0.0.1:{port}")];
        csp="default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; connect-src 'self'; img-src 'self' data:; font-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'none'".into();
    }
    let origin_validator = Arc::new(OriginValidator::new(allowed_origins));

    // 2b. Initialize Plugin Registry
    let plugin_registry = PluginRegistry::new();
    register_builtin_plugins(&plugin_registry);

    // 2c. Initialize Auto-Updater
    let auto_updater = Arc::new(AutoUpdater::new(env!("CARGO_PKG_VERSION"), &base_dir));

    // 2d. Initialize v0.14 Subsystems
    let hardware_keys = Arc::new(HardwareKeyEngine::new());
    let profile_manager = Arc::new(ProfileManager::new(&base_dir));
    let wasm_sandbox = Arc::new(WasmSandbox::new());

    // 2e. Initialize v0.15 Subsystems
    let biometric_enclave = Arc::new(BiometricEnclave::new());
    let crash_reporter = Arc::new(CrashReporter::new(&base_dir));

    // 2f. Initialize v0.16 Subsystems
    let secret_manager = Arc::new(SecretManager::new());
    let network_engine = Arc::new(NetworkPolicyEngine::new());
    let sig_verifier = Arc::new(SignatureVerifier::new());

    // 2g. Initialize v0.17 Subsystems
    let tpm_engine = Arc::new(TpmAttestationEngine::new());

    // 2h. Initialize v0.18 Subsystems
    let package_pipeline = Arc::new(MultiOsPipeline);
    let p2p_market = Arc::new(P2PPluginMarketplace::new());
    let hsm_engine = Arc::new(HsmPkcs11Engine::new());

    let router = Arc::new(IpcRouter::new(
        Arc::clone(&security_engine),
        Arc::clone(&handshake_session),
        plugin_registry,
        auto_updater,
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
    ));

    // 2. Bind the Secure Loopback Bridge
    let bridge_addr = format!("127.0.0.1:{port}");
    let server = CoreServer::bind(
        &bridge_addr,
        router,
        handshake_session,
        origin_validator,
        csp,
    )?;

    println!("[Security] Capabilities enforced: Strict Sandbox & Whitelisted Origins.");
    println!("[Security] Nonce generation: Active (Cryptographic Session Validated).");

    // 3. Parse CLI arguments
    // --spawn-window flag: launch UI Window
    if args
        .iter()
        .any(|arg| arg == "--spawn-window" || arg == "--native-window")
    {
        let win_cfg = window_config;
        let ui_url = if packaged {
            format!("http://127.0.0.1:{port}")
        } else {
            "http://localhost:5173".into()
        };
        if args.iter().any(|arg| arg == "--native-window") {
            #[cfg(feature = "native-desktop")]
            {
                std::thread::spawn(move || server.run());
                return native_window::run(&ui_url, &win_cfg, &base_dir);
            }
            #[cfg(not(feature = "native-desktop"))]
            return Err("Rebuild with --features native-desktop to use --native-window".into());
        }
        WindowManager::spawn_window(&ui_url, &win_cfg)?;
    }

    // 4. Run Loopback IPC Server
    server.run();

    Ok(())
}
