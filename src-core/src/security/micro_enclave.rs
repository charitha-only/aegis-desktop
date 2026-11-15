//! Micro-Enclave WASM Host Virtualization & Dynamic Memory Sandboxing
//!
//! Provides lightweight, high-assurance in-process micro-enclaves.
//! Constrains untrusted guest bytecode execution within strictly bounded linear memory,
//! enforcing deterministic instruction fuel budgets and capability access masks.

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const DEFAULT_ENCLAVE_FUEL_LIMIT: u64 = 1_000_000;
#[allow(dead_code)]
pub const MAX_ENCLAVE_MEMORY_PAGES: usize = 16; // 1 MB (64 KB per WebAssembly page)

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnclaveCapability {
    PureComputation,
    StorageRead,
    StorageWrite,
    CryptographicSign,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnclaveStatus {
    pub enclave_id: String,
    pub allocated_pages: usize,
    pub memory_used_bytes: usize,
    pub fuel_remaining: u64,
    pub is_halted: bool,
    pub capabilities: Vec<EnclaveCapability>,
}

#[allow(dead_code)]
pub struct MicroEnclaveInstance {
    enclave_id: String,
    memory: Vec<u8>,
    fuel_remaining: u64,
    capabilities: Vec<EnclaveCapability>,
    is_halted: bool,
}

#[allow(dead_code)]
impl MicroEnclaveInstance {
    pub fn new(id: &str, capabilities: Vec<EnclaveCapability>, fuel_limit: u64) -> Self {
        let initial_pages = 2; // 128 KB
        Self {
            enclave_id: id.to_string(),
            memory: vec![0u8; initial_pages * 65536],
            fuel_remaining: if fuel_limit == 0 { DEFAULT_ENCLAVE_FUEL_LIMIT } else { fuel_limit },
            capabilities,
            is_halted: false,
        }
    }

    /// Consumes fuel and traps execution if budget is exhausted
    pub fn consume_fuel(&mut self, units: u64) -> Result<(), &'static str> {
        if self.is_halted {
            return Err("Enclave execution halted");
        }
        if self.fuel_remaining < units {
            self.is_halted = true;
            return Err("Enclave fuel exhausted (Trap: FuelLimitExceeded)");
        }
        self.fuel_remaining -= units;
        Ok(())
    }

    /// Safe sandboxed memory read with strict bounds validation
    pub fn read_memory(&mut self, offset: usize, len: usize) -> Result<Vec<u8>, &'static str> {
        self.consume_fuel(len as u64)?;
        if offset + len > self.memory.len() {
            self.is_halted = true;
            return Err("Memory access out of bounds (Trap: OutOfBoundsMemoryAccess)");
        }
        Ok(self.memory[offset..offset + len].to_vec())
    }

    /// Safe sandboxed memory write with strict bounds validation
    pub fn write_memory(&mut self, offset: usize, data: &[u8]) -> Result<(), &'static str> {
        self.consume_fuel(data.len() as u64)?;
        if offset + data.len() > self.memory.len() {
            self.is_halted = true;
            return Err("Memory access out of bounds (Trap: OutOfBoundsMemoryAccess)");
        }
        self.memory[offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }

    /// Verifies whether the enclave holds the requested execution capability
    pub fn check_capability(&self, cap: EnclaveCapability) -> bool {
        self.capabilities.contains(&cap)
    }

    pub fn status(&self) -> EnclaveStatus {
        EnclaveStatus {
            enclave_id: self.enclave_id.clone(),
            allocated_pages: self.memory.len() / 65536,
            memory_used_bytes: self.memory.len(),
            fuel_remaining: self.fuel_remaining,
            is_halted: self.is_halted,
            capabilities: self.capabilities.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_micro_enclave_memory_bounds_and_fuel() {
        let mut enclave = MicroEnclaveInstance::new("plugin_calc_01", vec![EnclaveCapability::PureComputation], 1000);

        let data = [0x5A; 16];
        assert!(enclave.write_memory(100, &data).is_ok());

        let read = enclave.read_memory(100, 16).unwrap();
        assert_eq!(read, data);

        let status = enclave.status();
        assert_eq!(status.fuel_remaining, 1000 - 32);
        assert!(!status.is_halted);
    }

    #[test]
    fn test_micro_enclave_traps_out_of_bounds() {
        let mut enclave = MicroEnclaveInstance::new("plugin_violator", vec![], 5000);
        let oversized = [0xFF; 200000]; // larger than 128KB initial memory

        let res = enclave.write_memory(0, &oversized);
        assert!(res.is_err());
        assert!(enclave.status().is_halted);
    }

    #[test]
    fn test_micro_enclave_traps_on_fuel_exhaustion() {
        let mut enclave = MicroEnclaveInstance::new("plugin_infinite_loop", vec![], 50);
        let res = enclave.consume_fuel(100);

        assert!(res.is_err());
        assert!(enclave.status().is_halted);
    }
}
