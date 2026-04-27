use glam::Vec2;
use crate::color::Color;
use crate::ui_ir::{SurfacePrimitive, FrameStyle, TemporalStrategy, SurfaceId};
use dirtydata_core::provenance::CodeFragment;

/// §SSS: Authority Drawer — The Accountable Truth.
/// "Persistent panel = accountable truth. 逃がすな。"
pub struct AuthorityDrawer {
    pub id: SurfaceId,
    pub position: Vec2,
    pub size: Vec2,
    pub is_open: bool,
    pub scroll_y: f32,
    /// Currently selected line index (1-based).
    pub selected_line: Option<usize>,
}

impl AuthorityDrawer {
    pub fn new(id: SurfaceId, screen_size: Vec2) -> Self {
        let width = 400.0;
        Self {
            id,
            position: Vec2::new(screen_size.x - width, 0.0),
            size: Vec2::new(width, screen_size.y),
            is_open: true,
            scroll_y: 0.0,
            selected_line: None,
        }
    }

    pub fn resize(&mut self, screen_size: Vec2) {
        self.position = Vec2::new(screen_size.x - self.size.x, 0.0);
        self.size.y = screen_size.y;
    }

    pub fn build_primitives(&self, fragment: Option<&CodeFragment>) -> Vec<SurfacePrimitive> {
        let mut primitives = Vec::new();
        
        if !self.is_open {
            return primitives;
        }

        // 1. Background (Solid Matte)
        primitives.push(SurfacePrimitive::Frame {
            id: self.id,
            rect: [self.position.x, self.position.y, self.size.x, self.size.y],
            style: FrameStyle::Standard,
            color: Color::AUTHORITY_BG.to_array(),
            temporal: TemporalStrategy::Instant,
        });

        // 2. Border Line (Left Edge)
        primitives.push(SurfacePrimitive::Frame {
            id: self.id,
            rect: [self.position.x, 0.0, 1.0, self.size.y],
            style: FrameStyle::None,
            color: Color::ACCENT_LIME.to_array(),
            temporal: TemporalStrategy::Instant,
        });

        if let Some(_frag) = fragment {
            // Header BG
            primitives.push(SurfacePrimitive::Frame {
                id: self.id,
                rect: [self.position.x + 16.0, 16.0, self.size.x - 32.0, 40.0],
                style: FrameStyle::Standard,
                color: Color::BG_PANEL.to_array(),
                temporal: TemporalStrategy::Instant,
            });

            // Highlight selected line
            if let Some(line) = self.selected_line {
                let line_y = 100.0 + (line as f32 - 1.0) * 24.0;
                primitives.push(SurfacePrimitive::Frame {
                    id: self.id,
                    rect: [self.position.x, line_y, self.size.x, 24.0],
                    style: FrameStyle::None,
                    color: [Color::ACCENT_LIME.0.x, Color::ACCENT_LIME.0.y, Color::ACCENT_LIME.0.z, 0.15],
                    temporal: TemporalStrategy::Fast(0.06),
                });
            }
        }

        primitives
    }
}
