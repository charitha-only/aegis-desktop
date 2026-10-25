/// Aegis Desktop - Native Application Bundler Engine (v0.12.0)
///
/// Provides one-click standalone packaging for Aegis applications.
/// Bundles the compiled Rust binary + frontend dist/ into a self-contained
/// portable application directory, with optional InnoSetup installer
/// script generation for Windows native .exe installers.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

// ─── Bundle Manifest ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleManifest {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub icon: Option<String>,
    pub target_platforms: Vec<String>,
}

impl Default for BundleManifest {
    fn default() -> Self {
        Self {
            name: "AegisApp".to_string(),
            version: "1.0.0".to_string(),
            author: "Aegis Developer".to_string(),
            description: "Secure Desktop Application built with Aegis".to_string(),
            icon: None,
            target_platforms: vec!["windows".to_string()],
        }
    }
}

// ─── Bundle Engine ─────────────────────────────────────────────────────────

pub struct BundleEngine;

impl BundleEngine {
    /// Build a portable standalone bundle from the Aegis project.
    /// Generates:
    ///  - `dist-bundle/<AppName>/` — portable distribution directory
    ///  - `dist-bundle/<AppName>-installer.iss` — InnoSetup script (Windows)
    pub fn bundle(project_root: &Path, manifest: &BundleManifest) -> Result<BundleReport, String> {
        if !cfg!(target_os = "windows") {
            return Err("UNAVAILABLE: only Windows packaging is supported".into());
        }
        for field in [&manifest.name, &manifest.version] {
            if field.is_empty()
                || !field
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
                || field == "."
                || field.contains("..")
            {
                return Err("Invalid package name or version".into());
            }
        }
        if [&manifest.author, &manifest.description]
            .iter()
            .any(|v| v.chars().any(|c| c.is_control() || c == '"' || c == '{'))
        {
            return Err("Invalid installer metadata".into());
        }
        if !project_root
            .join("src-core/target/release/aegis-core.exe")
            .is_file()
        {
            return Err("Release binary missing: run npm run build:release first".into());
        }
        if !project_root.join("src-ui/dist/index.html").is_file() {
            return Err("Frontend build missing: build src-ui first".into());
        }
        if !project_root.join("aegis.config.json").is_file() {
            return Err("Application configuration missing".into());
        }
        let bundle_root = project_root.join("dist-bundle").join(&manifest.name);
        if bundle_root.exists() {
            return Err("Bundle output already exists; choose a new package name to avoid mixing old assets".into());
        }

        // 1. Create output directory tree
        fs::create_dir_all(&bundle_root)
            .map_err(|e| format!("Failed to create bundle directory: {}", e))?;

        let mut report = BundleReport {
            app_name: manifest.name.clone(),
            version: manifest.version.clone(),
            output_dir: bundle_root.to_string_lossy().to_string(),
            files_bundled: Vec::new(),
            installer_script_path: None,
            success: false,
            message: String::new(),
        };

        // 2. Copy Rust binary (src-core/target/release/aegis-core.exe on Windows)
        let binary_candidates = [
            project_root
                .join("src-core")
                .join("target")
                .join("release")
                .join("aegis-core.exe"),
            project_root
                .join("src-core")
                .join("target")
                .join("release")
                .join("aegis-core"),
            project_root
                .join("src-core")
                .join("target")
                .join("debug")
                .join("aegis-core.exe"),
            project_root
                .join("src-core")
                .join("target")
                .join("debug")
                .join("aegis-core"),
        ];

        let mut binary_bundled = false;
        for candidate in &binary_candidates {
            if candidate.exists() {
                let dest = bundle_root.join("aegis-core.exe");
                fs::copy(candidate, &dest).map_err(|e| format!("Failed to copy binary: {}", e))?;
                report
                    .files_bundled
                    .push(dest.to_string_lossy().to_string());
                binary_bundled = true;
                break;
            }
        }

        if !binary_bundled {
            // Create placeholder note for documentation
            let note_path = bundle_root.join("BUILD_FIRST.txt");
            let note = "Run 'cargo build --release' in src-core/ to compile the Rust binary before bundling.\n";
            fs::write(&note_path, note)
                .map_err(|e| format!("Failed to write build note: {}", e))?;
            report
                .files_bundled
                .push(note_path.to_string_lossy().to_string());
        }

        // 3. Copy frontend dist/ (src-ui/dist/)
        let frontend_dist = project_root.join("src-ui").join("dist");
        if frontend_dist.exists() {
            let dest_ui = bundle_root.join("ui");
            Self::copy_dir_all(&frontend_dist, &dest_ui)
                .map_err(|e| format!("Failed to copy frontend: {}", e))?;
            report
                .files_bundled
                .push(dest_ui.to_string_lossy().to_string());
        } else {
            let note_path = bundle_root.join("BUILD_UI_FIRST.txt");
            let note = "Run 'npm run build' in src-ui/ to build the frontend before bundling.\n";
            fs::write(&note_path, note)
                .map_err(|e| format!("Failed to write UI build note: {}", e))?;
            report
                .files_bundled
                .push(note_path.to_string_lossy().to_string());
        }

        // 4. Copy aegis.config.json
        let config_src = project_root.join("aegis.config.json");
        if config_src.exists() {
            let config_dest = bundle_root.join("aegis.config.json");
            fs::copy(&config_src, &config_dest)
                .map_err(|e| format!("Failed to copy config: {}", e))?;
            report
                .files_bundled
                .push(config_dest.to_string_lossy().to_string());
        }

        // 5. Copy sandbox directory (empty, with .gitkeep marker)
        let sandbox_dest = bundle_root.join("sandbox");
        fs::create_dir_all(&sandbox_dest)
            .map_err(|e| format!("Failed to create sandbox dir: {}", e))?;
        fs::write(sandbox_dest.join(".keep"), "")
            .map_err(|e| format!("Failed to write sandbox marker: {}", e))?;

        // 6. Write app launcher batch script (Windows)
        let launcher_path = bundle_root.join(format!("Launch {}.bat", manifest.name));
        let launcher_content = format!(
            "@echo off\r\ntitle {} v{}\r\ncd /d \"%~dp0\"\r\nstart \"\" aegis-core.exe --spawn-window\r\n",
            manifest.name, manifest.version
        );
        fs::write(&launcher_path, launcher_content)
            .map_err(|e| format!("Failed to write launcher script: {}", e))?;
        report
            .files_bundled
            .push(launcher_path.to_string_lossy().to_string());

        // 7. Generate InnoSetup installer script
        let iss_content = Self::generate_innosetup_script(manifest, &bundle_root);
        let iss_path = project_root
            .join("dist-bundle")
            .join(format!("{}-installer.iss", manifest.name));
        fs::write(&iss_path, iss_content)
            .map_err(|e| format!("Failed to write InnoSetup script: {}", e))?;
        report.installer_script_path = Some(iss_path.to_string_lossy().to_string());

        // 8. Write bundle manifest
        let manifest_json =
            serde_json::to_string_pretty(manifest).unwrap_or_else(|_| "{}".to_string());
        let manifest_path = bundle_root.join("aegis-bundle.json");
        fs::write(&manifest_path, manifest_json)
            .map_err(|e| format!("Failed to write bundle manifest: {}", e))?;
        report
            .files_bundled
            .push(manifest_path.to_string_lossy().to_string());

        report.success = true;
        report.message = format!(
            "Bundle created successfully at: {}. {} files packaged.",
            report.output_dir,
            report.files_bundled.len()
        );

        Ok(report)
    }

