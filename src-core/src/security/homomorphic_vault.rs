//! Additive Homomorphic Encryption Engine over Curve25519.
//!
//! Provides exponential ElGamal additive homomorphic encryption allowing secure
//! summation and scalar multiplication of encrypted financial balances without
//! disclosing plaintext quantities:
//! $\text{Enc}(m_1) \oplus \text{Enc}(m_2) = \text{Enc}(m_1 + m_2)$.

use curve25519_dalek::constants::ED25519_BASEPOINT_POINT;
use curve25519_dalek::edwards::{CompressedEdwardsY, EdwardsPoint};
use curve25519_dalek::scalar::Scalar;
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HomomorphicCiphertext {
    pub c1_hex: String,
    pub c2_hex: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[allow(dead_code)]
pub struct HomomorphicAdditionResult {
    pub is_valid: bool,
    pub aggregated_ciphertext: HomomorphicCiphertext,
    pub operations_count: usize,
    pub protocol: String,
}

#[allow(dead_code)]
pub struct HomomorphicVaultEngine {
    private_key: Scalar,
    pub public_key: EdwardsPoint,
}

#[allow(dead_code)]
impl HomomorphicVaultEngine {
    pub fn new(seed: &[u8; 32]) -> Self {
        let private_key = Scalar::from_bytes_mod_order(*seed);
        let public_key = ED25519_BASEPOINT_POINT * private_key;
        Self {
            private_key,
            public_key,
        }
    }

    /// Encrypt a 32-bit amount m with random blinding scalar r:
    /// C1 = r * G
    /// C2 = m * G + r * Y
    pub fn encrypt(&self, amount: u32, randomness_seed: &[u8; 32]) -> HomomorphicCiphertext {
        let r = Scalar::from_bytes_mod_order(*randomness_seed);
        let m = Scalar::from(amount as u64);

        let c1 = ED25519_BASEPOINT_POINT * r;
        let c2 = (ED25519_BASEPOINT_POINT * m) + (self.public_key * r);

        HomomorphicCiphertext {
            c1_hex: hex::encode(c1.compress().as_bytes()),
            c2_hex: hex::encode(c2.compress().as_bytes()),
        }
    }

    /// Add two encrypted ciphertexts without decrypting:
    /// C_sum = (C1 + C1', C2 + C2')
    pub fn add(
        ct1: &HomomorphicCiphertext,
        ct2: &HomomorphicCiphertext,
    ) -> Result<HomomorphicCiphertext, String> {
        let c1_1 = Self::parse_point(&ct1.c1_hex)?;
        let c2_1 = Self::parse_point(&ct1.c2_hex)?;

        let c1_2 = Self::parse_point(&ct2.c1_hex)?;
        let c2_2 = Self::parse_point(&ct2.c2_hex)?;

        let c1_sum = c1_1 + c1_2;
        let c2_sum = c2_1 + c2_2;

        Ok(HomomorphicCiphertext {
            c1_hex: hex::encode(c1_sum.compress().as_bytes()),
            c2_hex: hex::encode(c2_sum.compress().as_bytes()),
        })
    }

    /// Multiply an encrypted ciphertext by a plaintext scalar k:
    /// C_scaled = (k * C1, k * C2)
    pub fn multiply_scalar(
        ct: &HomomorphicCiphertext,
        scalar_k: u32,
    ) -> Result<HomomorphicCiphertext, String> {
        let c1 = Self::parse_point(&ct.c1_hex)?;
        let c2 = Self::parse_point(&ct.c2_hex)?;

        let k = Scalar::from(scalar_k as u64);
        let c1_scaled = c1 * k;
        let c2_scaled = c2 * k;

        Ok(HomomorphicCiphertext {
            c1_hex: hex::encode(c1_scaled.compress().as_bytes()),
            c2_hex: hex::encode(c2_scaled.compress().as_bytes()),
        })
    }

    /// Decrypt ciphertext: recovers M = C2 - x * C1 = m * G, then extracts discrete log
    pub fn decrypt(&self, ct: &HomomorphicCiphertext, max_search: u32) -> Result<u32, String> {
        let c1 = Self::parse_point(&ct.c1_hex)?;
        let c2 = Self::parse_point(&ct.c2_hex)?;

        let m_point = c2 - (c1 * self.private_key);

        // Baby-step search for discrete log: find m such that m * G == m_point
        let target_compressed = m_point.compress();

        // Check 0 first
        if target_compressed == (ED25519_BASEPOINT_POINT * Scalar::ZERO).compress() {
            return Ok(0);
        }

        // Fast baby-step giant-step table
        let step_size = 1000u32;
        let mut table = HashMap::new();
        let mut curr = EdwardsPoint::default();

        for i in 0..step_size {
            table.insert(curr.compress(), i);
            curr += ED25519_BASEPOINT_POINT;
        }

        let giant_step = ED25519_BASEPOINT_POINT * Scalar::from(step_size as u64);
        let mut gamma = m_point;

        for j in 0..(max_search / step_size + 1) {
            let comp = gamma.compress();
            if let Some(&i) = table.get(&comp) {
                let ans = (j * step_size).wrapping_add(i);
                if ans <= max_search {
                    return Ok(ans);
                }
            }
            gamma -= giant_step;
        }

        Err("Discrete logarithm exceeds search bounds".to_string())
    }

    fn parse_point(hex_str: &str) -> Result<EdwardsPoint, String> {
        let bytes = hex::decode(hex_str).map_err(|e| e.to_string())?;
        if bytes.len() != 32 {
            return Err("Invalid Edwards point byte length".to_string());
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        CompressedEdwardsY(arr)
            .decompress()
            .ok_or_else(|| "Failed to decompress point".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_homomorphic_encryption_addition_roundtrip() {
        let key_seed = [0x77u8; 32];
        let vault = HomomorphicVaultEngine::new(&key_seed);

        let amount_a = 1500u32;
        let amount_b = 2750u32;

        let ct_a = vault.encrypt(amount_a, &[0x11u8; 32]);
        let ct_b = vault.encrypt(amount_b, &[0x22u8; 32]);

        // Homomorphic sum without decryption: E(1500) + E(2750) = E(4250)
        let ct_sum = HomomorphicVaultEngine::add(&ct_a, &ct_b).expect("Addition should succeed");

        let decrypted_sum = vault.decrypt(&ct_sum, 100_000).expect("Decryption should succeed");
        assert_eq!(decrypted_sum, 4250, "1500 + 2750 must decrypt to 4250");
    }

    #[test]
    fn test_homomorphic_scalar_multiplication() {
        let key_seed = [0x88u8; 32];
        let vault = HomomorphicVaultEngine::new(&key_seed);

        let unit_price = 450u32;
        let quantity = 5u32;

        let ct = vault.encrypt(unit_price, &[0x33u8; 32]);
        // 5 * E(450) = E(2250)
        let ct_scaled = HomomorphicVaultEngine::multiply_scalar(&ct, quantity).unwrap();

        let total = vault.decrypt(&ct_scaled, 50_000).unwrap();
        assert_eq!(total, 2250, "450 * 5 must decrypt to 2250");
    }
}
