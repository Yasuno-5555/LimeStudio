//! GraphCanvas — The foundation of the Visible Compiler.
//! 
//! Handles pan, zoom, selection, and spatial navigation.

use glam::Vec2;
use crate::scene::{SurfaceScene, SurfaceNode, SurfaceEdge};
use crate::color::Color;

pub struct GraphCanvas {
    pub pan: Vec2,
    pub zoom: f32,
    pub selection: Vec<u32>,
    pub is_panning: bool,
    pub drag_start: Vec2,
}

impl GraphCanvas {
    pub fn new() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 1.0,
            selection: Vec::new(),
            is_panning: false,
            drag_start: Vec2::ZERO,
        }
    }

    /// Convert screen coordinates to world coordinates
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        (screen_pos - self.pan) / self.zoom
    }

    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        (world_pos * self.zoom) + self.pan
    }

    pub fn handle_zoom(&mut self, delta: f32, focus_screen: Vec2) {
        let focus_world = self.screen_to_world(focus_screen);
        let new_zoom = (self.zoom * (1.0 + delta)).clamp(0.1, 5.0);
        
        // Adjust pan to zoom towards focus point
        self.zoom = new_zoom;
        let new_focus_screen = self.world_to_screen(focus_world);
        self.pan -= new_focus_screen - focus_screen;
    }

    pub fn handle_pan(&mut self, delta: Vec2) {
        self.pan += delta;
    }

    pub fn update_scene(&self, scene: &mut SurfaceScene) {
        // Apply camera transformation to scene elements? 
        // Or keep scene in world coords and let renderer handle camera.
        // Trust UI: Scene is world, Renderer is camera.
    }
}

/// Semantic Selection State
pub struct SelectionState {
    pub primary_node: Option<u32>,
    pub secondary_nodes: Vec<u32>,
}
