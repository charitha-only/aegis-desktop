//! Atomic replacement within one directory. Callers must authorize the path and enforce quotas.
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};
pub const MAX_FILE_BYTES: usize = 2 * 1024 * 1024 + 256;
pub const MAX_DIRECTORY_ENTRIES: usize = 4096;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryEntry {
    name: String,
    is_dir: bool,
    size_bytes: u64,
}

/// The caller must authorize this directory against the active profile.
pub fn list_directory(path: &Path) -> Result<Vec<DirectoryEntry>, String> {
    let directory = fs::read_dir(path).map_err(|_| "Cannot list application directory")?;
    let mut entries = Vec::new();
    for (index, entry) in directory.enumerate() {
        if index >= MAX_DIRECTORY_ENTRIES {
            return Err("Application directory exceeds 4096-entry scan limit".into());
        }
        let entry = entry.map_err(|_| "Cannot enumerate application directory")?;
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        if name.starts_with(".aegis-") {
            continue;
        }
        let metadata = entry
            .metadata()
            .map_err(|_| "Cannot inspect application directory entry")?;
        entries.push(DirectoryEntry {
            name,
            is_dir: metadata.is_dir(),
            size_bytes: metadata.len(),
        });
    }
    entries.sort_unstable_by(|left, right| left.name.cmp(&right.name));
    Ok(entries)
}

pub fn read_bounded(path: &Path) -> Result<String, String> {
    let f = File::open(path).map_err(|_| "Cannot open stored file")?;
    if f.metadata().map_err(|_| "Cannot inspect file")?.len() > MAX_FILE_BYTES as u64 {
        return Err("Stored file exceeds size limit".into());
    }
    let mut data = String::new();
    f.take(MAX_FILE_BYTES as u64 + 1)
        .read_to_string(&mut data)
        .map_err(|_| "Cannot read stored file")?;
    if data.len() > MAX_FILE_BYTES {
        return Err("Stored file exceeds size limit".into());
    }
    Ok(data)
}
fn no_link(path: &Path) -> Result<(), String> {
    if let Ok(m) = fs::symlink_metadata(path) {
        if m.file_type().is_symlink() || !m.is_file() {
            return Err("Linked or non-file storage target rejected".into());
        }
    }
    Ok(())
}
fn replacement(path: &Path, data: &[u8]) -> Result<(), String> {
    no_link(path)?;
    let parent = path.parent().ok_or("Missing parent directory")?;
    let mut random = [0u8; 16];
    getrandom::getrandom(&mut random).map_err(|_| "Secure random source unavailable")?;
    let temp = parent.join(format!(".aegis-tmp-{}", hex::encode(random)));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let result = (|| {
        let mut file = options
            .open(&temp)
            .map_err(|_| "Cannot create temporary storage file")?;
        file.write_all(data)
            .map_err(|_| "Cannot write temporary storage file")?;
        file.sync_all().map_err(|_| "Cannot flush storage file")?;
        drop(file);
        fs::rename(&temp, path).map_err(|_| "Cannot atomically replace stored file")?;
        #[cfg(unix)]
        {
            File::open(parent)
                .and_then(|f| f.sync_all())
                .map_err(|_| "Cannot sync storage directory")?;
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}
pub fn backup_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".bak");
    PathBuf::from(name)
}
pub fn atomic_write(path: &Path, data: &[u8], backup: bool) -> Result<(), String> {
    write_inner(path, data, backup, None)
}
pub fn atomic_vault_write(path: &Path, data: &[u8], password: &str) -> Result<(), String> {
    write_inner(path, data, true, Some(password))
}
fn write_inner(
    path: &Path,
    data: &[u8],
    backup: bool,
    password: Option<&str>,
) -> Result<(), String> {
    if data.len() > MAX_FILE_BYTES {
        return Err("Storage data exceeds limit".into());
    }
    let lock_path = path
        .parent()
        .ok_or("Missing parent")?
        .join(".aegis-write.lock");
    no_link(&lock_path)?;
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(lock_path)
        .map_err(|_| "Cannot open storage lock")?;
    lock.try_lock()
        .map_err(|_| "Storage is busy; retry the operation")?;
    no_link(path)?;
    if backup && path.exists() {
        let old = read_bounded(path)?;
        if !old.starts_with("AEGIS_VAULT_V2:") {
            return Err(
                "Refusing to replace a legacy or invalid vault; preserve and migrate it explicitly"
                    .into(),
            );
        }
        let password = password.ok_or("Vault replacement requires authentication")?;
        let authenticated = zeroize::Zeroizing::new(
            crate::security::VaultEngine::decrypt(&old, password)
                .map_err(|_| "Existing vault password required; no file was changed")?,
        );
        drop(authenticated);
        replacement(&backup_path(path), old.as_bytes())?;
    }
    replacement(path, data)
}
