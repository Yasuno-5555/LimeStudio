use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectSpec {
    pub name: String,
    pub graph: dirtydata_core::ir::Graph,
    pub ui: limestudio_graph::LimeGraph,
    pub view: crate::view::ViewCache,
}

impl ProjectSpec {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            graph: dirtydata_core::ir::Graph::new(),
            ui: limestudio_graph::LimeGraph::new(),
            view: crate::view::ViewCache::new(),
        }
    }

    pub fn to_toml(&self) -> String {
        toml::to_string_pretty(self).unwrap_or_default()
    }

    pub fn save_to_file(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let toml = self.to_toml();
        std::fs::write(path, toml)?;
        Ok(())
    }

    pub fn load_from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let spec: Self = toml::from_str(&content)?;
        Ok(spec)
    }
}
