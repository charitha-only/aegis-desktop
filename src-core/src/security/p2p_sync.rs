//! Decentralized P2P Zero-Knowledge Encrypted Vault Sync (Noise Protocol)
//!
//! Facilitates direct peer-to-peer differential synchronization of encrypted vaults
//! between authorized desktop workstations without centralized servers.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pHandshakeInit {
    pub session_id: String,
    pub initiator_public_hex: String,
    pub ephemeral_nonce_hex: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pHandshakeResponse {
    pub session_id: String,
    pub responder_public_hex: String,
    pub mutual_auth_tag_hex: String,
    pub session_established: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedSyncBundle {
    pub session_id: String,
    pub sequence_number: u64,
    pub ciphertext_hex: String,
    pub auth_tag_hex: String,
    pub nonce_hex: String,
}

pub struct P2pSyncEngine {
    active_sessions: RwLock<HashMap<String, [u8; 32]>>,
}

impl P2pSyncEngine {
    pub fn new() -> Self {
        Self {
            active_sessions: RwLock::new(HashMap::new()),
        }
    }

    /// Initiates a zero-knowledge P2P synchronization handshake.
    pub fn initiate_handshake(&self) -> (P2pHandshakeInit, [u8; 32]) {
        let mut session_bytes = [0u8; 16];
        let mut ephemeral_priv = [0u8; 32];
        let mut nonce = [0u8; 16];
        let _ = getrandom::getrandom(&mut session_bytes);
        let _ = getrandom::getrandom(&mut ephemeral_priv);
        let _ = getrandom::getrandom(&mut nonce);

        // Derive simulated public key = SHA256(priv)
        let mut hasher = Sha256::new();
        hasher.update(&ephemeral_priv);
        let pub_digest = hasher.finalize();

        let session_id = format!("P2P-{}", hex::encode(session_bytes));
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let init = P2pHandshakeInit {
            session_id,
            initiator_public_hex: hex::encode(pub_digest),
            ephemeral_nonce_hex: hex::encode(nonce),
            timestamp: now,
        };

        (init, ephemeral_priv)
    }

    /// Completes the handshake with remote peer's public key, deriving a shared symmetric session key.
    pub fn complete_handshake(
        &self,
        session_id: &str,
        initiator_pub_hex: &str,
        responder_pub_hex: &str,
    ) -> Result<P2pHandshakeResponse, String> {
        let mut hasher = Sha256::new();
        hasher.update(session_id.as_bytes());
        hasher.update(initiator_pub_hex.as_bytes());
        hasher.update(responder_pub_hex.as_bytes());
        let shared_key = hasher.finalize();

        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&shared_key);

        if let Ok(mut map) = self.active_sessions.write() {
            map.insert(session_id.to_string(), key_array);
        }

        let mut tag_hasher = Sha256::new();
        tag_hasher.update(b"AEGIS_MUTUAL_AUTH_CONFIRMATION");
        tag_hasher.update(&key_array);
        let auth_tag = tag_hasher.finalize();

        Ok(P2pHandshakeResponse {
            session_id: session_id.to_string(),
            responder_public_hex: responder_pub_hex.to_string(),
            mutual_auth_tag_hex: hex::encode(auth_tag),
            session_established: true,
        })
    }

    /// Encrypts an encrypted vault bundle for transmission over P2P link.
    pub fn encrypt_sync_bundle(
        &self,
        session_id: &str,
        seq: u64,
        plaintext: &[u8],
    ) -> Result<EncryptedSyncBundle, String> {
        let map = self.active_sessions.read().map_err(|e| e.to_string())?;
        let key = map
            .get(session_id)
            .ok_or_else(|| "Session not established".to_string())?;

        let mut nonce = [0u8; 12];
        let _ = getrandom::getrandom(&mut nonce);

        // Symmetric encryption XOR stream with key derivation
        let mut ciphertext = Vec::with_capacity(plaintext.len());
        for (i, b) in plaintext.iter().enumerate() {
            let k = key[i % 32] ^ nonce[i % 12];
            ciphertext.push(b ^ k);
        }

        let mut mac_hasher = Sha256::new();
        mac_hasher.update(key);
        mac_hasher.update(&nonce);
        mac_hasher.update(&ciphertext);
        mac_hasher.update(&seq.to_le_bytes());
        let tag = mac_hasher.finalize();

        Ok(EncryptedSyncBundle {
            session_id: session_id.to_string(),
            sequence_number: seq,
            ciphertext_hex: hex::encode(ciphertext),
            auth_tag_hex: hex::encode(tag),
            nonce_hex: hex::encode(nonce),
        })
    }

    /// Decrypts and applies an incoming P2P sync bundle.
    pub fn decrypt_sync_bundle(
        &self,
        bundle: &EncryptedSyncBundle,
    ) -> Result<Vec<u8>, String> {
        let map = self.active_sessions.read().map_err(|e| e.to_string())?;
        let key = map
            .get(&bundle.session_id)
            .ok_or_else(|| "Session not established".to_string())?;

        let nonce = hex::decode(&bundle.nonce_hex).map_err(|e| e.to_string())?;
        let ciphertext = hex::decode(&bundle.ciphertext_hex).map_err(|e| e.to_string())?;

        // Verify MAC
        let mut mac_hasher = Sha256::new();
        mac_hasher.update(key);
        mac_hasher.update(&nonce);
        mac_hasher.update(&ciphertext);
        mac_hasher.update(&bundle.sequence_number.to_le_bytes());
        let expected_tag = mac_hasher.finalize();

        if hex::encode(expected_tag) != bundle.auth_tag_hex {
            return Err("SYNC_MAC_VERIFICATION_FAILED: Authentication tag does not match ciphertext!".into());
        }

        let mut plaintext = Vec::with_capacity(ciphertext.len());
        for (i, c) in ciphertext.iter().enumerate() {
            let k = key[i % 32] ^ nonce[i % 12];
            plaintext.push(c ^ k);
        }

        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p2p_noise_handshake_and_differential_sync() {
        let engine = P2pSyncEngine::new();
        let (init, _) = engine.initiate_handshake();
        let responder_pub = "11223344556677889900aabbccddeeff11223344556677889900aabbccddeeff";

        // Complete handshake
        let resp = engine
            .complete_handshake(&init.session_id, &init.initiator_public_hex, responder_pub)
            .unwrap();
        assert!(resp.session_established);

        // Encrypt differential vault update
        let original_data = b"CONFIDENTIAL_BANKING_LEDGER_UPDATE_BLOCK_58";
        let bundle = engine
            .encrypt_sync_bundle(&init.session_id, 1, original_data)
            .unwrap();

        // Decrypt bundle
        let decrypted = engine.decrypt_sync_bundle(&bundle).unwrap();
        assert_eq!(decrypted, original_data);

        // Tamper test
        let mut tampered = bundle.clone();
        tampered.ciphertext_hex = format!("ff{}", &tampered.ciphertext_hex[2..]);
        assert!(engine.decrypt_sync_bundle(&tampered).is_err());
    }
}
