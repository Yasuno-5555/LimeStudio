use crate::model::stable_id::SurfaceId;
use glam::Vec2;
use std::collections::HashMap;

pub struct CodeFragment {
    pub source: String,
    pub language: String,
}

/// §SSS: Visible Compiler Registry — The Authority Layer.
/// "ViewCache は受動的すぎる。Authority Layer は能動的に真実を管理する必要がある。"
pub struct VisibleCompilerRegistry {
    /// The source of truth for all node code.
    pub provenance: HashMap<SurfaceId, CodeFragment>,
    /// Cached layout metadata for the UI.
    pub ui_cache: HashMap<SurfaceId, SnippetMetadata>,
}

pub struct SnippetMetadata {
    pub offset: Vec2,
    pub is_dirty: bool,
    /// The hash of the last rendered content to prevent redundant shaping.
    pub last_hash: [u8; 32],
}

impl Default for VisibleCompilerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl VisibleCompilerRegistry {
    pub fn new() -> Self {
        Self {
            provenance: HashMap::new(),
            ui_cache: HashMap::new(),
        }
    }

    /// Fully update the authority with new provenance data.
    pub fn update_authority(&mut self, new_provenance: HashMap<SurfaceId, CodeFragment>) {
        self.provenance = new_provenance;
        // Mark all as dirty to force UI refresh if content changed
        for meta in self.ui_cache.values_mut() {
            meta.is_dirty = true;
        }
    }

    /// Get the fragment for a specific node.
    pub fn get_fragment(&self, id: SurfaceId) -> Option<&CodeFragment> {
        self.provenance.get(&id)
    }

    /// Mark a snippet as cleaned after processing by the text renderer.
    pub fn mark_clean(&mut self, id: SurfaceId) {
        if let Some(meta) = self.ui_cache.get_mut(&id) {
            meta.is_dirty = false;
        }
    }
}
