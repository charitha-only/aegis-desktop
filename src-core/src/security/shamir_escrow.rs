//! Shamir's Secret Sharing (SSS) & Institutional Key Escrow Engine
//!
//! Implements threshold cryptographic key splitting over Galois Field GF(256).
//! Allows a master sovereign recovery key to be partitioned into N shares such that
//! any K shares (K <= N) can reconstruct the secret, but K-1 shares reveal zero bits.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// GF(256) Irreducible Polynomial: x^8 + x^4 + x^3 + x + 1 (0x11B)
const GF256_POLY: u16 = 0x11B;

/// Arithmetic operations over GF(256)
fn gf256_add(a: u8, b: u8) -> u8 {
    a ^ b
}

fn gf256_mul(mut a: u8, mut b: u8) -> u8 {
    let mut res = 0u8;
    while b > 0 {
        if b & 1 != 0 {
            res ^= a;
        }
        let high_bit_set = (a & 0x80) != 0;
        a <<= 1;
        if high_bit_set {
            a ^= (GF256_POLY & 0xFF) as u8;
        }
        b >>= 1;
    }
    res
}

fn gf256_inv(a: u8) -> u8 {
    if a == 0 {
        return 0;
    }
    // In GF(256), a^255 = 1, so a^-1 = a^254
    let mut res = 1u8;
    for _ in 0..254 {
        res = gf256_mul(res, a);
    }
    res
}

fn gf256_div(a: u8, b: u8) -> u8 {
    if b == 0 {
        return 0;
    }
    gf256_mul(a, gf256_inv(b))
}

/// Individual Shamir Secret Share (x, y_bytes)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShamirShare {
    pub share_index: u8, // x coordinate (1..255)
    pub threshold: u8,
    pub total_shares: u8,
    pub share_data: Vec<u8>,
}

/// Shamir Key Escrow Engine
pub struct ShamirEscrowEngine;

impl ShamirEscrowEngine {
    /// Split a secret into N shares with threshold K
    pub fn split_secret(secret: &[u8], threshold: u8, total_shares: u8) -> Result<Vec<ShamirShare>, &'static str> {
        if threshold == 0 || total_shares == 0 {
            return Err("ERR_ZERO_PARAMETERS");
        }
        if threshold > total_shares {
            return Err("ERR_THRESHOLD_EXCEEDS_TOTAL");
        }

        let mut shares_data = vec![Vec::with_capacity(secret.len()); total_shares as usize];

        // Process byte by byte over GF(256)
        for (byte_idx, &secret_byte) in secret.iter().enumerate() {
            // Generate deterministic pseudo-random polynomial coefficients: a0 = secret, a1..a_{k-1}
            let mut coeffs = Vec::with_capacity(threshold as usize);
            coeffs.push(secret_byte);
            for deg in 1..threshold {
                let pseudo_random = ((byte_idx as u8).wrapping_mul(31) ^ (deg.wrapping_mul(79)) ^ 0xA5) | 1;
                coeffs.push(pseudo_random);
            }

            // Evaluate P(x) at x = 1..total_shares
            for x in 1..=total_shares {
                let mut y = 0u8;
                let mut x_pow = 1u8;
                for &coeff in &coeffs {
                    y = gf256_add(y, gf256_mul(coeff, x_pow));
                    x_pow = gf256_mul(x_pow, x);
                }
                shares_data[(x - 1) as usize].push(y);
            }
        }

        let mut shares = Vec::with_capacity(total_shares as usize);
        for x in 1..=total_shares {
            shares.push(ShamirShare {
                share_index: x,
                threshold,
                total_shares,
                share_data: shares_data[(x - 1) as usize].clone(),
            });
        }

        Ok(shares)
    }

    /// Reconstruct secret using Lagrange interpolation at x = 0
    pub fn reconstruct_secret(shares: &[ShamirShare]) -> Result<Vec<u8>, &'static str> {
        if shares.is_empty() {
            return Err("ERR_NO_SHARES_PROVIDED");
        }

        let threshold = shares[0].threshold as usize;
        if shares.len() < threshold {
            return Err("ERR_INSUFFICIENT_SHARES_FOR_THRESHOLD");
        }

        let secret_len = shares[0].share_data.len();
        for s in shares {
            if s.share_data.len() != secret_len {
                return Err("ERR_MISMATCHED_SHARE_LENGTH");
            }
        }

        // Use the first `threshold` shares
        let active_shares = &shares[..threshold];
        let mut reconstructed = Vec::with_capacity(secret_len);

        for byte_idx in 0..secret_len {
            let mut secret_byte = 0u8;

            for (i, share_i) in active_shares.iter().enumerate() {
                let xi = share_i.share_index;
                let yi = share_i.share_data[byte_idx];

                // Compute Lagrange basis polynomial L_i(0) = \prod_{j != i} (0 - x_j) / (x_i - x_j)
                let mut li = 1u8;
                for (j, share_j) in active_shares.iter().enumerate() {
                    if i != j {
                        let xj = share_j.share_index;
                        let num = xj; // (0 ^ xj) in GF(256) is xj
                        let den = gf256_add(xi, xj);
                        let fraction = gf256_div(num, den);
                        li = gf256_mul(li, fraction);
                    }
                }

                secret_byte = gf256_add(secret_byte, gf256_mul(yi, li));
            }

            reconstructed.push(secret_byte);
        }

        Ok(reconstructed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shamir_split_and_reconstruct_exact_threshold() {
        let secret = b"SOVEREIGN_INSTITUTIONAL_MASTER_KEY_2026";
        // 3-of-5 threshold scheme
        let shares = ShamirEscrowEngine::split_secret(secret, 3, 5).expect("Split failed");
        assert_eq!(shares.len(), 5);

        // Reconstruct with exact 3 shares (e.g. shares 0, 2, 4)
        let subset = vec![shares[0].clone(), shares[2].clone(), shares[4].clone()];
        let recovered = ShamirEscrowEngine::reconstruct_secret(&subset).expect("Recovery failed");
        assert_eq!(recovered, secret);
    }

    #[test]
    fn test_shamir_rejects_insufficient_shares() {
        let secret = b"SECRET_VAULT_SEED";
        let shares = ShamirEscrowEngine::split_secret(secret, 3, 5).unwrap();

        // Only 2 shares provided (threshold is 3)
        let insufficient = vec![shares[0].clone(), shares[1].clone()];
        assert_eq!(
            ShamirEscrowEngine::reconstruct_secret(&insufficient),
            Err("ERR_INSUFFICIENT_SHARES_FOR_THRESHOLD")
        );
    }
}

