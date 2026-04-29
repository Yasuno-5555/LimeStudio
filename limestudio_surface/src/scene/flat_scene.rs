//! Lime Surface Flat Scene Graph
//!
//! Optimized for rendering from DirtyData Graph snapshots and ViewCache.

use crate::color::Color;
use dirtydata_core::ir::Graph;
use dirtydata_core::types::StableId;
use glam::Vec2;
use limestudio_core::{UiIndex, ViewCache};

pub struct SurfaceScene {
    pub nodes: Vec<SurfaceNode>,
    pub edges: Vec<SurfaceEdge>,
    pub overlays: Vec<SurfaceOverlay>,
}

pub struct SurfaceNode {
    pub ui_index: UiIndex,
    pub kernel_id: StableId,
    pub position: Vec2,
    pub size: Vec2,
    pub color: Color,
    pub label: String,
}

pub struct SurfaceEdge {
    pub kernel_id: StableId,
    pub from_pos: Vec2,
    pub to_pos: Vec2,
    pub thickness: f32,
    pub color: Color,
}

pub struct SurfaceOverlay {
    pub content: OverlayContent,
    pub position: Vec2,
    pub alpha: f32,
}

pub enum OverlayContent {
    Tooltip(String),
    Provenance(Vec<String>),
    Warning(String),
}

impl Default for SurfaceScene {
    fn default() -> Self {
        Self::new()
    }
}
impl SurfaceScene {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            overlays: Vec::new(),
        }
    }

    /// Rebuild the scene from the reality (Graph) and perception (ViewCache).
    pub fn reconcile(&mut self, graph: &Graph, view_cache: &ViewCache) {
        self.nodes.clear();
        self.edges.clear();

        // 1. Build Nodes from ViewCache positions
        for (&kernel_id, &pos) in &view_cache.node_positions {
            if let Some(ui_index) = view_cache.id_map.get_ui_index(kernel_id) {
                let label = if let Some(node) = graph.node(&kernel_id) {
                    node.config
                        .get("name")
                        .and_then(|v| v.as_string())
                        .cloned()
                        .unwrap_or_else(|| format!("{:?}", node.kind))
                } else {
                    "Optimistic Node".to_string()
                };

                let is_selected = view_cache.selected_nodes.contains(&kernel_id);
                let color = if is_selected {
                    Color::from_rgba(178, 230, 51, 255) // Calm Lime
                } else {
                    Color::from_rgba(51, 51, 56, 255) // Surface
                };

                self.nodes.push(SurfaceNode {
                    ui_index,
                    kernel_id,
                    position: Vec2::new(pos[0], pos[1]),
                    size: Vec2::new(120.0, 60.0),
                    color,
                    label,
                });
            }
        }

        // 2. Build Edges from Graph topology
        for (&edge_id, edge) in &graph.topology.edges {
            let from_pos = view_cache.node_positions.get(&edge.source.node_id);
            let to_pos = view_cache.node_positions.get(&edge.target.node_id);

            if let (Some(fp), Some(tp)) = (from_pos, to_pos) {
                self.edges.push(SurfaceEdge {
                    kernel_id: edge_id,
                    from_pos: Vec2::new(fp[0], fp[1]),
                    to_pos: Vec2::new(tp[0], tp[1]),
                    thickness: 2.0,
                    color: Color::from_rgba(102, 102, 115, 255),
                });
            }
        }
    }

    pub fn hit_test(&self, point: Vec2) -> Option<UiIndex> {
        for node in &self.nodes {
            let half_size = node.size * 0.5;
            let min = node.position - half_size;
            let max = node.position + half_size;
            if point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y {
                return Some(node.ui_index);
            }
        }
        None
    }
}
