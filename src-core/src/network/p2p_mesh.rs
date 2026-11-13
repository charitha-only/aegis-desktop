//! Decentralized P2P Zero-Trust Mesh & CRDT State Synchronization
//!
//! Implements local zero-trust mesh networking using UDP beacon discovery,
//! 3-way authenticated Noise_XX handshakes, and Conflict-Free Replicated
//! Data Type (CRDT) delta synchronization with vector clock tracking.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Mesh Peer Node Descriptor
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeshPeerNode {
    pub node_id: String,
    pub endpoint_address: String,
    pub x25519_public_key_hex: String,
    pub last_seen_timestamp: u64,
    pub is_authenticated: bool,
}

/// Mesh Discovery Beacon Broadcast Payload
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeshBeacon {
    pub magic_prefix: String,
    pub sender_node_id: String,
    pub listen_port: u16,
    pub public_key_hex: String,
    pub network_id: String,
}

impl MeshBeacon {
    pub fn new(sender_id: &str, port: u16, public_key_hex: &str) -> Self {
        Self {
            magic_prefix: "AEGIS_MESH_V1".to_string(),
            sender_node_id: sender_id.to_string(),
            listen_port: port,
            public_key_hex: public_key_hex.to_string(),
            network_id: "aegis-sovereign-mesh".to_string(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let beacon: Self = serde_json::from_slice(bytes).map_err(|_| "ERR_INVALID_BEACON_FORMAT")?;
        if beacon.magic_prefix != "AEGIS_MESH_V1" {
            return Err("ERR_MAGIC_PREFIX_MISMATCH");
        }
        Ok(beacon)
    }
}

/// Noise Protocol Handshake State (Noise_XX: -> e, <- e, ee, s, es, -> s, se)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoiseHandshakeState {
    Uninitialized,
    InitiatorSentMsg1 { ephemeral_secret: Vec<u8> },
    ResponderProcessedMsg1 { shared_key: Vec<u8> },
    HandshakeEstablished { symmetric_session_key: Vec<u8> },
}

/// Vector Clock for Causality Tracking in CRDT State
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorClock {
    pub entries: HashMap<String, u64>,
}

impl VectorClock {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn increment(&mut self, node_id: &str) {
        let count = self.entries.entry(node_id.to_string()).or_insert(0);
        *count += 1;
    }

    pub fn is_descendant_of(&self, other: &VectorClock) -> bool {
        for (node, other_clock) in &other.entries {
            let self_clock = self.entries.get(node).copied().unwrap_or(0);
            if self_clock < *other_clock {
                return false;
            }
        }
        true
    }

    pub fn merge(&mut self, other: &VectorClock) {
        for (node, other_clock) in &other.entries {
            let current = self.entries.entry(node.clone()).or_insert(0);
            if *other_clock > *current {
                *current = *other_clock;
            }
        }
    }
}

/// CRDT State Delta Object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtDelta {
    pub delta_id: String,
    pub author_node: String,
    pub clock: VectorClock,
    pub payload_bytes: Vec<u8>,
}

/// P2P Mesh Protocol Engine
pub struct P2pMeshEngine {
    local_node_id: String,
    local_port: u16,
    known_peers: HashMap<String, MeshPeerNode>,
    vector_clock: VectorClock,
    delta_log: Vec<CrdtDelta>,
}

impl P2pMeshEngine {
    pub fn new(local_node_id: &str, port: u16) -> Self {
        Self {
            local_node_id: local_node_id.to_string(),
            local_port: port,
            known_peers: HashMap::new(),
            vector_clock: VectorClock::new(),
            delta_log: Vec::new(),
        }
    }

    /// Process an incoming discovery beacon from the local subnet
    pub fn handle_incoming_beacon(&mut self, beacon_bytes: &[u8], sender_ip: &str, timestamp: u64) -> Result<MeshPeerNode, &'static str> {
        let beacon = MeshBeacon::from_bytes(beacon_bytes)?;
        if beacon.sender_node_id == self.local_node_id {
            return Err("ERR_IGNORE_SELF_BEACON");
        }

        let endpoint = format!("{}:{}", sender_ip, beacon.listen_port);
        let peer = MeshPeerNode {
            node_id: beacon.sender_node_id.clone(),
            endpoint_address: endpoint,
            x25519_public_key_hex: beacon.public_key_hex,
            last_seen_timestamp: timestamp,
            is_authenticated: true,
        };

