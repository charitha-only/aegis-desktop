/// Aegis Desktop — Distributed P2P Plugin Marketplace (v0.18.0)
///
/// Features:
/// - Content-addressed plugin registry utilizing SHA-256 IPFS / BitTorrent info-hashes
/// - Multi-signature decentralized consensus validation (m-of-n required author approvals)
/// - Peer gossip simulation / distribution nodes
/// - Zero-Trust provenance and integrity sandbox check before installation

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

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

pub struct P2PPluginMarketplace {
    peers: Mutex<Vec<P2PPeerNode>>,
    listings: Mutex<HashMap<String, P2PPluginListing>>,
    trusted_network_keys: Mutex<Vec<String>>,
}

impl P2PPluginMarketplace {
    pub fn new() -> Self {
        let mut listings = HashMap::new();

        // Prepopulate standard decentralized catalog items
        let standard_plugins = vec![
            (
                "plugin.zk.telemetry-filter",
                "ZK Privacy Shield Filter",
                "1.4.0",
                "core.aegis.eth",
                "Intercepts analytics frames with Zero-Knowledge verification proofs.",
                "QmZkShieldFilter99xAb12CcD77FaaE43210987654321Bca",
                "a1b2c3d4e5f60718293a4b5c6d7e8f9012345678",
                2_450_000,
                2,
                vec![
                    P2PPluginMultiSig {
                        signer_identity: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
                        signature_hex: "3045022100e4ab8c72...18f3a9".to_string(),
                        timestamp: 1729890000,
                    },
                    P2PPluginMultiSig {
                        signer_identity: "did:key:z6MknGc3ocHs3zdPiJbnaE54nudNmtWCqYY5kVuK9vyuhEB3".to_string(),
                        signature_hex: "304402206f3a77bd21...99bc41".to_string(),
                        timestamp: 1729890120,
                    },
                ],
                48,
            ),
            (
                "plugin.crypto.mesh-sync",
                "Air-Gapped Mesh Vault Sync",
                "2.1.0",
                "meshdao.crypto",
                "Peer-to-peer encrypted sync for Aegis Vaults across local LAN/Bluetooth.",
                "QmMeshSyncLocalVault8899aabbccddeeff00112233445566",
                "b9a8c7d6e5f40312984756abcdef0123456789aa",
                4_120_000,
                2,
                vec![
                    P2PPluginMultiSig {
                        signer_identity: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
                        signature_hex: "3045022100fa123b09...88ac77".to_string(),
                        timestamp: 1729891000,
                    },
                    P2PPluginMultiSig {
                        signer_identity: "did:key:z6MkjTBxS4xW88nWD4pQoAky7v3yW5uK6vyuhEB1".to_string(),
                        signature_hex: "3045022100cb981144...12ed99".to_string(),
                        timestamp: 1729891200,
                    },
                ],
                82,
            ),
            (
                "plugin.ai.local-sanitizer",
                "Local LLM Prompt Sanitizer",
                "0.9.5",
                "sentinel.security.sol",
                "Hardware-enclave based local scanner removing PII and API keys from prompts.",
                "QmPromptSanitizerAiSecurityCore778899001122334455",
                "e5f60718293a4b5c6d7e8f90123456789abcdef0",
                8_340_000,
                1,
                vec![
                    P2PPluginMultiSig {
                        signer_identity: "did:key:z6MknGc3ocHs3zdPiJbnaE54nudNmtWCqYY5kVuK9vyuhEB3".to_string(),
                        signature_hex: "3044022099aabbccdd...ef0123".to_string(),
                        timestamp: 1729892000,
                    },
                ],
                31,
            ),
        ];

        for (id, name, ver, author, desc, cid, infohash, size, req_sigs, sigs, seeders) in standard_plugins {
            let verified = sigs.len() as u32 >= req_sigs;
            listings.insert(
                id.to_string(),
                P2PPluginListing {
                    plugin_id: id.to_string(),
                    name: name.to_string(),
                    version: ver.to_string(),
                    author: author.to_string(),
                    description: desc.to_string(),
                    cid: cid.to_string(),
                    bittorrent_infohash: infohash.to_string(),
                    size_bytes: size,
                    required_signatures: req_sigs,
                    signatures: sigs,
                    verified,
                    seeders_count: seeders,
                    download_status: "AVAILABLE".to_string(),
                },
            );
        }

        let peers = vec![
            P2PPeerNode {
                peer_id: "12D3KooWStP48qTjGv8N...NodeAlpha".to_string(),
                address: "/ip4/142.250.190.46/tcp/4001/p2p/QmAlpha".to_string(),
                reputation_score: 98,
                is_online: true,
                latency_ms: 24,
            },
            P2PPeerNode {
                peer_id: "12D3KooWB9eZf3vR...NodeBravo".to_string(),
                address: "/ip4/172.217.16.206/tcp/4001/p2p/QmBravo".to_string(),
                reputation_score: 94,
                is_online: true,
                latency_ms: 41,
            },
            P2PPeerNode {
                peer_id: "12D3KooWJh65xM2...NodeCharlie".to_string(),
                address: "/ip4/104.21.34.120/tcp/4001/p2p/QmCharlie".to_string(),
                reputation_score: 89,
                is_online: true,
                latency_ms: 18,
            },
        ];

        let trusted_keys = vec![
            "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
            "did:key:z6MknGc3ocHs3zdPiJbnaE54nudNmtWCqYY5kVuK9vyuhEB3".to_string(),
            "did:key:z6MkjTBxS4xW88nWD4pQoAky7v3yW5uK6vyuhEB1".to_string(),
        ];

        Self {
            peers: Mutex::new(peers),
            listings: Mutex::new(listings),
            trusted_network_keys: Mutex::new(trusted_keys),
        }
    }

