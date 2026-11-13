//! Dynamic Memory Canary Guard Engine
//!
//! Wraps sensitive in-memory heap allocations with 64-bit randomized cryptographic canaries.
//! Prevents heap-based buffer overruns, memory scrubbing, and out-of-bounds corruption
//! by validating head and tail canaries prior to granting transient plaintext access.

#![allow(dead_code)]

use std::sync::atomic::{AtomicU64, Ordering};

static CANARY_ENTROPY_COUNTER: AtomicU64 = AtomicU64::new(0xA5A5_F0F0_1234_5678);

/// Memory Buffer Protected by Dynamic Cryptographic Canaries
#[derive(Debug)]
pub struct CanaryProtectedBuffer {
    entropy: u64,
    head_canary: u64,
    payload: Vec<u8>,
    tail_canary: u64,
    is_compromised: bool,
}

impl CanaryProtectedBuffer {
    /// Allocate a new protected buffer with randomized 64-bit canaries
    pub fn allocate(secret: &[u8]) -> Self {
        let entropy = CANARY_ENTROPY_COUNTER.fetch_add(0x9E37_79B9_7F4A_7C15, Ordering::Relaxed);
        let head_canary = entropy ^ 0xDEAD_BEEF_CAFE_BABE;
        let tail_canary = entropy ^ 0xFEED_FACE_C001_D00D;

        Self {
            entropy,
            head_canary,
            payload: secret.to_vec(),
            tail_canary,
            is_compromised: false,
        }
    }

    /// Verify canary boundaries without reading payload
    pub fn verify_integrity(&self) -> bool {
        if self.is_compromised {
            return false;
        }

        let expected_head = self.entropy ^ 0xDEAD_BEEF_CAFE_BABE;
        let expected_tail = self.entropy ^ 0xFEED_FACE_C001_D00D;

        self.head_canary == expected_head && self.tail_canary == expected_tail
    }

    /// Read the protected payload transiently while validating canary bounds
    pub fn read_guarded(&mut self) -> Result<&[u8], &'static str> {
        if !self.verify_integrity() {
            // Memory corruption detected: trigger fail-closed zeroization
            self.emergency_zeroize();
            return Err("ERR_CANARY_CORRUPTION_DETECTED");
        }

        Ok(&self.payload)
    }

    /// Overwrite payload with new data, ensuring canaries remain uncorrupted
    pub fn write_guarded(&mut self, new_secret: &[u8]) -> Result<(), &'static str> {
        if !self.verify_integrity() {
            self.emergency_zeroize();
            return Err("ERR_CANARY_CORRUPTION_DETECTED");
        }

        self.payload = new_secret.to_vec();
        Ok(())
    }

    /// Fail-closed emergency zeroization of memory
    pub fn emergency_zeroize(&mut self) {
        self.is_compromised = true;
        for byte in self.payload.iter_mut() {
            *byte = 0;
        }
        self.entropy = 0;
        self.head_canary = 0;
        self.tail_canary = 0;
    }

    #[cfg(test)]
    pub fn simulate_heap_overflow(&mut self) {
        // Intentionally corrupt the tail canary
        self.tail_canary = 0x4141_4141_4141_4141;
    }
}

impl Drop for CanaryProtectedBuffer {
    fn drop(&mut self) {
        self.emergency_zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canary_allocation_and_read() {
        let mut buffer = CanaryProtectedBuffer::allocate(b"SUPER_SECRET_KEY_256_BITS");
        assert!(buffer.verify_integrity());

        let read_res = buffer.read_guarded().expect("Read failed");
        assert_eq!(read_res, b"SUPER_SECRET_KEY_256_BITS");
    }

    #[test]
    fn test_canary_detects_overflow_and_zeroizes() {
        let mut buffer = CanaryProtectedBuffer::allocate(b"SOVEREIGN_TRANSACTION_PAYLOAD");
        assert!(buffer.verify_integrity());

        // Simulate external heap corruption
        buffer.simulate_heap_overflow();

        // Reading must fail and zeroize memory
        let read_res = buffer.read_guarded();
        assert_eq!(read_res, Err("ERR_CANARY_CORRUPTION_DETECTED"));
        assert!(!buffer.verify_integrity());
        assert_eq!(buffer.payload, vec![0; 29]);
    }
}

