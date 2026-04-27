use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectTopology {
    pub name: String,
    pub nodes: HashMap<String, NodeConfig>,
    pub edges: Vec<EdgeConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeConfig {
    pub kind: String,
    pub position: [f32; 2],
    pub params: HashMap<String, f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EdgeConfig {
    pub from: String,
    pub from_port: String,
    pub to: String,
    pub to_port: String,
}

impl ProjectTopology {
    pub fn to_toml(&self) -> String {
        toml::to_string_pretty(self).unwrap_or_default()
    }

    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }
}
