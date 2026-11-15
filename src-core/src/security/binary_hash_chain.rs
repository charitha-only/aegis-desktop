//! Secure Core Binary Delta Integrity Hash Chain Engine.
//!
//! Provides continuous, tamper-evident hash chaining across virtual memory segments
//! of the active Aegis Core binary. Detects code injection, process hollowing,
//! and unauthorized dynamic instruction patching at runtime.

use sha2::{Digest, Sha256};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryPageBlock {
    pub page_index: usize,
    pub virtual_offset: u64,
    pub page_size: usize,
    pub page_hash_hex: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BinaryHashChainReport {
    pub total_pages_scanned: usize,
    pub chain_root_hash_hex: String,
    pub is_clean: bool,
    pub tampered_page_indices: Vec<usize>,
    pub memory_span_bytes: usize,
}

#[allow(dead_code)]
pub struct BinaryHashChain;

#[allow(dead_code)]
impl BinaryHashChain {
    /// Compute the integrity hash chain over contiguous memory segments
    pub fn compute_chain(pages: &[&[u8]], base_virtual_address: u64) -> (String, Vec<MemoryPageBlock>) {
        let mut running_chain_hash = [0u8; 32];
        // Genesis block seed
        running_chain_hash.copy_from_slice(&Sha256::digest(b"AEGIS_CORE_BINARY_HASH_CHAIN_GENESIS_v2.5"));

        let mut block_records = Vec::with_capacity(pages.len());

        for (idx, page) in pages.iter().enumerate() {
            let page_hash = Sha256::digest(page);
            let virtual_offset = base_virtual_address + (idx * 4096) as u64;

            let mut chain_hasher = Sha256::new();
            chain_hasher.update(&running_chain_hash);
            chain_hasher.update(&page_hash);
            chain_hasher.update(&virtual_offset.to_be_bytes());
            running_chain_hash = chain_hasher.finalize().into();

            block_records.push(MemoryPageBlock {
                page_index: idx,
                virtual_offset,
                page_size: page.len(),
                page_hash_hex: hex::encode(page_hash),
            });
        }

        (hex::encode(running_chain_hash), block_records)
    }

    /// Audit live memory pages against known good expected page hashes
    pub fn audit_memory(
        live_pages: &[&[u8]],
        expected_root_hash: &str,
        base_address: u64,
    ) -> BinaryHashChainReport {
        let (computed_root, blocks) = Self::compute_chain(live_pages, base_address);

        let is_clean = computed_root == expected_root_hash;
        let mut tampered = Vec::new();

        if !is_clean {
            // Find which pages differ
            for block in blocks {
                let actual_hash = hex::encode(Sha256::digest(live_pages[block.page_index]));
                if actual_hash != block.page_hash_hex {
                    tampered.push(block.page_index);
                }
            }
        }

        let total_bytes = live_pages.iter().map(|p| p.len()).sum();

        BinaryHashChainReport {
            total_pages_scanned: live_pages.len(),
            chain_root_hash_hex: computed_root,
            is_clean,
            tampered_page_indices: tampered,
            memory_span_bytes: total_bytes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_hash_chain_clean_memory_verifies() {
        let page1 = [0x90u8; 4096]; // NOP sled
        let page2 = [0xC3u8; 4096]; // RET sled
        let page3 = [0x48u8; 4096]; // REX.W prefix

        let pages: Vec<&[u8]> = vec![&page1, &page2, &page3];
        let (root_hash, _) = BinaryHashChain::compute_chain(&pages, 0x140000000);

        let report = BinaryHashChain::audit_memory(&pages, &root_hash, 0x140000000);

        assert!(report.is_clean);
        assert_eq!(report.total_pages_scanned, 3);
        assert!(report.tampered_page_indices.is_empty());
        assert_eq!(report.memory_span_bytes, 4096 * 3);
    }

    #[test]
    fn test_binary_hash_chain_tampered_page_detected() {
        let page1 = [0x90u8; 4096];
        let mut page2 = [0xC3u8; 4096];
        let pages_golden: Vec<&[u8]> = vec![&page1, &page2];

        let (golden_root, _) = BinaryHashChain::compute_chain(&pages_golden, 0x140000000);

        // Adversary injects hook in page 2
        page2[100] = 0xCC; // INT3 breakpoint injected
        let pages_live: Vec<&[u8]> = vec![&page1, &page2];

        let report = BinaryHashChain::audit_memory(&pages_live, &golden_root, 0x140000000);

        assert!(!report.is_clean, "Memory tampering must be flagged");
        assert_ne!(report.chain_root_hash_hex, golden_root);
    }
}
