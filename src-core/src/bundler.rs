/// Aegis Desktop - Native Application Bundler Engine (v0.12.0)
///
/// Provides one-click standalone packaging for Aegis applications.
/// Bundles the compiled Rust binary + frontend dist/ into a self-contained
/// portable application directory, with optional InnoSetup installer
/// script generation for Windows native .exe installers.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::io::Write;
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

// Owned build scratch space. Cooperating builders serialize by application name.
struct BundleTransaction {
    stage: Option<PathBuf>,
    lock_path: PathBuf,
    lock: Option<fs::File>,
    published_sidecar: Option<PathBuf>,
}
impl BundleTransaction {
    fn acquire(parent: &Path, name: &str) -> Result<Self, String> {
        match fs::create_dir(parent) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(format!("Cannot create bundle output parent: {e}")),
        }
        BundleEngine::require_asset_directory(parent)
            .map_err(|e| format!("Invalid bundle output parent: {e}"))?;
        let lock_path = parent.join(format!(".aegis-bundle-{name}.lock"));
        let lock = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
            .map_err(|e| {
                format!("Bundle build lock unavailable; inspect existing lock before retrying: {e}")
            })?;
        let mut transaction = Self {
            stage: None,
            lock_path,
            lock: Some(lock),
            published_sidecar: None,
        };
        let mut random = zeroize::Zeroizing::new([0u8; 16]);
        getrandom::getrandom(&mut *random)
            .map_err(|e| format!("Cannot allocate bundle staging name: {e}"))?;
        let stage = parent.join(format!(".aegis-stage-{name}-{}", hex::encode(&*random)));
        fs::create_dir(&stage).map_err(|e| format!("Cannot create private bundle stage: {e}"))?;
        transaction.stage = Some(stage);
        Ok(transaction)
    }
}
impl Drop for BundleTransaction {
    fn drop(&mut self) {
        if let Some(path) = self.published_sidecar.take() {
            if let Err(e) = fs::remove_file(path) {
                eprintln!("Cannot remove owned installer sidecar after failure: {e}");
            }
        }
        if let Some(path) = self.stage.take() {
            if let Err(e) = fs::remove_dir_all(path) {
                eprintln!("Cannot remove owned bundle stage: {e}");
            }
        }
        drop(self.lock.take());
        if let Err(e) = fs::remove_file(&self.lock_path) {
            eprintln!("Cannot remove owned bundle lock: {e}");
        }
    }
}

pub struct BundleEngine;

impl BundleEngine {
    /// Build a portable standalone bundle from the Aegis project.
    /// Generates:
    ///  - `dist-bundle/<AppName>/` — portable distribution directory
    ///  - `dist-bundle/<AppName>-installer.iss` — InnoSetup script (Windows)
    pub fn bundle(project_root: &Path, manifest: &BundleManifest) -> Result<BundleReport, String> {
        let executable = if cfg!(windows) {
            "aegis-core.exe"
        } else {
            "aegis-core"
        };
        let binary = project_root
            .join("src-core/target/release")
            .join(executable);
        Self::bundle_with_executable(project_root, manifest, &binary)
    }

