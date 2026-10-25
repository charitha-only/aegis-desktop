//! Platform-specific application data locations, independent of the working directory.
use std::path::{Path, PathBuf};

#[derive(Clone, Copy)]
pub enum Platform {
    Windows,
    Linux,
    Macos,
}

#[derive(Default)]
pub struct PlatformEnvironment {
    pub home: Option<PathBuf>,
    pub local_app_data: Option<PathBuf>,
    pub roaming_app_data: Option<PathBuf>,
    pub xdg_data_home: Option<PathBuf>,
    pub xdg_config_home: Option<PathBuf>,
    pub xdg_cache_home: Option<PathBuf>,
}

#[derive(serde::Serialize)]
pub struct AppDirectories {
    pub data: PathBuf,
    pub config: PathBuf,
    pub cache: PathBuf,
}

impl AppDirectories {
    /// Pure location resolution; creates no folders and changes no environment variables.
    pub fn resolve(
        platform: Platform,
        app_id: &str,
        env: &PlatformEnvironment,
    ) -> Result<Self, String> {
        let data = resolve_app_data_dir(
            platform,
            app_id,
            env.home.as_deref(),
            env.local_app_data.as_deref(),
            env.xdg_data_home.as_deref(),
        )?;
        let suffix = PathBuf::from("AegisDesktop").join(app_id);
        let (config, cache) = match platform {
            Platform::Windows => (
                env.roaming_app_data
                    .as_ref()
                    .ok_or("APPDATA is required for configuration")?
                    .join(&suffix),
                data.join("Cache"),
            ),
            Platform::Linux => {
                let select = |value: &Option<PathBuf>, fallback: &str| -> Result<PathBuf, String> {
                    match value.as_ref().filter(|p| p.is_absolute()) {
                        Some(path) => Ok(path.join(&suffix)),
                        None => Ok(env
                            .home
                            .as_ref()
                            .ok_or("HOME is required")?
                            .join(fallback)
                            .join(&suffix)),
                    }
                };
                (
                    select(&env.xdg_config_home, ".config")?,
                    select(&env.xdg_cache_home, ".cache")?,
                )
            }
            Platform::Macos => (
                data.clone(),
                env.home
                    .as_ref()
                    .ok_or("HOME is required")?
                    .join("Library/Caches")
                    .join(&suffix),
            ),
        };
        if !config.is_absolute() || !cache.is_absolute() {
            return Err("Platform directory roots must be absolute".into());
        }
        Ok(Self {
            data,
            config,
            cache,
        })
    }
}

impl PlatformEnvironment {
    pub fn current() -> Self {
        let read = |name| std::env::var_os(name).map(PathBuf::from);
        Self {
            home: read("HOME"),
            local_app_data: read("LOCALAPPDATA"),
            roaming_app_data: read("APPDATA"),
            xdg_data_home: read("XDG_DATA_HOME"),
            xdg_config_home: read("XDG_CONFIG_HOME"),
            xdg_cache_home: read("XDG_CACHE_HOME"),
        }
    }
}

impl Platform {
    pub fn current() -> Result<Self, String> {
        if cfg!(target_os = "windows") {
            Ok(Self::Windows)
        } else if cfg!(target_os = "linux") {
            Ok(Self::Linux)
        } else if cfg!(target_os = "macos") {
            Ok(Self::Macos)
        } else {
            Err("This operating system has no platform adapter".into())
        }
    }

    pub fn capabilities(self) -> serde_json::Value {
        serde_json::json!({
            "os": match self { Self::Windows => "windows", Self::Linux => "linux", Self::Macos => "macos" },
            "architecture": std::env::consts::ARCH,
            "embeddedWebview": false,
            "lifecycleAdapterCompiled": matches!(self, Self::Windows),
            "installerAdapterCompiled": matches!(self, Self::Windows),
            "osKeyStore": false,
            "encryptedFileStorage": true
        })
    }
}

