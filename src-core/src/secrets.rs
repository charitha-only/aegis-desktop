use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMetadata {
    pub key: String,
    pub version: u32,
    pub created_at: u64,
    pub rotated_at: Option<u64>,
    pub memory_hard_iterations: u32,
    pub digest_fingerprint: String,
}

struct StoredSecret {
    envelope: String,
    meta: SecretMetadata,
}
/// Ephemeral encrypted secrets. Cleared when the process exits; use vault files for persistence.
pub struct SecretManager {
    secrets: Mutex<HashMap<String, StoredSecret>>,
}
impl SecretManager {
    pub fn new() -> Self {
        Self {
            secrets: Mutex::new(HashMap::new()),
        }
    }
    pub fn store_secret(
        &self,
        key: &str,
        value: &str,
        password: &str,
    ) -> Result<SecretMetadata, String> {
        let mut secrets = self.secrets.lock().unwrap();
        if secrets.len() >= 128 && !secrets.contains_key(key) {
            return Err("Secret quota exceeded".into());
        }
        if value.len() > 65536 {
            return Err("Secret exceeds 64 KiB".into());
        }
        let envelope = crate::security::VaultEngine::encrypt(value, password)?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let version = secrets.get(key).map_or(1, |s| s.meta.version + 1);
        let meta = SecretMetadata {
            key: key.into(),
            version,
            created_at: secrets.get(key).map_or(now, |s| s.meta.created_at),
            rotated_at: if version > 1 { Some(now) } else { None },
            memory_hard_iterations: 2,
            digest_fingerprint: hex::encode(Sha256::digest(envelope.as_bytes())),
        };
        secrets.insert(
            key.into(),
            StoredSecret {
                envelope,
                meta: meta.clone(),
            },
        );
        Ok(meta)
    }
    pub fn retrieve_secret(&self, key: &str, password: &str) -> Result<String, String> {
        let s = self.secrets.lock().unwrap();
        crate::security::VaultEngine::decrypt(
            &s.get(key).ok_or("Secret not found")?.envelope,
            password,
        )
    }
    pub fn list_secrets(&self) -> Vec<SecretMetadata> {
        self.secrets
            .lock()
            .unwrap()
            .values()
            .map(|s| s.meta.clone())
            .collect()
    }
}