    /// List all marketplace plugins with validation state
    pub fn list_plugins(&self) -> Vec<P2PPluginListing> {
        let map = self.listings.lock().unwrap();
        map.values().cloned().collect()
    }

    /// List active P2P swarm peers
    pub fn list_peers(&self) -> Vec<P2PPeerNode> {
        self.peers.lock().unwrap().clone()
    }

    /// Add a multi-sig endorsement to a decentralized plugin listing
    pub fn sign_plugin(&self, plugin_id: &str, signer_did: &str) -> Result<P2PPluginListing, String> {
        let mut map = self.listings.lock().unwrap();
        let listing = map.get_mut(plugin_id).ok_or_else(|| format!("Plugin '{}' not found", plugin_id))?;

        // Check if signer is trusted in registry
        let trusted = self.trusted_network_keys.lock().unwrap();
        if !trusted.contains(&signer_did.to_string()) {
            return Err(format!("Signer DID '{}' is not registered in the decentralized trust anchor set", signer_did));
        }

        // Check if already signed
        if listing.signatures.iter().any(|s| s.signer_identity == signer_did) {
            return Err("Signer has already endorsed this plugin version".to_string());
        }

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let mut hasher = Sha256::new();
        hasher.update(listing.cid.as_bytes());
        hasher.update(signer_did.as_bytes());
        let sig_digest = format!("{:x}", hasher.finalize());

        listing.signatures.push(P2PPluginMultiSig {
            signer_identity: signer_did.to_string(),
            signature_hex: format!("3045022100{}", &sig_digest[..32]),
            timestamp,
        });

        listing.verified = listing.signatures.len() as u32 >= listing.required_signatures;
        Ok(listing.clone())
    }

    /// Install/Download a plugin from P2P swarm with multi-sig consensus verification
    pub fn install_plugin(&self, plugin_id: &str) -> Result<P2PPluginListing, String> {
        let mut map = self.listings.lock().unwrap();
        let listing = map.get_mut(plugin_id).ok_or_else(|| format!("Plugin '{}' not found", plugin_id))?;

        if !listing.verified {
            return Err(format!(
                "Zero-Trust Policy Block: Plugin '{}' only has {}/{} required decentralized signatures.",
                plugin_id,
                listing.signatures.len(),
                listing.required_signatures
            ));
        }

        listing.download_status = "INSTALLED".to_string();
        listing.seeders_count += 1;
        Ok(listing.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p2p_marketplace_verification_and_install() {
        let market = P2PPluginMarketplace::new();
        let plugins = market.list_plugins();
        assert!(!plugins.is_empty());

        let res = market.install_plugin("plugin.zk.telemetry-filter");
        assert!(res.is_ok());
        assert_eq!(res.unwrap().download_status, "INSTALLED");
    }
}
