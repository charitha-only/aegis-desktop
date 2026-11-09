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
        if name.to_ascii_lowercase().starts_with(".aegis-") {
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
    let mut random = zeroize::Zeroizing::new([0u8; 16]);
    getrandom::getrandom(&mut *random).map_err(|_| "Secure random source unavailable")?;
    let temp = parent.join(format!(".aegis-tmp-{}", hex::encode(&*random)));
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
        let written_len = file
            .metadata()
            .map_err(|_| "Cannot inspect temporary storage file")?
            .len();
        if written_len != data.len() as u64 {
            return Err("Storage write verification failed: size mismatch".into());
        }
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

/// Computes hex-encoded SHA-256 digest of arbitrary memory bytes.
pub fn compute_sha256(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(data))
}

/// Computes the hex-encoded SHA-256 digest of a stored file within the bounded file budget.
pub fn file_sha256(path: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    no_link(path)?;
    let file = File::open(path).map_err(|_| "Cannot open stored file for checksum calculation")?;
    let metadata = file.metadata().map_err(|_| "Cannot inspect file")?;
    if metadata.len() > MAX_FILE_BYTES as u64 {
        return Err("Stored file exceeds size limit".into());
    }
    let mut take_reader = file.take(MAX_FILE_BYTES as u64 + 1);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    let mut total_read = 0usize;
    loop {
        let n = take_reader
            .read(&mut buffer)
            .map_err(|_| "Cannot read stored file for checksum calculation")?;
        if n == 0 {
            break;
        }
        total_read += n;
        if total_read > MAX_FILE_BYTES {
            return Err("Stored file exceeds size limit".into());
        }
        hasher.update(&buffer[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

/// Verifies that a stored file matches an expected hex SHA-256 digest.
pub fn verify_file_sha256(path: &Path, expected_hex: &str) -> Result<bool, String> {
    let actual_hex = file_sha256(path)?;
    Ok(actual_hex.eq_ignore_ascii_case(expected_hex.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_and_verify_sha256() {
        let payload = b"Hello, Aegis Secure Zero-Trust Storage!";
        let hash = compute_sha256(payload);
        assert_eq!(hash.len(), 64);

        let mut rnd = [0u8; 8];
        let _ = getrandom::getrandom(&mut rnd);
        let temp_dir = std::env::temp_dir().join(format!("aegis-storage-test-{}", hex::encode(rnd)));
        let _ = fs::create_dir_all(&temp_dir);
        let file_path = temp_dir.join("payload.dat");

        atomic_write(&file_path, payload, false).expect("atomic write succeeds");
        assert!(verify_file_sha256(&file_path, &hash).expect("verification succeeds"));
        assert!(!verify_file_sha256(
            &file_path,
            "0000000000000000000000000000000000000000000000000000000000000000"
        )
        .unwrap());

        let computed = file_sha256(&file_path).expect("file_sha256 succeeds");
        assert_eq!(computed, hash);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
