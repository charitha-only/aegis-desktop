//! Post-Quantum Double Ratchet Protocol (PQ-Ratchet).
//!
//! Implements a post-quantum forward-secure and break-in recoverable messaging ratchet.
//! Combines KEM-based ratchet steps with symmetric KDF chains to provide continuous
//! cryptographic renewal and replay rejection.

use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PqRatchetEncryptedMessage {
    pub session_id: String,
    pub ratchet_step: u32,
    pub message_counter: u32,
    pub kem_entropy_digest_hex: String,
    pub nonce_hex: String,
    pub ciphertext_hex: String,
    pub auth_tag_hex: String,
}

#[allow(dead_code)]
pub struct PqRatchetSession {
    pub session_id: String,
    root_key: [u8; 32],
    tx_chain_key: [u8; 32],
    rx_chain_key: [u8; 32],
    pub tx_count: u32,
    pub rx_count: u32,
    pub ratchet_step: u32,
    is_initiator: bool,
}

impl Drop for PqRatchetSession {
    fn drop(&mut self) {
        self.root_key.zeroize();
        self.tx_chain_key.zeroize();
        self.rx_chain_key.zeroize();
    }
}

#[allow(dead_code)]
impl PqRatchetSession {
    /// Initialize Alice's end of the ratchet session.
    pub fn init_alice(session_id: &str, shared_secret: &[u8; 32]) -> Self {
        let (root_key, tx_chain, rx_chain) = Self::derive_initial_keys(shared_secret, true);
        Self {
            session_id: session_id.to_string(),
            root_key,
            tx_chain_key: tx_chain,
            rx_chain_key: rx_chain,
            tx_count: 0,
            rx_count: 0,
            ratchet_step: 0,
            is_initiator: true,
        }
    }

    /// Initialize Bob's end of the ratchet session.
    pub fn init_bob(session_id: &str, shared_secret: &[u8; 32]) -> Self {
        let (root_key, tx_chain, rx_chain) = Self::derive_initial_keys(shared_secret, false);
        Self {
            session_id: session_id.to_string(),
            root_key,
            tx_chain_key: tx_chain,
            rx_chain_key: rx_chain,
            tx_count: 0,
            rx_count: 0,
            ratchet_step: 0,
            is_initiator: false,
        }
    }

    fn derive_initial_keys(
        shared_secret: &[u8; 32],
        is_alice: bool,
    ) -> ([u8; 32], [u8; 32], [u8; 32]) {
        let mut rk_hasher = Sha256::new();
        rk_hasher.update(b"AEGIS_PQ_RATCHET_ROOT_INIT");
        rk_hasher.update(shared_secret);
        let root_key: [u8; 32] = rk_hasher.finalize().into();

        let mut ck1_hasher = Sha256::new();
        ck1_hasher.update(b"AEGIS_PQ_RATCHET_CHAIN_A");
        ck1_hasher.update(&root_key);
        let chain_a: [u8; 32] = ck1_hasher.finalize().into();

        let mut ck2_hasher = Sha256::new();
        ck2_hasher.update(b"AEGIS_PQ_RATCHET_CHAIN_B");
        ck2_hasher.update(&root_key);
        let chain_b: [u8; 32] = ck2_hasher.finalize().into();

        if is_alice {
            (root_key, chain_a, chain_b)
        } else {
            (root_key, chain_b, chain_a)
        }
    }

    /// Advance the root key via Post-Quantum KEM shared entropy (DH / KEM ratchet step).
    pub fn step_pq_kem(&mut self, kem_shared_entropy: &[u8; 32]) {
        self.ratchet_step += 1;
        self.tx_count = 0;
        self.rx_count = 0;

        let mut rk_hasher = Sha256::new();
        rk_hasher.update(b"AEGIS_PQ_KEM_RATCHET_ADVANCE");
        rk_hasher.update(&self.root_key);
        rk_hasher.update(kem_shared_entropy);
        self.root_key = rk_hasher.finalize().into();

        // Re-seed sending and receiving chains
        let mut ck1_hasher = Sha256::new();
        ck1_hasher.update(b"AEGIS_PQ_RATCHET_CHAIN_A");
        ck1_hasher.update(&self.root_key);
        ck1_hasher.update(&self.ratchet_step.to_be_bytes());
        let chain_a: [u8; 32] = ck1_hasher.finalize().into();

        let mut ck2_hasher = Sha256::new();
        ck2_hasher.update(b"AEGIS_PQ_RATCHET_CHAIN_B");
        ck2_hasher.update(&self.root_key);
        ck2_hasher.update(&self.ratchet_step.to_be_bytes());
        let chain_b: [u8; 32] = ck2_hasher.finalize().into();

        if self.is_initiator {
            self.tx_chain_key = chain_a;
            self.rx_chain_key = chain_b;
        } else {
            self.tx_chain_key = chain_b;
            self.rx_chain_key = chain_a;
        }
    }

