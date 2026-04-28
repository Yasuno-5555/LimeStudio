use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};
use dirtydata_core::types::TypedPort;
use std::path::Path;
use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalUnit {
    Normalized,
    Hertz,
    Seconds,
    Decibels,
    Bipolar,
}

/// §NRT: Node Tiering — The Law of Access
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum NodeTier {
    /// S-Tier: Standard, low CPU, rock-solid stability.
    Core,
    /// A-Tier: Advanced, specialized, higher CPU.
    Advanced,
    /// B-Tier: Experimental, chaotic, potential for instability.
    Experimental,
    /// Forbidden: Destructive, boundary-breaking, unstable.
    Forbidden,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamDefinition {
    pub name: String,
    pub unit: SignalUnit,
    pub default: f32,
    pub range: (f32, f32),
}

/// §NBP: Node Blueprint — The DNA of a VPL Node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeBlueprint {
    pub id_name: String,
    pub display_name: String,
    pub category: String,
    pub tier: NodeTier,
    pub tags: Vec<String>,
    pub ports: Vec<TypedPort>,
    pub params: Vec<ParamDefinition>,
    pub version: String,
}

pub struct NodeRegistry {
    pub blueprints: BTreeMap<String, NodeBlueprint>,
}

impl Default for NodeRegistry {
    fn default() -> Self { Self::new() }
}

impl NodeRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            blueprints: BTreeMap::new(),
        };
        registry.populate_standard_library();
        
        // §NDM: Discovery Mechanism — Automatic expansion
        if let Ok(workspace_root) = std::env::current_dir() {
            let search_path = workspace_root.join("DirtyData/crates");
            if search_path.exists() {
                let _ = registry.discover_nodes(&search_path);
            }
        }
        
        registry
    }

    /// §NDM: Discovery Mechanism — Recursive file system scan
    pub fn discover_nodes(&mut self, root: &Path) -> anyhow::Result<()> {
        if root.is_dir() {
            for entry in fs::read_dir(root)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    let blueprint_path = path.join("blueprint.toml");
                    if blueprint_path.exists() {
                        if let Ok(content) = fs::read_to_string(blueprint_path) {
                            if let Ok(value) = toml::from_str::<toml::Value>(&content) {
                                if let Some(blueprint) = self.parse_blueprint_toml(&value) {
                                    self.register(blueprint);
                                }
                            }
                        }
                    }
                    // Recursive scan for subdirectories (e.g. nested crates)
                    let _ = self.discover_nodes(&path);
                }
            }
        }
        Ok(())
    }

    fn parse_blueprint_toml(&self, value: &toml::Value) -> Option<NodeBlueprint> {
        let b = value.get("blueprint")?;
        
        let id_name = b.get("id_name")?.as_str()?.to_string();
        let display_name = b.get("display_name")?.as_str()?.to_string();
        let category = b.get("category")?.as_str()?.to_string();
        let tier_str = b.get("tier")?.as_str()?;
        let tier = match tier_str {
            "Core" => NodeTier::Core,
            "Advanced" => NodeTier::Advanced,
            "Experimental" => NodeTier::Experimental,
            "Forbidden" => NodeTier::Forbidden,
            _ => NodeTier::Experimental,
        };
        
        let version = b.get("version")?.as_str()?.to_string();
        let tags = b.get("tags")?.as_array()?.iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
            
        let mut params = Vec::new();
        if let Some(p_list) = value.get("params").and_then(|v| v.as_array()) {
            for p in p_list {
                let name = p.get("name")?.as_str()?.to_string();
                let unit_str = p.get("unit")?.as_str()?;
                let unit = match unit_str {
                    "Hertz" => SignalUnit::Hertz,
                    "Seconds" => SignalUnit::Seconds,
                    "Decibels" => SignalUnit::Decibels,
                    "Bipolar" => SignalUnit::Bipolar,
                    _ => SignalUnit::Normalized,
                };
                let default = p.get("default")?.as_float()? as f32;
                let range_arr = p.get("range")?.as_array()?;
                let range = (range_arr[0].as_float()? as f32, range_arr[1].as_float()? as f32);
                
                params.push(ParamDefinition { name, unit, default, range });
            }
        }

        Some(NodeBlueprint {
            id_name,
            display_name,
            category,
            tier,
            tags,
            ports: vec![], // To be expanded with port definitions
            params,
            version,
        })
    }

    fn populate_standard_library(&mut self) {
        // Fallback or internal nodes could be added here
    }

    pub fn register(&mut self, blueprint: NodeBlueprint) {
        self.blueprints.insert(blueprint.id_name.clone(), blueprint);
    }

    pub fn find_by_category(&self, category: &str) -> Vec<&NodeBlueprint> {
        self.blueprints.values()
            .filter(|b| b.category == category)
            .collect()
    }

    pub fn find_by_tier(&self, tier: NodeTier) -> Vec<&NodeBlueprint> {
        self.blueprints.values()
            .filter(|b| b.tier == tier)
            .collect()
    }
}