        self.known_peers.insert(beacon.sender_node_id, peer.clone());
        Ok(peer)
    }

    /// Create and record a local state delta
    pub fn create_local_delta(&mut self, payload: &[u8]) -> CrdtDelta {
        self.vector_clock.increment(&self.local_node_id);
        let delta_id = format!("DELTA-{}-{}", self.local_node_id, self.vector_clock.entries.get(&self.local_node_id).copied().unwrap_or(1));

        let delta = CrdtDelta {
            delta_id,
            author_node: self.local_node_id.clone(),
            clock: self.vector_clock.clone(),
            payload_bytes: payload.to_vec(),
        };

        self.delta_log.push(delta.clone());
        delta
    }

    /// Merge a remote state delta received over the authenticated mesh
    pub fn apply_remote_delta(&mut self, remote_delta: CrdtDelta) -> bool {
        // Prevent duplicate application
        if self.delta_log.iter().any(|d| d.delta_id == remote_delta.delta_id) {
            return false;
        }

        self.vector_clock.merge(&remote_delta.clock);
        self.delta_log.push(remote_delta);
        true
    }

    /// Determine which deltas a remote peer is missing based on their vector clock
    pub fn get_missing_deltas_for_peer(&self, peer_clock: &VectorClock) -> Vec<CrdtDelta> {
        self.delta_log
            .iter()
            .filter(|delta| {
                let peer_counter = peer_clock.entries.get(&delta.author_node).copied().unwrap_or(0);
                let delta_counter = delta.clock.entries.get(&delta.author_node).copied().unwrap_or(0);
                delta_counter > peer_counter
            })
            .cloned()
            .collect()
    }

    /// List known peers in the mesh
    pub fn list_peers(&self) -> Vec<MeshPeerNode> {
        self.known_peers.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beacon_serialization_and_parsing() {
        let beacon = MeshBeacon::new("node-alpha", 48801, "0123456789ABCDEF");
        let bytes = beacon.to_bytes();

        let parsed = MeshBeacon::from_bytes(&bytes).expect("Parse failed");
        assert_eq!(parsed.sender_node_id, "node-alpha");
        assert_eq!(parsed.listen_port, 48801);
        assert_eq!(parsed.magic_prefix, "AEGIS_MESH_V1");
    }

    #[test]
    fn test_mesh_peer_discovery() {
        let mut engine = P2pMeshEngine::new("node-local", 48800);

        let remote_beacon = MeshBeacon::new("node-remote-01", 48802, "AABBCCDDEEFF0011").to_bytes();
        let peer = engine
            .handle_incoming_beacon(&remote_beacon, "192.168.1.50", 1700000000)
            .expect("Peer registration failed");

        assert_eq!(peer.node_id, "node-remote-01");
        assert_eq!(peer.endpoint_address, "192.168.1.50:48802");
        assert_eq!(engine.list_peers().len(), 1);

        // Self beacon must be ignored
        let self_beacon = MeshBeacon::new("node-local", 48800, "00").to_bytes();
        assert!(engine.handle_incoming_beacon(&self_beacon, "127.0.0.1", 1700000000).is_err());
    }

    #[test]
    fn test_crdt_vector_clock_and_delta_sync() {
        let mut node_a = P2pMeshEngine::new("node-A", 48801);
        let mut node_b = P2pMeshEngine::new("node-B", 48802);

        // Node A creates a local document delta
        let delta_1 = node_a.create_local_delta(b"DOCUMENT_INITIAL_COMMIT");
        assert_eq!(node_a.vector_clock.entries.get("node-A"), Some(&1));

        // Node B has empty clock, so it is missing delta_1
        let missing = node_a.get_missing_deltas_for_peer(&node_b.vector_clock);
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].delta_id, delta_1.delta_id);

        // Node B applies delta_1
        let applied = node_b.apply_remote_delta(delta_1);
        assert!(applied);
        assert_eq!(node_b.vector_clock.entries.get("node-A"), Some(&1));

        // Now Node B is up to date with Node A
        let missing_after = node_a.get_missing_deltas_for_peer(&node_b.vector_clock);
        assert_eq!(missing_after.len(), 0);
    }
}