    /// Step the symmetric TX chain and derive a single-use message key.
    fn step_tx_chain(&mut self) -> [u8; 32] {
        let mut msg_key_hasher = Sha256::new();
        msg_key_hasher.update(b"AEGIS_PQ_MSG_KEY");
        msg_key_hasher.update(&self.tx_chain_key);
        msg_key_hasher.update(&self.tx_count.to_be_bytes());
        let msg_key: [u8; 32] = msg_key_hasher.finalize().into();

        let mut next_chain_hasher = Sha256::new();
        next_chain_hasher.update(b"AEGIS_PQ_NEXT_CHAIN_KEY");
        next_chain_hasher.update(&self.tx_chain_key);
        self.tx_chain_key = next_chain_hasher.finalize().into();

        self.tx_count += 1;
        msg_key
    }

    /// Step the symmetric RX chain and derive a single-use message key.
    fn step_rx_chain(&mut self, expected_counter: u32) -> Result<[u8; 32], String> {
        if expected_counter != self.rx_count {
            return Err(format!(
                "Sequence counter mismatch / out of order: expected {}, got {}",
                self.rx_count, expected_counter
            ));
        }

        let mut msg_key_hasher = Sha256::new();
        msg_key_hasher.update(b"AEGIS_PQ_MSG_KEY");
        msg_key_hasher.update(&self.rx_chain_key);
        msg_key_hasher.update(&self.rx_count.to_be_bytes());
        let msg_key: [u8; 32] = msg_key_hasher.finalize().into();

        let mut next_chain_hasher = Sha256::new();
        next_chain_hasher.update(b"AEGIS_PQ_NEXT_CHAIN_KEY");
        next_chain_hasher.update(&self.rx_chain_key);
        self.rx_chain_key = next_chain_hasher.finalize().into();

        self.rx_count += 1;
        Ok(msg_key)
    }

    /// Encrypt a payload using the symmetric ratchet chain and ChaCha20-Poly1305.
    pub fn encrypt(
        &mut self,
        plaintext: &[u8],
        associated_data: &[u8],
    ) -> Result<PqRatchetEncryptedMessage, String> {
        let msg_counter = self.tx_count;
        let mut msg_key = self.step_tx_chain();

        let cipher = ChaCha20Poly1305::new_from_slice(&msg_key)
            .map_err(|e| format!("Cipher init failed: {}", e))?;
        msg_key.zeroize();

        // 12-byte Nonce derived from ratchet step and message counter
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[0..4].copy_from_slice(&self.ratchet_step.to_be_bytes());
        nonce_bytes[4..8].copy_from_slice(&msg_counter.to_be_bytes());
        nonce_bytes[8..12].copy_from_slice(b"PQRT");
        let nonce = Nonce::from_slice(&nonce_bytes);

        let mut ad_buffer = Vec::from(associated_data);
        ad_buffer.extend_from_slice(self.session_id.as_bytes());
        ad_buffer.extend_from_slice(&self.ratchet_step.to_be_bytes());
        ad_buffer.extend_from_slice(&msg_counter.to_be_bytes());

        let payload = chacha20poly1305::aead::Payload {
            msg: plaintext,
            aad: &ad_buffer,
        };

        let ciphertext_with_tag = cipher
            .encrypt(nonce, payload)
            .map_err(|e| format!("Encryption failure: {}", e))?;

        if ciphertext_with_tag.len() < 16 {
            return Err("Encrypted payload too short for AEAD tag".to_string());
        }

        let split_idx = ciphertext_with_tag.len() - 16;
        let ct_bytes = &ciphertext_with_tag[..split_idx];
        let tag_bytes = &ciphertext_with_tag[split_idx..];

        let mut kem_digest = Sha256::new();
        kem_digest.update(&self.root_key);
        let kem_hex = hex::encode(kem_digest.finalize());

        Ok(PqRatchetEncryptedMessage {
            session_id: self.session_id.clone(),
            ratchet_step: self.ratchet_step,
            message_counter: msg_counter,
            kem_entropy_digest_hex: kem_hex,
            nonce_hex: hex::encode(nonce_bytes),
            ciphertext_hex: hex::encode(ct_bytes),
            auth_tag_hex: hex::encode(tag_bytes),
        })
    }

