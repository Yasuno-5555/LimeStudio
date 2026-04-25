//! Lime Surface Flat Scene Graph
//! 
//! Avoid deep DOM-style trees. Use flat arrays for ECS-like spatial queries.

use glam::Vec2;
use crate::color::Color;

pub struct SurfaceScene {
    pub nodes: Vec<SurfaceNode>,
    pub edges: Vec<SurfaceEdge>,
    pub overlays: Vec<SurfaceOverlay>,
}

pub struct SurfaceNode {
    pub id: u32,
    pub position: Vec2,
    pub size: Vec2,
    pub color: Color,
    pub label: String,
    pub state_flags: u32,
}

pub struct SurfaceEdge {
    pub from_node: u32,
    pub from_port: u32,
    pub to_node: u32,
    pub to_port: u32,
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

impl SurfaceScene {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            overlays: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
        self.overlays.clear();
    }

    /// Spatial query for hit testing
    pub fn hit_test(&self, point: Vec2) -> Option<u32> {
        // Simple linear search for now, could be quadtree/spatial hash later
        for node in &self.nodes {
            let half_size = node.size * 0.5;
            let min = node.position - half_size;
            let max = node.position + half_size;
            if point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y {
                return Some(node.id);
            }
        }
        None
    }
}
