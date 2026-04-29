use crate::color::Color;
use crate::ui_ir::{GlyphPlacement, SurfaceId, SurfacePrimitive};
use glam::Vec2;

pub struct SurfaceLabel {
    pub id: SurfaceId,
    pub text: String,
    pub position: Vec2,
    pub is_secondary: bool,
    pub colors: LabelColors,
}

pub struct LabelColors {
    pub primary: Color,
    pub secondary: Color,
}

impl SurfaceLabel {
    pub fn new(id: SurfaceId, text: String, position: Vec2) -> Self {
        Self {
            id,
            text,
            position,
            is_secondary: false,
            colors: LabelColors {
                primary: Color::TEXT_PRIMARY,
                secondary: Color::TEXT_SECONDARY,
            },
        }
    }

    pub fn build_primitives(&self) -> Vec<SurfacePrimitive> {
        let color = if self.is_secondary {
            self.colors.secondary
        } else {
            self.colors.primary
        };

        // In a real implementation, we would shape the text here.
        // For now, we emit a GlyphRun placeholder.
        vec![SurfacePrimitive::GlyphRun {
            placements: vec![GlyphPlacement {
                glyph_id: 0, // Root glyph placeholder
                pos: [self.position.x, self.position.y],
                scale: 1.0,
            }],
            color: color.to_array(),
        }]
    }
}