    /// Bundle a caller-selected trusted host executable without a default-path fallback.
    pub fn bundle_with_executable(
        project_root: &Path,
        manifest: &BundleManifest,
        binary: &Path,
    ) -> Result<BundleReport, String> {
        if !manifest
            .target_platforms
            .iter()
            .any(|target| target == std::env::consts::OS)
        {
            return Err("Native bundle target must include the current operating system".into());
        }
        let executable = if cfg!(windows) {
            "aegis-core.exe"
        } else {
            "aegis-core"
        };
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
        if !binary.is_file() {
            return Err("Release binary missing: run npm run build:release first".into());
        }
        Self::require_asset_directory(&project_root.join("src-ui"))
            .and_then(|_| Self::validate_asset_tree(&project_root.join("src-ui/dist")))
            .map_err(|e| format!("Cannot validate frontend assets: {e}"))?;
        if !project_root.join("src-ui/dist/index.html").is_file() {
            return Err("Frontend build missing: build src-ui first".into());
        }
        if !project_root.join("aegis.config.json").is_file() {
            return Err("Application configuration missing".into());
        }
        let config: Value = serde_json::from_str(
            &fs::read_to_string(project_root.join("aegis.config.json"))
                .map_err(|e| format!("Cannot read application configuration: {e}"))?,
        )
        .map_err(|e| format!("Invalid application configuration: {e}"))?;
        if let Some(app_id) = config.get("appId") {
            crate::platform::packaged_app_id(Some(app_id), project_root)?;
        }
        let output_parent = project_root.join("dist-bundle");
        let final_bundle_root = output_parent.join(&manifest.name);
        Self::require_absent(&final_bundle_root)
            .map_err(|e| format!("Bundle output already exists or cannot be inspected: {e}"))?;
        let sidecar = output_parent.join(format!("{}-installer.iss", manifest.name));
        if cfg!(windows) {
            Self::require_absent(&sidecar).map_err(|e| {
                format!("Installer sidecar already exists or cannot be inspected: {e}")
            })?;
        }
        let mut transaction = BundleTransaction::acquire(&output_parent, &manifest.name)?;
        let stage = transaction
            .stage
            .as_ref()
            .ok_or("Missing owned bundle stage")?
            .clone();
        let bundle_root = stage.join("payload");
        fs::create_dir(&bundle_root).map_err(|e| format!("Failed to create staged bundle: {e}"))?;

        let mut report = BundleReport {
            app_name: manifest.name.clone(),
            version: manifest.version.clone(),
            output_dir: final_bundle_root.to_string_lossy().to_string(),
            files_bundled: Vec::new(),
            installer_script_path: None,
            success: false,
            message: String::new(),
        };

        // Copy only the selected executable; never substitute another build path.
        let destination = bundle_root.join(executable);
        fs::copy(&binary, &destination).map_err(|e| format!("Failed to copy binary: {e}"))?;
        report
            .files_bundled
            .push(destination.to_string_lossy().into_owned());

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

        let (launcher_path, launcher_content) = if cfg!(windows) {
            (bundle_root.join(format!("Launch {}.bat", manifest.name)), format!("@echo off\r\ntitle {} v{}\r\ncd /d \"%~dp0\"\r\nstart \"\" aegis-core.exe --spawn-window\r\n", manifest.name, manifest.version))
        } else {
            (bundle_root.join("launch.sh"), "#!/bin/sh\ncd \"$(dirname \"$0\")\" || exit 1\nexec ./aegis-core --spawn-window \"$@\"\n".to_string())
        };
        fs::write(&launcher_path, launcher_content)
            .map_err(|e| format!("Failed to write launcher: {e}"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&launcher_path, fs::Permissions::from_mode(0o700))
                .map_err(|e| e.to_string())?;
        }
        report
            .files_bundled
            .push(launcher_path.to_string_lossy().into_owned());

        // Inno Setup is Windows-only; Unix output is an unsigned portable directory.
        let staged_sidecar = stage.join("installer.iss");
        if cfg!(windows) {
            let iss_content = Self::generate_innosetup_script(manifest, &final_bundle_root);
            fs::write(&staged_sidecar, iss_content)
                .map_err(|e| format!("Failed to write staged InnoSetup script: {e}"))?;
            report.installer_script_path = Some(sidecar.to_string_lossy().into_owned());
        }

        // 8. Write bundle manifest
        let manifest_json =
            serde_json::to_string_pretty(manifest).unwrap_or_else(|_| "{}".to_string());
        let manifest_path = bundle_root.join("aegis-bundle.json");
        fs::write(&manifest_path, manifest_json)
            .map_err(|e| format!("Failed to write bundle manifest: {}", e))?;
        report
            .files_bundled
            .push(manifest_path.to_string_lossy().to_string());

        for file in &mut report.files_bundled {
            let relative = Path::new(file.as_str())
                .strip_prefix(&bundle_root)
                .map_err(|_| "Invalid staged report path")?;
            *file = final_bundle_root
                .join(relative)
                .to_string_lossy()
                .into_owned();
        }
        Self::require_absent(&final_bundle_root)
            .map_err(|e| format!("Bundle output appeared during build: {e}"))?;
        if cfg!(windows) {
            let mut installer = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&sidecar)
                .map_err(|e| format!("Cannot publish installer sidecar without overwrite: {e}"))?;
            transaction.published_sidecar = Some(sidecar);
            let bytes = fs::read(&staged_sidecar)
                .map_err(|e| format!("Cannot read staged installer: {e}"))?;
            installer
                .write_all(&bytes)
                .map_err(|e| format!("Cannot write installer sidecar: {e}"))?;
        }
        fs::rename(&bundle_root, &final_bundle_root)
            .map_err(|e| format!("Cannot publish staged bundle: {e}"))?;
        transaction.published_sidecar = None;

