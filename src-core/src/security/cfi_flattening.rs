//! Control Flow Integrity (CFI) & Shadow Stack Protection Engine.
//!
//! Enforces forward-edge and backward-edge control flow integrity across security-critical
//! operations. Uses a hardware-style shadow stack and state-transition graph to
//! detect Return-Oriented Programming (ROP) and Jump-Oriented Programming (JOP) attacks.

use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExecutionPhase {
    Initialized,
    VaultUnlocked,
    HardwareAttested,
    TransactionSigned,
    Completed,
    Halted,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShadowStackFrame {
    pub call_id: u32,
    pub caller_token_hex: String,
    pub expected_return_phase: ExecutionPhase,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CfiStatusReport {
    pub current_phase: ExecutionPhase,
    pub stack_depth: usize,
    pub is_intact: bool,
    pub violations_detected: usize,
    pub status_message: String,
}

#[allow(dead_code)]
pub struct CfiSupervisor {
    pub current_phase: ExecutionPhase,
    shadow_stack: Vec<ShadowStackFrame>,
    violations: usize,
}

#[allow(dead_code)]
impl CfiSupervisor {
    pub fn new() -> Self {
        Self {
            current_phase: ExecutionPhase::Initialized,
            shadow_stack: Vec::new(),
            violations: 0,
        }
    }

    /// Helper to generate a tamper-evident call token
    fn generate_token(call_id: u32, phase: ExecutionPhase) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"AEGIS_CFI_SHADOW_FRAME_TOKEN");
        hasher.update(&call_id.to_be_bytes());
        hasher.update(&[phase as u8]);
        hex::encode(hasher.finalize())
    }

    /// Push frame onto shadow stack for a function call
    pub fn push_call(&mut self, call_id: u32, return_phase: ExecutionPhase) {
        let token = Self::generate_token(call_id, return_phase);
        self.shadow_stack.push(ShadowStackFrame {
            call_id,
            caller_token_hex: token,
            expected_return_phase: return_phase,
        });
    }

    /// Pop and verify frame on function return (backward-edge CFI)
    pub fn pop_return(&mut self, call_id: u32) -> Result<ExecutionPhase, String> {
        let frame = match self.shadow_stack.pop() {
            Some(f) => f,
            None => {
                self.violations += 1;
                return Err("Shadow stack underflow: unauthorized return sequence".to_string());
            }
        };

        let expected_token = Self::generate_token(call_id, frame.expected_return_phase);
        if frame.call_id != call_id || frame.caller_token_hex != expected_token {
            self.violations += 1;
            self.current_phase = ExecutionPhase::Halted;
            return Err("CFI Violation: Corrupted shadow stack frame or hijacked return address".to_string());
        }

        self.current_phase = frame.expected_return_phase;
        Ok(frame.expected_return_phase)
    }

    /// Verify a state machine phase transition (forward-edge CFI)
    pub fn transition_phase(&mut self, next: ExecutionPhase) -> Result<(), String> {
        let is_valid = match (self.current_phase, next) {
            (ExecutionPhase::Initialized, ExecutionPhase::VaultUnlocked) => true,
            (ExecutionPhase::VaultUnlocked, ExecutionPhase::HardwareAttested) => true,
            (ExecutionPhase::VaultUnlocked, ExecutionPhase::Completed) => true,
            (ExecutionPhase::HardwareAttested, ExecutionPhase::TransactionSigned) => true,
            (ExecutionPhase::TransactionSigned, ExecutionPhase::Completed) => true,
            (_, ExecutionPhase::Halted) => true,
            _ => false,
        };

        if !is_valid {
            self.violations += 1;
            self.current_phase = ExecutionPhase::Halted;
            return Err(format!(
                "Illegal control-flow transition from {:?} to {:?}",
                self.current_phase, next
            ));
        }

        self.current_phase = next;
        Ok(())
    }

    pub fn status(&self) -> CfiStatusReport {
        CfiStatusReport {
            current_phase: self.current_phase,
            stack_depth: self.shadow_stack.len(),
            is_intact: self.violations == 0 && self.current_phase != ExecutionPhase::Halted,
            violations_detected: self.violations,
            status_message: if self.violations == 0 {
                "CONTROL_FLOW_INTEGRITY_VERIFIED".to_string()
            } else {
                "EXECUTION_HIJACK_PREVENTED".to_string()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cfi_valid_control_flow_execution() {
        let mut cfi = CfiSupervisor::new();
        assert_eq!(cfi.current_phase, ExecutionPhase::Initialized);

        // Valid forward-edge transitions
        cfi.transition_phase(ExecutionPhase::VaultUnlocked).unwrap();
        cfi.push_call(101, ExecutionPhase::VaultUnlocked);

        cfi.transition_phase(ExecutionPhase::HardwareAttested).unwrap();
        cfi.transition_phase(ExecutionPhase::TransactionSigned).unwrap();

        // Valid return
        let ret_phase = cfi.pop_return(101).unwrap();
        assert_eq!(ret_phase, ExecutionPhase::VaultUnlocked);

        cfi.transition_phase(ExecutionPhase::Completed).unwrap();
        let status = cfi.status();
        assert!(status.is_intact);
        assert_eq!(status.violations_detected, 0);
    }

    #[test]
    fn test_cfi_illegal_transition_hijack_detected() {
        let mut cfi = CfiSupervisor::new();

        // Attempt to skip straight from Initialized to TransactionSigned (unauthorized shortcut)
        let res = cfi.transition_phase(ExecutionPhase::TransactionSigned);
        assert!(res.is_err());
        assert_eq!(cfi.current_phase, ExecutionPhase::Halted);

        let status = cfi.status();
        assert!(!status.is_intact);
        assert_eq!(status.violations_detected, 1);
        assert_eq!(status.status_message, "EXECUTION_HIJACK_PREVENTED");
    }
}
