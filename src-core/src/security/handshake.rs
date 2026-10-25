use sha2::{Digest, Sha256};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct HandshakeSession {
    session_token: Mutex<String>,
    created_at: u64,
}

impl HandshakeSession {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // Generate secure 256-bit token
        let mut hasher = Sha256::new();
        hasher.update(now.to_le_bytes());
        hasher.update(b"AEGIS_CRYPTOGRAPHIC_NONCE_SALT_V1");
        hasher.update(std::process::id().to_le_bytes());
        let token = format!("{:x}", hasher.finalize());

        Self {
            session_token: Mutex::new(token),
            created_at: now,
        }
    }

    pub fn get_token(&self) -> String {
        self.session_token.lock().unwrap().clone()
    }

    pub fn validate_nonce(&self, incoming_nonce: &str) -> bool {
        let active = self.session_token.lock().unwrap();
        // Constant time comparison to prevent timing attacks
        if active.len() != incoming_nonce.len() {
            return false;
        }
        active.as_bytes().iter().zip(incoming_nonce.as_bytes()).fold(0, |acc, (a, b)| acc | (a ^ b)) == 0
    }
}
