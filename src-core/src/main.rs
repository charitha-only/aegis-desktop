mod bundler;
mod ipc;
mod plugin;
mod security;
mod server;
mod tray;
mod updater;
mod window;

use ipc::IpcRouter;
use plugin::{PluginRegistry, register_builtin_plugins};
use security::{CapabilityManifest, HandshakeSession, OriginValidator, SecurityEngine};
use server::CoreServer;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use updater::AutoUpdater;
use window::{WindowConfig, WindowManager};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("============================================================");
    println!("     🛡️  AEGIS DESKTOP CORE v0.13.0 • HIGH-SECURITY RUNTIME  ");
    println!("============================================================");

    // 1. Locate and parse aegis.config.json
    let config_path = Path::new("aegis.config.json");
    let mut allowed_origins = vec![
        "http://localhost:5173".to_string(),
        "http://127.0.0.1:5173".to_string(),
    ];
    let mut csp = "default-src 'self' 'unsafe-inline';".to_string();
    let mut win_title = "Aegis Core • Enterprise Desktop".to_string();
    let mut win_width = 1280;
    let mut win_height = 840;

    let manifest = if config_path.exists() {
        match fs::read_to_string(config_path) {
            Ok(content) => {
                let parsed: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
                if let Some(origins) = parsed["security"]["allowedOrigins"].as_array() {
                    allowed_origins = origins.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                }
                if let Some(csp_str) = parsed["security"]["contentSecurityPolicy"].as_str() {
                    csp = csp_str.to_string();
                }
                if let Some(w) = parsed["window"]["width"].as_u64() {
                    win_width = w as u32;
                }
                if let Some(h) = parsed["window"]["height"].as_u64() {
                    win_height = h as u32;
                }
                if let Some(t) = parsed["window"]["title"].as_str() {
                    win_title = t.to_string();
                }
                serde_json::from_value::<CapabilityManifest>(parsed["security"]["capabilities"].clone())
                    .unwrap_or_default()
            }
            Err(_) => CapabilityManifest::default(),
        }
    } else {
        CapabilityManifest::default()
    };

    let base_dir = std::env::current_dir()?;
    let security_engine = Arc::new(SecurityEngine::new(manifest, &base_dir));
    let handshake_session = Arc::new(HandshakeSession::new());
    let origin_validator = Arc::new(OriginValidator::new(allowed_origins));

    // 2b. Initialize Plugin Registry
    let plugin_registry = PluginRegistry::new();
    register_builtin_plugins(&plugin_registry);

    // 2c. Initialize Auto-Updater
    let auto_updater = Arc::new(AutoUpdater::new("0.13.0", &base_dir));

    let router = Arc::new(IpcRouter::new(
        Arc::clone(&security_engine),
        Arc::clone(&handshake_session),
        plugin_registry,
        auto_updater,
    ));

    // 2. Bind the Secure Loopback Bridge
    let bridge_addr = "127.0.0.1:48291";
    let server = CoreServer::bind(
        bridge_addr,
        router,
        handshake_session,
        origin_validator,
        csp,
    )?;

    println!("[Security] Capabilities enforced: Strict Sandbox & Whitelisted Origins.");
    println!("[Security] Nonce generation: Active (Cryptographic Session Validated).");

    // 3. Parse CLI arguments
    let args: Vec<String> = std::env::args().collect();

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
            Err(e) => eprintln!("[Bundler] ❌  Bundle failed: {}", e),
        }
        return Ok(());
    }

    // --spawn-window flag: launch UI Window
    if args.iter().any(|arg| arg == "--spawn-window") {
        let win_cfg = WindowConfig {
            title: win_title,
            width: win_width,
            height: win_height,
            resizable: true,
            modal: false,
            always_on_top: false,
            kiosk: false,
        };
        let _ = WindowManager::spawn_window("http://localhost:5173", &win_cfg);
    }

    // 4. Run Loopback IPC Server
    server.run();

    Ok(())
}
