//! Platform-specific application data locations, independent of the working directory.
use std::path::{Path, PathBuf};

#[derive(Clone, Copy)]
pub enum Platform {
    Windows,
    Linux,
    Macos,
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
