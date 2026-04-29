use crate::color::Color;
use crate::ui_ir::{FrameStyle, GlyphPlacement, SurfaceId, SurfacePrimitive, TemporalStrategy};
use glam::Vec2;

pub struct SurfaceButton {
    pub id: SurfaceId,
    pub label: String,
    pub position: Vec2,
    pub size: Vec2,
    pub is_pressed: bool,
    pub is_focused: bool,
    pub colors: ButtonColors,
}

pub struct ButtonColors {
    pub base: Color,
    pub active: Color,
    pub text: Color,
}

impl SurfaceButton {
    pub fn new(id: SurfaceId, label: String, position: Vec2) -> Self {
        Self {
            id,
            label,
            position,
            size: Vec2::new(80.0, 32.0), // 8px grid
            is_pressed: false,
            is_focused: false,
            colors: ButtonColors {
                base: Color::BG_PANEL,
                active: Color::ACCENT_LIME,
                text: Color::TEXT_PRIMARY,
            },
        }
    }

    pub fn build_primitives(&self) -> Vec<SurfacePrimitive> {
        let mut primitives = Vec::new();

        // 0. Focus Ring (The Civilization)
        if self.is_focused {
            primitives.push(SurfacePrimitive::FocusRing {
                id: self.id,
                rect: [
                    self.position.x - 2.0,
                    self.position.y - 2.0,
                    self.size.x + 4.0,
                    self.size.y + 4.0,
                ],
                color: Color::ACCENT_BLUE.to_array(),
                temporal: TemporalStrategy::Standard,
            });
        }

        // 1. Background Frame
        primitives.push(SurfacePrimitive::Frame {
            id: self.id,
            rect: [self.position.x, self.position.y, self.size.x, self.size.y],
            style: FrameStyle::Standard,
            color: if self.is_pressed {
                self.colors.active.to_array()
            } else {
                self.colors.base.to_array()
            },
            temporal: TemporalStrategy::Standard,
        });

        // 2. Label
        primitives.push(SurfacePrimitive::GlyphRun {
            placements: vec![GlyphPlacement {
                glyph_id: 0,
                pos: [self.position.x + 8.0, self.position.y + 20.0],
                scale: 1.0,
            }],
            color: self.colors.text.to_array(),
        });

        primitives
    }
}