        report.success = true;
        report.message = format!(
            "Bundle created successfully at: {}. {} files packaged.",
            report.output_dir,
            report.files_bundled.len()
        );

        Ok(report)
    }

    fn require_absent(path: &Path) -> std::io::Result<()> {
        match fs::symlink_metadata(path) {
            Ok(_) => Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "Output is occupied",
            )),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn asset_metadata(path: &Path) -> std::io::Result<fs::Metadata> {
        let metadata = fs::symlink_metadata(path)?;
        #[cfg(windows)]
        let reparse = {
            use std::os::windows::fs::MetadataExt;
            metadata.file_attributes() & 0x400 != 0
        };
        #[cfg(not(windows))]
        let reparse = false;
        if reparse
            || metadata.file_type().is_symlink()
            || (!metadata.is_file() && !metadata.is_dir())
        {
            return Err(std::io::Error::other(
                "Linked or special frontend asset rejected",
            ));
        }
        Ok(metadata)
    }

    fn require_asset_directory(path: &Path) -> std::io::Result<()> {
        if !Self::asset_metadata(path)?.is_dir() {
            return Err(std::io::Error::other(
                "Frontend source must be an ordinary directory",
            ));
        }
        Ok(())
    }

    fn validate_asset_tree(path: &Path) -> std::io::Result<()> {
        Self::require_asset_directory(path)?;
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if Self::asset_metadata(&entry.path())?.is_dir() {
                Self::validate_asset_tree(&entry.path())?;
            }
        }
        Ok(())
    }

    /// Recursively copy a directory tree
    fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
        Self::require_asset_directory(src)?;
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = Self::asset_metadata(&entry.path())?;
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
AppId=Aegis.{name}
AppName={name}
AppVersion={version}
AppPublisher={author}
AppComments={description}
DefaultDirName={{localappdata}}\Programs\{name}
DefaultGroupName={name}
OutputDir={{#SourcePath}}installers
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
Name: "{{group}}\{name}"; Filename: "{{app}}\aegis-core.exe"; Parameters: "--spawn-window"
Name: "{{userdesktop}}\{name}"; Filename: "{{app}}\aegis-core.exe"; Parameters: "--spawn-window"; Tasks: desktopicon

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

/// Read explicit bundle identity; optional absent presentation fields retain defaults.
pub fn load_bundle_manifest(config_path: &Path) -> Result<BundleManifest, String> {
    let content = fs::read_to_string(config_path)
        .map_err(|e| format!("Cannot read bundle configuration: {e}"))?;
    let parsed: Value =
        serde_json::from_str(&content).map_err(|e| format!("Invalid bundle configuration: {e}"))?;
    let object = parsed
        .as_object()
        .ok_or("Bundle configuration must be an object")?;
    let string_field = |key: &str, default: Option<&str>| -> Result<String, String> {
        match object.get(key) {
            Some(Value::String(value)) => Ok(value.clone()),
            Some(_) => Err(format!("Bundle configuration {key} must be a string")),
            None => default
                .map(str::to_string)
                .ok_or_else(|| format!("Bundle configuration requires {key}")),
        }
    };
    let icon = match object.get("icon") {
        None | Some(Value::Null) => None,
        Some(Value::String(value)) => Some(value.clone()),
        Some(_) => return Err("Bundle configuration icon must be a string or null".into()),
    };
    Ok(BundleManifest {
        name: string_field("name", None)?.replace(' ', ""),
        version: string_field("version", None)?,
        author: string_field("author", Some("Aegis Developer"))?,
        description: string_field("description", Some("Secure Aegis Desktop Application"))?,
        icon,
        target_platforms: vec!["windows".into(), "linux".into(), "macos".into()],
    })
}
