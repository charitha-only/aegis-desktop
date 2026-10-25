//! Unavailable integration: fails closed until a real provider is implemented.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PPeerNode {
    pub peer_id: String,
    pub address: String,
    pub reputation_score: u32,
    pub is_online: bool,
    pub latency_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PPluginMultiSig {
    pub signer_identity: String,
    pub signature_hex: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PPluginListing {
    pub plugin_id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub cid: String, // IPFS Content Identifier
    pub bittorrent_infohash: String,
    pub size_bytes: u64,
    pub required_signatures: u32,
    pub signatures: Vec<P2PPluginMultiSig>,
    pub verified: bool,
    pub seeders_count: u32,
    pub download_status: String, // "AVAILABLE", "SEEDING", "INSTALLED"
}

pub struct P2PPluginMarketplace;
impl P2PPluginMarketplace {
    pub fn new() -> Self {
        Self
    }
    pub fn list_plugins(&self) -> Vec<P2PPluginListing> {
        vec![]
    }
    pub fn list_peers(&self) -> Vec<P2PPeerNode> {
        vec![]
    }
    pub fn sign_plugin(&self, _id: &str, _signer: &str) -> Result<P2PPluginListing, String> {
        Err("UNAVAILABLE: P2P transport and publisher key verification are not configured".into())
    }
    pub fn install_plugin(&self, _id: &str) -> Result<P2PPluginListing, String> {
        Err("UNAVAILABLE: No verified plugin package was downloaded or installed".into())
    }
}
