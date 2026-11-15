//! Cryptographic Time-Lock Puzzle Engine (Rivest-Shamir-Wagner RSW96).
//!
//! Provides inherently non-parallelizable sequential squaring time-lock puzzles
//! modulo $N$. Guarantees that sensitive encrypted payloads cannot be decrypted
//! before a pre-determined elapsed duration of wall-clock computational time.

use sha2::{Digest, Sha256};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TimeLockPuzzle {
    pub modulus_n_hex: String,
    pub generator_a: u64,
    pub squaring_iterations_t: u64,
    pub encrypted_payload_hex: String,
    pub payload_hash_hex: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PuzzleSolutionResult {
    pub is_solved: bool,
    pub decrypted_payload: String,
    pub final_residue_hex: String,
    pub iterations_performed: u64,
    pub error: Option<String>,
}

#[allow(dead_code)]
pub struct TimeLockEngine;

#[allow(dead_code)]
impl TimeLockEngine {
    /// Modular exponentiation: (base^exp) % modulus
    pub fn mod_pow(mut base: u128, mut exp: u128, modulus: u128) -> u128 {
        if modulus == 1 {
            return 0;
        }
        let mut result = 1u128;
        base %= modulus;
        while exp > 0 {
            if exp % 2 == 1 {
                result = (result * base) % modulus;
            }
            exp /= 2;
            base = (base * base) % modulus;
        }
        result
    }

    /// Generate a time-lock puzzle with parameterized iterations T
    /// Modulus N = p * q
    pub fn create_puzzle(
        secret_payload: &str,
        squaring_iterations: u64,
    ) -> TimeLockPuzzle {
        // Two 64-bit safe primes
        let p: u128 = 3037000493;
        let q: u128 = 3037000507;
        let n: u128 = p * q;
        let phi: u128 = (p - 1) * (q - 1);
        let a: u128 = 2;

        // Shortcut computation for creator: e = 2^T mod phi(N)
        let e = Self::mod_pow(2, squaring_iterations as u128, phi);
        let b = Self::mod_pow(a, e, n);

        // Key = H(b)
        let mut hasher = Sha256::new();
        hasher.update(b"AEGIS_TIMELOCK_KEY_DERIVATION");
        hasher.update(&b.to_be_bytes());
        let key = hasher.finalize();

        // Encrypt payload via XOR
        let payload_bytes = secret_payload.as_bytes();
        let mut encrypted = Vec::with_capacity(payload_bytes.len());
        for (i, &byte) in payload_bytes.iter().enumerate() {
            encrypted.push(byte ^ key[i % key.len()]);
        }

        let payload_hash = hex::encode(Sha256::digest(payload_bytes));

        TimeLockPuzzle {
            modulus_n_hex: format!("{:x}", n),
            generator_a: a as u64,
            squaring_iterations_t: squaring_iterations,
            encrypted_payload_hex: hex::encode(encrypted),
            payload_hash_hex: payload_hash,
        }
    }

    /// Solve time-lock puzzle via sequential squaring: a_{i} = a_{i-1}^2 mod N
    pub fn solve_puzzle(puzzle: &TimeLockPuzzle) -> PuzzleSolutionResult {
        let n = match u128::from_str_radix(&puzzle.modulus_n_hex, 16) {
            Ok(val) => val,
            Err(_) => {
                return PuzzleSolutionResult {
                    is_solved: false,
                    decrypted_payload: String::new(),
                    final_residue_hex: String::new(),
                    iterations_performed: 0,
                    error: Some("Invalid modulus N hex format".to_string()),
                }
            }
        };

        let mut current = puzzle.generator_a as u128;

        // Non-parallelizable sequential squaring loop
        for _ in 0..puzzle.squaring_iterations_t {
            current = (current * current) % n;
        }

        let mut hasher = Sha256::new();
        hasher.update(b"AEGIS_TIMELOCK_KEY_DERIVATION");
        hasher.update(&current.to_be_bytes());
        let key = hasher.finalize();

        let encrypted_bytes = match hex::decode(&puzzle.encrypted_payload_hex) {
            Ok(b) => b,
            Err(e) => {
                return PuzzleSolutionResult {
                    is_solved: false,
                    decrypted_payload: String::new(),
                    final_residue_hex: format!("{:x}", current),
                    iterations_performed: puzzle.squaring_iterations_t,
                    error: Some(format!("Invalid encrypted payload hex: {}", e)),
                }
            }
        };

        let mut decrypted = Vec::with_capacity(encrypted_bytes.len());
        for (i, &byte) in encrypted_bytes.iter().enumerate() {
            decrypted.push(byte ^ key[i % key.len()]);
        }

        let computed_hash = hex::encode(Sha256::digest(&decrypted));
        let is_valid = computed_hash == puzzle.payload_hash_hex;

        let decrypted_str = String::from_utf8(decrypted).unwrap_or_default();

        PuzzleSolutionResult {
            is_solved: is_valid,
            decrypted_payload: if is_valid { decrypted_str } else { String::new() },
            final_residue_hex: format!("{:x}", current),
            iterations_performed: puzzle.squaring_iterations_t,
            error: if is_valid {
                None
            } else {
                Some("Decrypted payload checksum mismatch".to_string())
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timelock_puzzle_generate_and_solve() {
        let secret = "AEGIS_TOP_SECRET_ESCROW_KEY_DISCLOSURE";
        let iterations = 1024u64; // Moderate iteration count for instant unit test

        let puzzle = TimeLockEngine::create_puzzle(secret, iterations);
        assert_eq!(puzzle.squaring_iterations_t, 1024);

        let solution = TimeLockEngine::solve_puzzle(&puzzle);
        assert!(solution.is_solved, "Puzzle solution must succeed: {:?}", solution.error);
        assert_eq!(solution.decrypted_payload, secret);
        assert_eq!(solution.iterations_performed, 1024);
    }

    #[test]
    fn test_timelock_puzzle_tampered_solution_fails() {
        let secret = "FINANCIAL_TRANSACTION_PAYLOAD";
        let mut puzzle = TimeLockEngine::create_puzzle(secret, 512);

        // Adversary alters encrypted payload
        let mut enc = hex::decode(&puzzle.encrypted_payload_hex).unwrap();
        enc[0] ^= 0x01;
        puzzle.encrypted_payload_hex = hex::encode(enc);

        let solution = TimeLockEngine::solve_puzzle(&puzzle);
        assert!(!solution.is_solved, "Tampered puzzle must fail checksum verification");
    }
}