    /// Decrypt an inbound ratchet message and verify AEAD authenticity.
    pub fn decrypt(
        &mut self,
        msg: &PqRatchetEncryptedMessage,
        associated_data: &[u8],
    ) -> Result<Vec<u8>, String> {
        if msg.session_id != self.session_id {
            return Err("Session ID mismatch in ratchet message".to_string());
        }
        if msg.ratchet_step != self.ratchet_step {
            return Err(format!(
                "Ratchet step mismatch: expected step {}, received step {}",
                self.ratchet_step, msg.ratchet_step
            ));
        }

        let mut msg_key = self.step_rx_chain(msg.message_counter)?;

        let cipher = ChaCha20Poly1305::new_from_slice(&msg_key)
            .map_err(|e| format!("Cipher init failed: {}", e))?;
        msg_key.zeroize();

        let nonce_bytes = hex::decode(&msg.nonce_hex)
            .map_err(|e| format!("Invalid nonce hex: {}", e))?;
        if nonce_bytes.len() != 12 {
            return Err("Invalid nonce length for ChaCha20Poly1305".to_string());
        }
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ct_bytes = hex::decode(&msg.ciphertext_hex)
            .map_err(|e| format!("Invalid ciphertext hex: {}", e))?;
        let tag_bytes = hex::decode(&msg.auth_tag_hex)
            .map_err(|e| format!("Invalid tag hex: {}", e))?;

        let mut combined = ct_bytes;
        combined.extend_from_slice(&tag_bytes);

        let mut ad_buffer = Vec::from(associated_data);
        ad_buffer.extend_from_slice(self.session_id.as_bytes());
        ad_buffer.extend_from_slice(&msg.ratchet_step.to_be_bytes());
        ad_buffer.extend_from_slice(&msg.message_counter.to_be_bytes());

        let payload = chacha20poly1305::aead::Payload {
            msg: &combined,
            aad: &ad_buffer,
        };

        let decrypted = cipher
            .decrypt(nonce, payload)
            .map_err(|_| "Decryption/Tag verification failed: ciphertext or AAD tampered".to_string())?;

        Ok(decrypted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pq_ratchet_session_message_flow() {
        let master_seed = [0x42u8; 32];
        let mut alice = PqRatchetSession::init_alice("ratchet-sess-901", &master_seed);
        let mut bob = PqRatchetSession::init_bob("ratchet-sess-901", &master_seed);

        let msg1 = b"POST_QUANTUM_FORWARD_SECRET_PAYLOAD_ALPHA";
        let aad = b"SYSTEM_CHANNEL_01";

        // Alice encrypts to Bob
        let enc1 = alice.encrypt(msg1, aad).expect("Alice encryption should succeed");
        assert_eq!(enc1.message_counter, 0);

        let dec1 = bob.decrypt(&enc1, aad).expect("Bob decryption should succeed");
        assert_eq!(dec1, msg1);

        // Bob encrypts back to Alice
        let msg2 = b"BOB_ACKNOWLEDGEMENT_REPLY_BETA";
        let enc2 = bob.encrypt(msg2, aad).expect("Bob encryption should succeed");
        assert_eq!(enc2.message_counter, 0);

        let dec2 = alice.decrypt(&enc2, aad).expect("Alice decryption should succeed");
        assert_eq!(dec2, msg2);

        // Now advance PQ KEM Ratchet Step
        let kem_entropy = [0x99u8; 32];
        alice.step_pq_kem(&kem_entropy);
        bob.step_pq_kem(&kem_entropy);

        let msg3 = b"NEW_EPOCH_POST_KEM_ROTATED_MESSAGE";
        let enc3 = alice.encrypt(msg3, aad).expect("Alice encryption in new epoch");
        assert_eq!(enc3.ratchet_step, 1);

        let dec3 = bob.decrypt(&enc3, aad).expect("Bob decryption in new epoch");
        assert_eq!(dec3, msg3);
    }

    #[test]
    fn test_pq_ratchet_out_of_order_message_handling() {
        let master_seed = [0x11u8; 32];
        let mut alice = PqRatchetSession::init_alice("ratchet-seq-test", &master_seed);
        let mut bob = PqRatchetSession::init_bob("ratchet-seq-test", &master_seed);

        let msg1 = b"MESSAGE_COUNTER_0";
        let msg2 = b"MESSAGE_COUNTER_1";
        let aad = b"AAD_VALIDATION";

        let _enc1 = alice.encrypt(msg1, aad).unwrap();
        let enc2 = alice.encrypt(msg2, aad).unwrap();

        // Attempting to decrypt msg2 before msg1 should fail sequence check
        let result = bob.decrypt(&enc2, aad);
        assert!(result.is_err(), "Out of order message must be rejected by monotonic check");
        assert!(result.unwrap_err().contains("Sequence counter mismatch"));
    }
}
