use std::collections::HashMap;
use crate::model::stable_id::SurfaceId;
use dirtydata_core::provenance::{ProvenanceMap, CodeFragment};
use glam::Vec2;

/// §SSS: Visible Compiler Registry — The Authority Layer.
/// "ViewCache は受動的すぎる。Authority Layer は能動的に真実を管理する必要がある。"
pub struct VisibleCompilerRegistry {
    /// The source of truth for all node code.
    pub provenance: ProvenanceMap,
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
    fn default() -> Self { Self::new() }
}

impl VisibleCompilerRegistry {
    pub fn new() -> Self {
        Self {
            provenance: ProvenanceMap::new(),
            ui_cache: HashMap::new(),
        }
    }

    /// Fully update the authority with new provenance data.
    pub fn update_authority(&mut self, new_provenance: ProvenanceMap) {
        for (node_id, fragment) in new_provenance.node_to_code {
            let surface_id = SurfaceId(node_id);
            
            // Check if the content has changed using hashes
            if let Some(meta) = self.ui_cache.get_mut(&surface_id) {
                if meta.last_hash != fragment.source_hash {
                    meta.last_hash = fragment.source_hash;
                    meta.is_dirty = true;
                }
            } else {
                self.ui_cache.insert(surface_id, SnippetMetadata {
                    offset: Vec2::new(140.0, 0.0),
                    is_dirty: true,
                    last_hash: fragment.source_hash,
                });
            }
            
            self.provenance.insert(node_id, fragment);
        }
    }

    /// Get the fragment for a specific node.
    pub fn get_fragment(&self, id: SurfaceId) -> Option<&CodeFragment> {
        self.provenance.node_to_code.get(&id.0)
    }

    /// Mark a snippet as cleaned after processing by the text renderer.
    pub fn mark_clean(&mut self, id: SurfaceId) {
        if let Some(meta) = self.ui_cache.get_mut(&id) {
            meta.is_dirty = false;
        }
    }
}
