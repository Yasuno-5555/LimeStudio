//! GraphCanvas — The foundation of the Visible Compiler.
//! 
//! Handles pan, zoom, and spatial navigation.
//! Selection is driven by ViewCache (Perception).

use glam::Vec2;
use limestudio_core::{UiIndex};
use crate::color::Color;
use crate::ui_ir::{SurfacePrimitive, FrameStyle, TemporalStrategy, SurfaceId};

pub struct GraphCanvas {
    pub id: SurfaceId,
    pub pan: Vec2,
    pub zoom: f32,
    pub is_panning: bool,
    pub drag_start: Vec2,
    pub is_dirty: bool,
}

impl GraphCanvas {
    pub fn new(id: SurfaceId) -> Self {
        Self {
            id,
            pan: Vec2::ZERO,
            zoom: 1.0,
            is_panning: false,
            drag_start: Vec2::ZERO,
            is_dirty: true,
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
        
        self.zoom = new_zoom;
        let new_focus_screen = self.world_to_screen(focus_world);
        self.pan -= new_focus_screen - focus_screen;
    }

    pub fn handle_pan(&mut self, delta: Vec2) {
        self.pan += delta;
    }

    pub fn build_primitives(&self, screen_size: Vec2) -> Vec<SurfacePrimitive> {
        let mut primitives = Vec::new();

        // 1. Background Grid (Trust UI: 8px grid)
        // Draw major grid lines using Frames
        let grid_size = 64.0 * self.zoom;
        let start_x = (self.pan.x % grid_size) - grid_size;
        let start_y = (self.pan.y % grid_size) - grid_size;

        let mut x = start_x;
        while x < screen_size.x {
            primitives.push(SurfacePrimitive::Frame {
                id: self.id,
                rect: [x, 0.0, 1.0, screen_size.y],
                style: FrameStyle::None,
                color: Color::SYNTAX_COMMENT.to_array(),
                temporal: TemporalStrategy::Instant,
            });
            x += grid_size;
        }

        let mut y = start_y;
        while y < screen_size.y {
            primitives.push(SurfacePrimitive::Frame {
                id: self.id,
                rect: [0.0, y, screen_size.x, 1.0],
                style: FrameStyle::None,
                color: Color::SYNTAX_COMMENT.to_array(),
                temporal: TemporalStrategy::Instant,
            });
            y += grid_size;
        }

        primitives
    }
}

/// Semantic Selection State
pub struct SelectionState {
    pub primary_node: Option<UiIndex>,
    pub secondary_nodes: Vec<UiIndex>,
}
