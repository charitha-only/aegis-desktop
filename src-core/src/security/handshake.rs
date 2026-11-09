use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct HandshakeSession {
    session_token: Mutex<zeroize::Zeroizing<String>>,
    created_at: AtomicU64,
}

impl HandshakeSession {
    pub fn new() -> Self {
        let now = current_time_millis();
        let token = generate_secure_token();

        Self {
            session_token: Mutex::new(zeroize::Zeroizing::new(token)),
            created_at: AtomicU64::new(now),
        }
    }

    pub fn get_token(&self) -> String {
        self.session_token.lock().unwrap().to_string()
    }

    pub fn created_at(&self) -> u64 {
        self.created_at.load(Ordering::Acquire)
    }

    pub fn age_millis(&self) -> u64 {
        let now = current_time_millis();
        let created = self.created_at.load(Ordering::Acquire);
        now.saturating_sub(created)
    }

    #[allow(dead_code)]
    pub fn is_expired(&self, max_age_millis: u64) -> bool {
        self.age_millis() > max_age_millis
    }

    pub fn rotate(&self) -> String {
        let now = current_time_millis();
        let new_token = generate_secure_token();
        let mut guard = self.session_token.lock().unwrap();
        *guard = zeroize::Zeroizing::new(new_token.clone());
        self.created_at.store(now, Ordering::Release);
        new_token
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

fn current_time_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn generate_secure_token() -> String {
    let mut random = zeroize::Zeroizing::new([0u8; 32]);
    getrandom::getrandom(&mut *random).expect("OS secure random source is required");
    hex::encode(&*random)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handshake_token_generation_and_validation() {
        let session = HandshakeSession::new();
        let token = session.get_token();
        assert_eq!(token.len(), 64);
        assert!(session.validate_nonce(&token));
        assert!(!session.validate_nonce("invalid-nonce"));
        assert!(!session.is_expired(100_000));

        let rotated = session.rotate();
        assert_ne!(token, rotated);
        assert!(!session.validate_nonce(&token));
        assert!(session.validate_nonce(&rotated));
    }
}
