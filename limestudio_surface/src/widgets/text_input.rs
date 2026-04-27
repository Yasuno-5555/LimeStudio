use glam::Vec2;
use crate::color::Color;
use crate::ui_ir::{SurfacePrimitive, FrameStyle, TemporalStrategy, IndicatorKind, GlyphPlacement, SurfaceId};

pub struct SurfaceTextInput {
    pub id: SurfaceId,
    pub label: String,
    pub position: Vec2,
    pub size: Vec2,
    pub text: String,
    pub is_active: bool,
    pub is_focused: bool,
    pub cursor_pos: usize,
    pub colors: TextInputColors,
}

pub struct TextInputColors {
    pub bg: Color,
    pub text: Color,
    pub border: Color,
    pub caret: Color,
}

impl SurfaceTextInput {
    pub fn new(id: SurfaceId, label: String, position: Vec2) -> Self {
        Self {
            id,
            label,
            position,
            size: Vec2::new(160.0, 32.0), // 8px grid
            text: String::new(),
            is_active: false,
            is_focused: false,
            cursor_pos: 0,
            colors: TextInputColors {
                bg: Color::BG_PANEL,
                text: Color::TEXT_PRIMARY,
                border: Color::SYNTAX_COMMENT,
                caret: Color::ACCENT_LIME,
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
                temporal: TemporalStrategy::Standard(0.06),
            });
        }
        
        // 1. Box Frame
        primitives.push(SurfacePrimitive::Frame {
            id: self.id,
            rect: [self.position.x, self.position.y, self.size.x, self.size.y],
            style: if self.is_active { FrameStyle::Field } else { FrameStyle::Standard },
            color: self.colors.bg.to_array(),
            temporal: TemporalStrategy::Instant,
        });

        // 2. Text Content
        primitives.push(SurfacePrimitive::GlyphRun {
            placements: vec![GlyphPlacement {
                glyph_id: 0,
                pos: [self.position.x + 8.0, self.position.y + 20.0],
                scale: 1.0,
            }],
            color: self.colors.text.to_array(),
        });

        // 3. Caret (Indicator)
        if self.is_active {
            primitives.push(SurfacePrimitive::Indicator {
                id: self.id,
                rect: [self.position.x + 8.0 + (self.cursor_pos as f32 * 8.0), self.position.y + 6.0, 2.0, 20.0],
                kind: IndicatorKind::Led,
                value: 1.0,
                color: self.colors.caret.to_array(),
                temporal: TemporalStrategy::Fast(0.06),
            });
        }

        primitives
    }
}
