use glam::Vec2;
use crate::color::Color;
use crate::ui_ir::{SurfacePrimitive, FrameStyle, TemporalStrategy, GlyphPlacement, SurfaceId};

pub struct NumberBox {
    pub id: SurfaceId,
    pub value: f32,
    pub position: Vec2,
    pub size: Vec2,
    pub is_editing: bool,
    pub is_focused: bool,
    pub colors: NumberBoxColors,
}

pub struct NumberBoxColors {
    pub base: Color,
    pub edit: Color,
    pub text: Color,
}

impl NumberBox {
    pub fn new(id: SurfaceId, value: f32, position: Vec2) -> Self {
        Self {
            id,
            value,
            position,
            size: Vec2::new(64.0, 24.0),
            is_editing: false,
            is_focused: false,
            colors: NumberBoxColors {
                base: Color::BG_PANEL,
                edit: Color::ACCENT_BLUE,
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
                rect: [self.position.x - 2.0, self.position.y - 2.0, self.size.x + 4.0, self.size.y + 4.0],
                color: Color::ACCENT_BLUE.to_array(),
                temporal: TemporalStrategy::Standard,
            });
        }
        
        // 1. Box Frame
        primitives.push(SurfacePrimitive::Frame {
            id: self.id,
            rect: [self.position.x, self.position.y, self.size.x, self.size.y],
            style: if self.is_editing { FrameStyle::Field } else { FrameStyle::Number },
            color: if self.is_editing { self.colors.edit.to_array() } else { self.colors.base.to_array() },
            temporal: TemporalStrategy::Instant,
        });

        // 2. Value Text
        primitives.push(SurfacePrimitive::GlyphRun {
            placements: vec![GlyphPlacement {
                glyph_id: 0,
                pos: [self.position.x + 4.0, self.position.y + 16.0],
                scale: 1.0,
            }],
            color: self.colors.text.to_array(),
        });

        primitives
    }
}
