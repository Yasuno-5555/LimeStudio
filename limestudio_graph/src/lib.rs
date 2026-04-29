//! LimeGraph: The Unified Nervous System.
//! This is the Single Source of Truth for both DSP and UI.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub type NodeId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimeGraph {
    pub nodes: HashMap<NodeId, LimeNode>,
    pub edges: Vec<LimeEdge>,
    pub metadata: GraphMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimeNode {
    pub id: NodeId,
    pub kind: String,

    // --- Components ---
    pub dsp: Option<DspComponent>,
    pub ui: Option<UiComponent>,
    pub forensic: Option<ForensicComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DspComponent {
    pub params: HashMap<String, f32>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiComponent {
    pub position: [f32; 2],
    pub label: String,
    pub color: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForensicComponent {
    pub last_modified_by: String,
    pub timestamp: u64,
    pub hash_chain: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimeEdge {
    pub from: NodeId,
    pub to: NodeId,
    pub from_port: String,
    pub to_port: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphMetadata {
    pub project_name: String,
    pub version: u32,
    pub total_provenance_hash: [u8; 32],
}

impl Default for LimeGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl LimeGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            metadata: GraphMetadata::default(),
        }
    }

    pub fn add_node(&mut self, kind: &str, pos: [f32; 2]) -> NodeId {
        let id = Uuid::new_v4();
        let node = LimeNode {
            id,
            kind: kind.to_string(),
            dsp: Some(DspComponent {
                params: HashMap::new(),
                is_active: true,
            }),
            ui: Some(UiComponent {
                position: pos,
                label: kind.to_string(),
                color: [0.5, 0.5, 0.5, 1.0],
            }),
            forensic: Some(ForensicComponent {
                last_modified_by: "Squeezer".to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                hash_chain: Vec::new(),
            }),
        };
        self.nodes.insert(id, node);
        id
    }
}