    /// Recursively copy a directory tree
    fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            if ty.is_symlink() {
                return Err(std::io::Error::other("Linked assets are forbidden"));
            }
            let dest_path = dst.join(entry.file_name());
            if ty.is_dir() {
                Self::copy_dir_all(&entry.path(), &dest_path)?;
            } else {
                fs::copy(entry.path(), &dest_path)?;
            }
        }
        Ok(())
    }

    /// Generate a Windows InnoSetup installer script (.iss)
    fn generate_innosetup_script(manifest: &BundleManifest, bundle_dir: &Path) -> String {
        let bundle_dir_str = bundle_dir.to_string_lossy().replace('/', "\\");
        format!(
            r#"; Aegis Desktop - Auto-Generated InnoSetup Installer Script
; Generated by Aegis Bundler v0.12.0
; Compile this with Inno Setup Compiler (https://jrsoftware.org/isinfo.php)

[Setup]
AppName={name}
AppVersion={version}
AppPublisher={author}
AppComments={description}
DefaultDirName={{localappdata}}\Programs\{name}
DefaultGroupName={name}
OutputDir=dist-bundle\installers
OutputBaseFilename={name}-Setup-v{version}
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
DisableProgramGroupPage=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Create a &desktop icon"; GroupDescription: "Additional icons:"

[Files]
Source: "{bundle_dir}\*"; DestDir: "{{app}}"; Flags: recursesubdirs createallsubdirs ignoreversion

[Icons]
Name: "{{group}}\{name}"; Filename: "{{app}}\aegis-core.exe"
Name: "{{commondesktop}}\{name}"; Filename: "{{app}}\aegis-core.exe"; Tasks: desktopicon

[Run]
Filename: "{{app}}\aegis-core.exe"; Parameters: "--spawn-window"; Description: "Launch {name}"; Flags: nowait postinstall skipifsilent shellexec
"#,
            name = manifest.name,
            version = manifest.version,
            author = manifest.author,
            description = manifest.description,
            bundle_dir = bundle_dir_str,
        )
    }
}

// ─── Bundle Report ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct BundleReport {
    pub app_name: String,
    pub version: String,
    pub output_dir: String,
    pub files_bundled: Vec<String>,
    pub installer_script_path: Option<String>,
    pub success: bool,
    pub message: String,
}

/// Read bundle manifest from aegis.config.json or return defaults
pub fn load_bundle_manifest(config_path: &Path) -> BundleManifest {
    if let Ok(content) = fs::read_to_string(config_path) {
        if let Ok(parsed) = serde_json::from_str::<Value>(&content) {
            return BundleManifest {
                name: parsed["name"]
                    .as_str()
                    .unwrap_or("AegisApp")
                    .replace(' ', "")
                    .to_string(),
                version: parsed["version"].as_str().unwrap_or("1.0.0").to_string(),
                author: parsed["author"]
                    .as_str()
                    .unwrap_or("Aegis Developer")
                    .to_string(),
                description: parsed["description"]
                    .as_str()
                    .unwrap_or("Secure Aegis Desktop Application")
                    .to_string(),
                icon: parsed["icon"].as_str().map(|s| s.to_string()),
                target_platforms: vec![
                    "windows".to_string(),
                    "linux".to_string(),
                    "macos".to_string(),
                ],
            };
        }
    }
    BundleManifest::default()
}
