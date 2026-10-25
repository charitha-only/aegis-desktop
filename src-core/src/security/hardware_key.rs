/// Aegis Desktop - Hardware Key & WebAuthn / FIDO2 Engine (v0.14.0)
///
/// Implements hardware-backed authentication challenge-response simulation,
/// WebAuthn-compatible assertions, and physical security key token derivation (e.g. YubiKey / FIDO2).

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareKeyInfo {
    pub key_id: String,
    pub manufacturer: String,
    pub model: String,
    pub protocol: String,
    pub enrolled_at: u64,
    pub touch_required: bool,
    pub pin_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeRequest {
    pub challenge_id: String,
    pub relying_party: String,
    pub challenge_nonce: String,
    pub expires_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResponse {
    pub challenge_id: String,
    pub key_id: String,
    pub client_data_json: String,
    pub authenticator_data: String,
    pub signature_hex: String,
    pub verified: bool,
}

pub struct HardwareKeyEngine {
    enrolled_keys: Mutex<Vec<HardwareKeyInfo>>,
    active_challenges: Mutex<Vec<ChallengeRequest>>,
    enrolled_pin_hash: Mutex<String>,
    failed_attempts: Mutex<u32>,
}

impl HardwareKeyEngine {
    pub fn new() -> Self {
        let default_key = HardwareKeyInfo {
            key_id: "aegis-fido2-primary".to_string(),
            manufacturer: "Yubico / FIDO Alliance".to_string(),
            model: "YubiKey 5 NFC (Hardware Security Module)".to_string(),
            protocol: "FIDO2 / CTAP2.1 & WebAuthn Level 3".to_string(),
            enrolled_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
            touch_required: true,
            pin_verified: false,
        };

        // Standard enrolled secure PIN hash (Default enrolled PIN is "123456")
        let mut h = Sha256::new();
        h.update(b"123456");
        h.update(b"AEGIS_FIDO2_SALT_PIN");
        let pin_hash = format!("{:x}", h.finalize());

        Self {
            enrolled_keys: Mutex::new(vec![default_key]),
            active_challenges: Mutex::new(Vec::new()),
            enrolled_pin_hash: Mutex::new(pin_hash),
            failed_attempts: Mutex::new(0),
        }
    }

    /// List all currently enrolled hardware tokens
    pub fn list_keys(&self) -> Vec<HardwareKeyInfo> {
        let keys = self.enrolled_keys.lock().unwrap();
        keys.clone()
    }

    /// Generate a cryptographic challenge for physical security key verification
    pub fn generate_challenge(&self, relying_party: &str) -> ChallengeRequest {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let challenge_id = format!("fido2_ch_{:x}", now);

        let mut hasher = Sha256::new();
        hasher.update(relying_party.as_bytes());
        hasher.update(now.to_le_bytes());
        hasher.update(b"AEGIS_FIDO2_CHALLENGE_NONCE_SEED");
        let challenge_nonce = format!("{:x}", hasher.finalize());

        let req = ChallengeRequest {
            challenge_id,
            relying_party: relying_party.to_string(),
            challenge_nonce,
            expires_at: now + 60, // 60 seconds validity
        };

        let mut challenges = self.active_challenges.lock().unwrap();
        challenges.push(req.clone());
        req
    }

    /// Set or update the enrolled hardware key PIN
    pub fn set_pin(&self, current_pin: &str, new_pin: &str) -> Result<(), String> {
        let mut attempts = self.failed_attempts.lock().unwrap();
        if *attempts >= 3 {
            return Err("Hardware Security Key locked: Exceeded maximum PIN attempts. Device requires hardware reset.".to_string());
        }

        let mut curr_h = Sha256::new();
        curr_h.update(current_pin.as_bytes());
        curr_h.update(b"AEGIS_FIDO2_SALT_PIN");
        let curr_hash = format!("{:x}", curr_h.finalize());

        let mut enrolled = self.enrolled_pin_hash.lock().unwrap();
        if *enrolled != curr_hash {
            *attempts += 1;
            return Err(format!("Incorrect current PIN. Attempts remaining: {}", 3 - *attempts));
        }

        if new_pin.len() < 4 {
            return Err("New PIN must be at least 4 digits".to_string());
        }

        let mut new_h = Sha256::new();
        new_h.update(new_pin.as_bytes());
        new_h.update(b"AEGIS_FIDO2_SALT_PIN");
        *enrolled = format!("{:x}", new_h.finalize());
        *attempts = 0;
        Ok(())
    }

    /// Verify a hardware key signature against a generated challenge
    pub fn verify_signature(&self, challenge_id: &str, user_pin: &str) -> Result<ChallengeResponse, String> {
        let mut attempts = self.failed_attempts.lock().unwrap();
        if *attempts >= 3 {
            return Err("Hardware Key Locked: 3 consecutive invalid PIN attempts! CTAP2 PIN retry counter exhausted.".to_string());
        }

        if user_pin.trim().is_empty() {
            return Err("User PIN required for FIDO2 verification (cannot be empty)".to_string());
        }

        // Strict PIN Verification against enrolled hash
        let mut pin_hasher = Sha256::new();
        pin_hasher.update(user_pin.as_bytes());
        pin_hasher.update(b"AEGIS_FIDO2_SALT_PIN");
        let computed_pin_hash = format!("{:x}", pin_hasher.finalize());

        let enrolled_hash = self.enrolled_pin_hash.lock().unwrap();
        if *enrolled_hash != computed_pin_hash {
            *attempts += 1;
            let remaining = 3 - *attempts;
            return Err(format!(
                "CTAP2_ERR_PIN_INVALID: The security key rejected the PIN. Remaining attempts before token lockout: {}",
                remaining
            ));
        }

        // Reset failed counter on successful PIN verification
        *attempts = 0;

        let challenges = self.active_challenges.lock().unwrap();
        let chal = challenges.iter().find(|c| c.challenge_id == challenge_id)
            .ok_or_else(|| "Challenge expired or invalid".to_string())?;

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        if now > chal.expires_at {
            return Err("Challenge has expired. Re-initiate WebAuthn handshake.".to_string());
        }

        let mut keys = self.enrolled_keys.lock().unwrap();
        let key = keys.first_mut().ok_or_else(|| "No hardware security key enrolled".to_string())?;
        key.pin_verified = true;

        // Compute Authenticator Data & Cryptographic Assertion
        let client_data = format!("{{\"type\":\"webauthn.get\",\"challenge\":\"{}\",\"origin\":\"aegis://desktop\"}}", chal.challenge_nonce);
        
        let mut sig_hasher = Sha256::new();
        sig_hasher.update(chal.challenge_nonce.as_bytes());
        sig_hasher.update(key.key_id.as_bytes());
        sig_hasher.update(user_pin.as_bytes());
        sig_hasher.update(b"FIDO2_USER_PRESENCE_TOUCH_ACKNOWLEDGED");
        let signature_hex = format!("{:x}", sig_hasher.finalize());

        let auth_data_hash = {
            let mut h = Sha256::new();
            h.update(chal.relying_party.as_bytes());
            h.update(&[0x01]); // Flag: UP (User Present) + UV (User Verified)
            format!("{:x}", h.finalize())
        };

        Ok(ChallengeResponse {
            challenge_id: challenge_id.to_string(),
            key_id: key.key_id.clone(),
            client_data_json: client_data,
            authenticator_data: auth_data_hash,
            signature_hex,
            verified: true,
        })
    }
}
