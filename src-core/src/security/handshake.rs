use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct HandshakeSession {
    session_token: Mutex<zeroize::Zeroizing<String>>,
    created_at: u64,
}

impl HandshakeSession {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let mut random = [0u8; 32];
        getrandom::getrandom(&mut random).expect("OS secure random source is required");
        let token = hex::encode(random);

        Self {
            session_token: Mutex::new(zeroize::Zeroizing::new(token)),
            created_at: now,
        }
    }

    pub fn get_token(&self) -> String {
        self.session_token.lock().unwrap().to_string()
    }

    pub fn validate_nonce(&self, incoming_nonce: &str) -> bool {
        let active = self.session_token.lock().unwrap();
        // Constant time comparison to prevent timing attacks
        if active.len() != incoming_nonce.len() {
            return false;
        }
        active
            .as_bytes()
            .iter()
            .zip(incoming_nonce.as_bytes())
            .fold(0, |acc, (a, b)| acc | (a ^ b))
            == 0
    }
}