/// Resolve a location from explicit platform environment inputs without touching disk.
pub fn resolve_app_data_dir(
    platform: Platform,
    app_id: &str,
    home: Option<&Path>,
    local: Option<&Path>,
    xdg: Option<&Path>,
) -> Result<PathBuf, String> {
    let device = app_id.split('.').next().unwrap_or("").to_ascii_uppercase();
    if app_id.is_empty()
        || app_id.len() > 128
        || app_id == "."
        || app_id == ".."
        || app_id.ends_with(['.', ' '])
        || !app_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._- ".contains(&b))
        || matches!(device.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (device.len() == 4
            && (device.starts_with("COM") || device.starts_with("LPT"))
            && matches!(device.as_bytes()[3], b'1'..=b'9'))
    {
        return Err("Application ID must be a portable single directory name".into());
    }
    let base = match platform {
        Platform::Windows => local.ok_or("LOCALAPPDATA is required")?.to_path_buf(),
        Platform::Linux => match xdg.filter(|p| p.is_absolute()) {
            Some(path) => path.to_path_buf(),
            None => home.ok_or("HOME is required")?.join(".local/share"),
        },
        Platform::Macos => home
            .ok_or("HOME is required")?
            .join("Library/Application Support"),
    };
    if !base.is_absolute() {
        return Err("Application data root must be absolute".into());
    }
    Ok(base.join("AegisDesktop").join(app_id))
}

/// Resolve the running platform's data directory. The caller creates it when needed.
pub fn app_data_dir(app_id: &str) -> Result<PathBuf, String> {
    let platform = if cfg!(target_os = "windows") {
        Platform::Windows
    } else if cfg!(target_os = "linux") {
        Platform::Linux
    } else if cfg!(target_os = "macos") {
        Platform::Macos
    } else {
        return Err("Application data locations are unsupported on this OS".into());
    };
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    let xdg = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from);
    resolve_app_data_dir(
        platform,
        app_id,
        home.as_deref(),
        local.as_deref(),
        xdg.as_deref(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_directories_keep_data_config_and_cache_in_standard_roots() {
        let root = std::env::temp_dir();
        let env = PlatformEnvironment {
            home: Some(root.join("home")),
            local_app_data: Some(root.join("local")),
            roaming_app_data: Some(root.join("roaming")),
            xdg_data_home: Some(root.join("data")),
            xdg_config_home: Some(root.join("config")),
            xdg_cache_home: Some(root.join("cache")),
        };
        let linux = AppDirectories::resolve(Platform::Linux, "Example", &env).unwrap();
        assert_eq!(linux.config, root.join("config/AegisDesktop/Example"));
        assert_eq!(linux.cache, root.join("cache/AegisDesktop/Example"));
        assert_eq!(linux.data, root.join("data/AegisDesktop/Example"));
        let windows = AppDirectories::resolve(Platform::Windows, "Example", &env).unwrap();
        assert_eq!(windows.data, root.join("local/AegisDesktop/Example"));
        assert_eq!(windows.config, root.join("roaming/AegisDesktop/Example"));
        assert_eq!(windows.cache, root.join("local/AegisDesktop/Example/Cache"));
        let mac = AppDirectories::resolve(Platform::Macos, "Example", &env).unwrap();
        assert_eq!(
            mac.config,
            root.join("home/Library/Application Support/AegisDesktop/Example")
        );
        assert_eq!(
            mac.cache,
            root.join("home/Library/Caches/AegisDesktop/Example")
        );
    }

    #[test]
    fn rejects_unsafe_app_ids_and_relative_environment_roots() {
        let root = std::env::temp_dir();
        for id in [
            "",
            "..",
            "../outside",
            "/absolute",
            "bad\\id",
            "bad:id",
            "CON",
            "NUL.txt",
            "trailing.",
            "trailing ",
        ] {
            assert!(
                resolve_app_data_dir(Platform::Windows, id, None, Some(&root), None).is_err(),
                "{id:?}"
            );
        }
        assert!(resolve_app_data_dir(
            Platform::Windows,
            "ExampleApp",
            None,
            Some(Path::new("relative")),
            None
        )
        .is_err());
        assert!(resolve_app_data_dir(
            Platform::Macos,
            "ExampleApp",
            Some(Path::new("relative")),
            None,
            None
        )
        .is_err());
        assert_eq!(
            resolve_app_data_dir(
                Platform::Linux,
                "ExampleApp",
                Some(&root),
                None,
                Some(Path::new("relative"))
            )
            .unwrap(),
            root.join(".local/share/AegisDesktop/ExampleApp")
        );
    }

    #[test]
    fn resolves_each_platform_without_creating_directories() {
        let root = std::env::temp_dir();
        let home = root.join("aegis-home-fixture");
        let local = root.join("aegis-local-fixture");
        let xdg = root.join("aegis-xdg-fixture");
        for (platform, expected) in [
            (Platform::Windows, local.join("AegisDesktop/ExampleApp")),
            (Platform::Linux, xdg.join("AegisDesktop/ExampleApp")),
            (
                Platform::Macos,
                home.join("Library/Application Support/AegisDesktop/ExampleApp"),
            ),
        ] {
            assert_eq!(
                resolve_app_data_dir(
                    platform,
                    "ExampleApp",
                    Some(&home),
                    Some(&local),
                    Some(&xdg)
                )
                .unwrap(),
                expected
            );
        }
        assert_eq!(
            resolve_app_data_dir(Platform::Linux, "ExampleApp", Some(&home), None, None).unwrap(),
            home.join(".local/share/AegisDesktop/ExampleApp")
        );
        assert!(resolve_app_data_dir(Platform::Windows, "ExampleApp", None, None, None).is_err());
    }
}
