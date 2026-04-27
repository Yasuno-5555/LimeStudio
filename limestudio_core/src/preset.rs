use serde::{Serialize, Deserialize};
use dirtydata_core::ir::Graph;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PresetArtifact {
    pub name: String,
    pub version: String,
    pub graph: Graph,
    pub parameters: HashMap<String, f32>,
    pub metadata: HashMap<String, String>,
    pub hash: String, // blake3 hash of (graph + parameters)
    pub source_hash: Option<String>, // blake3 hash of the Rust source (Visible Compiler)
}

impl PresetArtifact {
    pub fn new(name: String, graph: Graph, parameters: HashMap<String, f32>, source_code: Option<&str>) -> Self {
        let mut artifact = Self {
            name,
            version: "0.1.0".to_string(),
            graph,
            parameters,
            metadata: HashMap::new(),
            hash: String::new(),
            source_hash: source_code.map(|c| blake3::hash(c.as_bytes()).to_string()),
        };
        artifact.hash = artifact.calculate_hash();
        artifact
    }

    pub fn calculate_hash(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        // Stable serialization for hashing
        let graph_json = serde_json::to_string(&self.graph).unwrap();
        let mut param_keys: Vec<_> = self.parameters.keys().collect();
        param_keys.sort();
        
        hasher.update(graph_json.as_bytes());
        for key in param_keys {
            hasher.update(key.as_bytes());
            hasher.update(&self.parameters.get(key).unwrap().to_le_bytes());
        }
        
        hasher.finalize().to_string()
    }

    pub fn verify(&self) -> bool {
        self.hash == self.calculate_hash()
    }
}
