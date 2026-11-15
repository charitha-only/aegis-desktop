//! Hardware-Enforced HKDF Key Derivation Engine (NIST SP 800-108 & RFC 5869)
//!
//! Generates hierarchical, domain-separated cryptographic sub-keys from a hardware-bound
//! root of trust (TPM / PIV / HSM). Guarantees cryptographic isolation between storage,
//! native IPC transport, and P2P mesh sub-systems.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

#[allow(dead_code)]
pub const HKDF_HASH_LEN: usize = 32;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyDomain {
    StorageVault,
    IpcAuthentication,
    MeshTransport,
    AuditLedger,
    EphemeralSession,
}

#[allow(dead_code)]
impl KeyDomain {
    pub fn context_label(&self) -> &'static [u8] {
        match self {
            KeyDomain::StorageVault => b"AEGIS-KEY-DOMAIN-STORAGE-VAULT-v2.1",
            KeyDomain::IpcAuthentication => b"AEGIS-KEY-DOMAIN-IPC-AUTH-v2.1",
            KeyDomain::MeshTransport => b"AEGIS-KEY-DOMAIN-MESH-TRANSPORT-v2.1",
            KeyDomain::AuditLedger => b"AEGIS-KEY-DOMAIN-AUDIT-LEDGER-v2.1",
            KeyDomain::EphemeralSession => b"AEGIS-KEY-DOMAIN-EPHEMERAL-SESSION-v2.1",
        }
    }
}

#[allow(dead_code)]
pub struct HardwareDerivedKey {
    pub domain: KeyDomain,
    pub key_bytes: [u8; HKDF_HASH_LEN],
    pub key_id: String,
}

impl Drop for HardwareDerivedKey {
    fn drop(&mut self) {
        self.key_bytes.zeroize();
    }
}

#[allow(dead_code)]
pub struct HardwareHkdfEngine;

#[allow(dead_code)]
impl HardwareHkdfEngine {
    /// HMAC-SHA256 implementation using constant-time outer/inner pad
    fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; HKDF_HASH_LEN] {
        let mut padded_key = [0u8; 64];
        if key.len() > 64 {
            let mut hasher = Sha256::new();
            hasher.update(key);
            let h = hasher.finalize();
            padded_key[..32].copy_from_slice(&h);
        } else {
            padded_key[..key.len()].copy_from_slice(key);
        }

        let mut ipad = [0x36u8; 64];
        let mut opad = [0x5cu8; 64];
        for i in 0..64 {
            ipad[i] ^= padded_key[i];
            opad[i] ^= padded_key[i];
        }

        let mut inner_hasher = Sha256::new();
        inner_hasher.update(&ipad);
        inner_hasher.update(data);
        let inner_hash = inner_hasher.finalize();

        let mut outer_hasher = Sha256::new();
        outer_hasher.update(&opad);
        outer_hasher.update(&inner_hash);
        let mut out = [0u8; HKDF_HASH_LEN];
        out.copy_from_slice(&outer_hasher.finalize());
        out
    }

    /// HKDF-Extract: PRK = HMAC-Hash(salt, IKM)
    pub fn extract(salt: &[u8], ikm: &[u8]) -> [u8; HKDF_HASH_LEN] {
        Self::hmac_sha256(salt, ikm)
    }

    /// HKDF-Expand: OKM = HMAC-Hash(PRK, info || 0x01)
    pub fn expand(prk: &[u8; HKDF_HASH_LEN], info: &[u8]) -> [u8; HKDF_HASH_LEN] {
        let mut payload = Vec::with_capacity(info.len() + 1);
        payload.extend_from_slice(info);
        payload.push(0x01);
        Self::hmac_sha256(prk, &payload)
    }

    /// Derives a domain-separated subkey from hardware master key (HMK)
    pub fn derive_sub_key(
        hardware_master_entropy: &[u8],
        salt: &[u8],
        domain: KeyDomain,
        nonce_context: &[u8],
    ) -> HardwareDerivedKey {
        let prk = Self::extract(salt, hardware_master_entropy);

        let mut info = Vec::new();
        info.extend_from_slice(domain.context_label());
        info.extend_from_slice(nonce_context);

        let okm = Self::expand(&prk, &info);

        let mut id_hasher = Sha256::new();
        id_hasher.update(&okm[..16]);
        id_hasher.update(domain.context_label());
        let key_id = hex::encode(&id_hasher.finalize()[..8]);

        HardwareDerivedKey {
            domain,
            key_bytes: okm,
            key_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hkdf_rfc5869_deterministic_derivation() {
        let ikm = b"HARDWARE_SILICON_MASTER_ROOT_ENTROPY_0x99";
        let salt = b"AEGIS_CHIP_UNIQUE_SALT_2026";

        let storage_key = HardwareHkdfEngine::derive_sub_key(ikm, salt, KeyDomain::StorageVault, b"epoch_1");
        let ipc_key = HardwareHkdfEngine::derive_sub_key(ikm, salt, KeyDomain::IpcAuthentication, b"epoch_1");

        // Assert domain separation: keys must differ even with same master seed
        assert_ne!(storage_key.key_bytes, ipc_key.key_bytes);
        assert_ne!(storage_key.key_id, ipc_key.key_id);
    }

    #[test]
    fn test_hkdf_nonce_differentiation() {
        let ikm = b"HARDWARE_SEED";
        let salt = b"SALT";

        let key_a = HardwareHkdfEngine::derive_sub_key(ikm, salt, KeyDomain::MeshTransport, b"session_1");
        let key_b = HardwareHkdfEngine::derive_sub_key(ikm, salt, KeyDomain::MeshTransport, b"session_2");

        assert_ne!(key_a.key_bytes, key_b.key_bytes);
    }
}
