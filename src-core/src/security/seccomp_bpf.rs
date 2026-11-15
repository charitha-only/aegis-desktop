//! Linux BPF / Seccomp Syscall Filter Synthesizer (cBPF / eBPF)
//!
//! Generates strict kernel-level Seccomp-BPF bytecode filter rules restricting child
//! worker processes and sandboxed plugins to essential standard I/O and IPC syscalls.
//! Denies network egress, child process forking, ptrace inspection, and arbitrary execution.

use serde::{Deserialize, Serialize};

pub const AUDIT_ARCH_X86_64: u32 = 0xC000003E;
#[allow(dead_code)]
pub const AUDIT_ARCH_AARCH64: u32 = 0xC00000B7;

// BPF Instruction Classes & Fields
pub const BPF_LD: u16 = 0x00;
pub const BPF_W: u16 = 0x00;
pub const BPF_ABS: u16 = 0x20;
pub const BPF_JMP: u16 = 0x05;
pub const BPF_JEQ: u16 = 0x10;
pub const BPF_K: u16 = 0x00;
pub const BPF_RET: u16 = 0x06;

// Seccomp Return Actions
pub const SECCOMP_RET_KILL_PROCESS: u32 = 0x80000000;
#[allow(dead_code)]
pub const SECCOMP_RET_TRAP: u32 = 0x00030000;
#[allow(dead_code)]
pub const SECCOMP_RET_ERRNO: u32 = 0x00050001; // EPERM = 1
pub const SECCOMP_RET_ALLOW: u32 = 0x7FFF0000;

// Standard Linux x86_64 Syscall Numbers
pub const SYS_READ: u32 = 0;
pub const SYS_WRITE: u32 = 1;
pub const SYS_CLOSE: u32 = 3;
pub const SYS_POLL: u32 = 7;
pub const SYS_NANOSLEEP: u32 = 35;
pub const SYS_FUTEX: u32 = 202;
pub const SYS_EXIT_GROUP: u32 = 231;
pub const SYS_EPOLL_WAIT: u32 = 232;
pub const SYS_RESTART_SYSCALL: u32 = 219;

// Forbidden Syscalls
pub const SYS_FORK: u32 = 57;
pub const SYS_EXECVE: u32 = 59;
pub const SYS_PTRACE: u32 = 101;
pub const SYS_SOCKET: u32 = 41;
pub const SYS_CONNECT: u32 = 42;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SockFilter {
    pub code: u16,
    pub jt: u8,
    pub jf: u8,
    pub k: u32,
}

impl SockFilter {
    pub fn stmt(code: u16, k: u32) -> Self {
        Self { code, jt: 0, jf: 0, k }
    }

