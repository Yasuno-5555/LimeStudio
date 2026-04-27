use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};
use dirtydata_core::types::{TypedPort, SignalUnit};

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
        registry
    }

    /// §NDM: Discovery Mechanism
    /// TODO: Scan manifest.toml files in DirtyData crates.
    fn populate_standard_library(&mut self) {
        // --- 1. Chaos Module (Experimental) ---
        self.register(NodeBlueprint {
            id_name: "chaos.chua".into(),
            display_name: "Chua's Circuit".into(),
            category: "Chaos".into(),
            tier: NodeTier::Experimental,
            tags: vec!["Physical".into(), "Non-linear".into(), "Noisy".into()],
            ports: vec![], // Defined by process
            params: vec![
                ParamDefinition { name: "Alpha".into(), unit: SignalUnit::Normalized, default: 15.6, range: (0.0, 30.0) },
                ParamDefinition { name: "Beta".into(), unit: SignalUnit::Normalized, default: 28.0, range: (0.0, 50.0) },
                ParamDefinition { name: "Rate".into(), unit: SignalUnit::Hertz, default: 1.0, range: (0.01, 100.0) },
            ],
            version: "0.1.0".into(),
        });

        // --- 2. Spectral Module (Advanced) ---
        self.register(NodeBlueprint {
            id_name: "spectral.blur".into(),
            display_name: "Spectral Blur".into(),
            category: "Spectral".into(),
            tier: NodeTier::Advanced,
            tags: vec!["FFT".into(), "Ambient".into()],
            ports: vec![],
            params: vec![
                ParamDefinition { name: "Size".into(), unit: SignalUnit::Seconds, default: 0.5, range: (0.0, 10.0) },
                ParamDefinition { name: "Feedback".into(), unit: SignalUnit::Normalized, default: 0.8, range: (0.0, 0.99) },
            ],
            version: "0.2.1".into(),
        });

        // --- 3. Filter Module (Core) ---
        self.register(NodeBlueprint {
            id_name: "filter.svf".into(),
            display_name: "SVF Filter".into(),
            category: "Filter".into(),
            tier: NodeTier::Core,
            tags: vec!["Standard".into(), "ZDF".into()],
            ports: vec![],
            params: vec![
                ParamDefinition { name: "Cutoff".into(), unit: SignalUnit::Hertz, default: 1000.0, range: (20.0, 20000.0) },
                ParamDefinition { name: "Resonance".into(), unit: SignalUnit::Normalized, default: 0.5, range: (0.0, 1.0) },
            ],
            version: "1.0.0".into(),
        });
    }

    pub fn register(&mut self, blueprint: NodeBlueprint) {
        self.blueprints.insert(blueprint.id_name.clone(), blueprint);
    }

    /// §NCF: Context Filtering
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