    pub fn jump(code: u16, k: u32, jt: u8, jf: u8) -> Self {
        Self { code, jt, jf, k }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeccompPolicyReport {
    pub architecture: String,
    pub total_filter_instructions: usize,
    pub allowed_syscalls: Vec<u32>,
    pub blocked_syscalls: Vec<u32>,
    pub default_action: String,
    pub filter_hash: String,
}

pub struct SeccompBpfSynthesizer;

impl SeccompBpfSynthesizer {
    /// Generates strict cBPF instruction program for x86_64 Linux kernel
    pub fn build_strict_isolation_filter(custom_allowlist: Option<Vec<u32>>) -> Vec<SockFilter> {
        let allowed = custom_allowlist.unwrap_or_else(|| vec![
            SYS_READ,
            SYS_WRITE,
            SYS_CLOSE,
            SYS_POLL,
            SYS_NANOSLEEP,
            SYS_FUTEX,
            SYS_EXIT_GROUP,
            SYS_EPOLL_WAIT,
            SYS_RESTART_SYSCALL,
        ]);

        let mut program = Vec::new();

        // 1. Validate Architecture: Load arch at offset 4 of seccomp_data (arch field)
        program.push(SockFilter::stmt(BPF_LD | BPF_W | BPF_ABS, 4));
        // Jump if Equal to AUDIT_ARCH_X86_64, continue (jt=1, jf=0) else fail
        program.push(SockFilter::jump(BPF_JMP | BPF_JEQ | BPF_K, AUDIT_ARCH_X86_64, 1, 0));
        // Kill process if architecture mismatch
        program.push(SockFilter::stmt(BPF_RET | BPF_K, SECCOMP_RET_KILL_PROCESS));

        // 2. Load Syscall Number at offset 0 of seccomp_data (nr field)
        program.push(SockFilter::stmt(BPF_LD | BPF_W | BPF_ABS, 0));

        // 3. Syscall Jump Table
        let num_allowed = allowed.len();
        for (i, &syscall_nr) in allowed.iter().enumerate() {
            // If match, jump to ALLOW instruction; else jump to next check
            let jump_to_allow = (num_allowed - i) as u8;
            program.push(SockFilter::jump(BPF_JMP | BPF_JEQ | BPF_K, syscall_nr, jump_to_allow, 0));
        }

        // 4. Default Fail-Closed Fallthrough: Kill process
        program.push(SockFilter::stmt(BPF_RET | BPF_K, SECCOMP_RET_KILL_PROCESS));

        // 5. Allow Destination
        program.push(SockFilter::stmt(BPF_RET | BPF_K, SECCOMP_RET_ALLOW));

        program
    }

    /// Evaluates the BPF instruction program against simulated seccomp_data
    #[allow(dead_code)]
    pub fn simulate_filter(program: &[SockFilter], arch: u32, syscall_nr: u32) -> u32 {
        let mut pc = 0;
        let mut acc = 0u32;

        while pc < program.len() {
            let inst = &program[pc];
            match inst.code {
                0x20 => { // BPF_LD | BPF_W | BPF_ABS
                    if inst.k == 4 {
                        acc = arch;
                    } else if inst.k == 0 {
                        acc = syscall_nr;
                    }
                    pc += 1;
                }
                0x15 => { // BPF_JMP | BPF_JEQ | BPF_K
                    if acc == inst.k {
                        pc += 1 + inst.jt as usize;
                    } else {
                        pc += 1 + inst.jf as usize;
                    }
                }
                0x06 => { // BPF_RET | BPF_K
                    return inst.k;
                }
                _ => {
                    return SECCOMP_RET_KILL_PROCESS;
                }
            }
        }
        SECCOMP_RET_KILL_PROCESS
    }

    pub fn generate_policy_report() -> SeccompPolicyReport {
        let filter = Self::build_strict_isolation_filter(None);
        let allowed = vec![
            SYS_READ, SYS_WRITE, SYS_CLOSE, SYS_POLL,
            SYS_NANOSLEEP, SYS_FUTEX, SYS_EXIT_GROUP,
            SYS_EPOLL_WAIT, SYS_RESTART_SYSCALL,
        ];
        let blocked = vec![SYS_FORK, SYS_EXECVE, SYS_PTRACE, SYS_SOCKET, SYS_CONNECT];

        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        for inst in &filter {
            hasher.update(&inst.code.to_le_bytes());
            hasher.update(&[inst.jt, inst.jf]);
            hasher.update(&inst.k.to_le_bytes());
        }
        let filter_hash = hex::encode(hasher.finalize());

        SeccompPolicyReport {
            architecture: "Linux x86_64 (AUDIT_ARCH_X86_64)".to_string(),
            total_filter_instructions: filter.len(),
            allowed_syscalls: allowed,
            blocked_syscalls: blocked,
            default_action: "SECCOMP_RET_KILL_PROCESS (Fail-Closed)".to_string(),
            filter_hash,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seccomp_filter_generation_and_arch_validation() {
        let filter = SeccompBpfSynthesizer::build_strict_isolation_filter(None);
        assert!(!filter.is_empty());

        // Architecture mismatch must trigger kill
        let result = SeccompBpfSynthesizer::simulate_filter(&filter, AUDIT_ARCH_AARCH64, SYS_READ);
        assert_eq!(result, SECCOMP_RET_KILL_PROCESS);
    }

    #[test]
    fn test_seccomp_filter_allows_whitelisted_syscalls() {
        let filter = SeccompBpfSynthesizer::build_strict_isolation_filter(None);

        assert_eq!(SeccompBpfSynthesizer::simulate_filter(&filter, AUDIT_ARCH_X86_64, SYS_READ), SECCOMP_RET_ALLOW);
        assert_eq!(SeccompBpfSynthesizer::simulate_filter(&filter, AUDIT_ARCH_X86_64, SYS_WRITE), SECCOMP_RET_ALLOW);
        assert_eq!(SeccompBpfSynthesizer::simulate_filter(&filter, AUDIT_ARCH_X86_64, SYS_CLOSE), SECCOMP_RET_ALLOW);
        assert_eq!(SeccompBpfSynthesizer::simulate_filter(&filter, AUDIT_ARCH_X86_64, SYS_FUTEX), SECCOMP_RET_ALLOW);
        assert_eq!(SeccompBpfSynthesizer::simulate_filter(&filter, AUDIT_ARCH_X86_64, SYS_EXIT_GROUP), SECCOMP_RET_ALLOW);
    }

    #[test]
    fn test_seccomp_filter_blocks_hostile_syscalls() {
        let filter = SeccompBpfSynthesizer::build_strict_isolation_filter(None);

        assert_eq!(SeccompBpfSynthesizer::simulate_filter(&filter, AUDIT_ARCH_X86_64, SYS_EXECVE), SECCOMP_RET_KILL_PROCESS);
        assert_eq!(SeccompBpfSynthesizer::simulate_filter(&filter, AUDIT_ARCH_X86_64, SYS_FORK), SECCOMP_RET_KILL_PROCESS);
        assert_eq!(SeccompBpfSynthesizer::simulate_filter(&filter, AUDIT_ARCH_X86_64, SYS_PTRACE), SECCOMP_RET_KILL_PROCESS);
        assert_eq!(SeccompBpfSynthesizer::simulate_filter(&filter, AUDIT_ARCH_X86_64, SYS_SOCKET), SECCOMP_RET_KILL_PROCESS);
        assert_eq!(SeccompBpfSynthesizer::simulate_filter(&filter, AUDIT_ARCH_X86_64, SYS_CONNECT), SECCOMP_RET_KILL_PROCESS);
    }

    #[test]
    fn test_seccomp_policy_report_integrity() {
        let report = SeccompBpfSynthesizer::generate_policy_report();
        assert_eq!(report.total_filter_instructions, 15);
        assert_eq!(report.allowed_syscalls.len(), 9);
        assert_eq!(report.blocked_syscalls.len(), 5);
        assert_eq!(report.filter_hash.len(), 64);
    }
}
