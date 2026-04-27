use glam::Vec2;
use crate::color::Color;
use crate::ui_ir::{SurfacePrimitive, FrameStyle, TemporalStrategy, IndicatorKind, GlyphPlacement, SurfaceId};

pub struct SurfaceDropDown {
    pub id: SurfaceId,
    pub param_id: String,
    pub label: String,
    pub position: Vec2,
    pub size: Vec2,
    pub options: Vec<String>,
    pub selected_index: usize,
    pub is_open: bool,
    pub is_focused: bool,
    pub colors: DropDownColors,
}

pub struct DropDownColors {
    pub base: Color,
    pub text: Color,
    pub highlight: Color,
}

impl SurfaceDropDown {
    pub fn new(id: SurfaceId, param_id: String, label: String, position: Vec2, options: Vec<String>) -> Self {
        Self {
            id,
            param_id,
            label,
            position,
            size: Vec2::new(128.0, 32.0),
            options,
            selected_index: 0,
            is_open: false,
            is_focused: false,
            colors: DropDownColors {
                base: Color::BG_PANEL,
                text: Color::TEXT_PRIMARY,
                highlight: Color::ACCENT_LIME,
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
        
        // 1. Base Frame
        primitives.push(SurfacePrimitive::Frame {
            id: self.id,
            rect: [self.position.x, self.position.y, self.size.x, self.size.y],
            style: FrameStyle::Standard,
            color: self.colors.base.to_array(),
            temporal: TemporalStrategy::Instant,
        });

        // 2. Selected Text
        let _selected_text = self.options.get(self.selected_index).cloned().unwrap_or_default();
        primitives.push(SurfacePrimitive::GlyphRun {
            placements: vec![GlyphPlacement {
                glyph_id: 0,
                pos: [self.position.x + 8.0, self.position.y + 20.0],
                scale: 1.0,
            }],
            color: self.colors.text.to_array(),
        });

        // 3. Arrow (Indicator)
        primitives.push(SurfacePrimitive::Indicator {
            id: self.id,
            rect: [self.position.x + self.size.x - 24.0, self.position.y + 8.0, 16.0, 16.0],
            kind: IndicatorKind::Radio, // Using radio as a circle placeholder for arrow
            value: if self.is_open { 1.0 } else { 0.0 },
            color: self.colors.highlight.to_array(),
            temporal: TemporalStrategy::Standard(0.06),
        });

        // 4. Open List (Overlay)
        if self.is_open {
            let list_height = (self.options.len() as f32) * self.size.y;
            primitives.push(SurfacePrimitive::Frame {
                id: self.id,
                rect: [self.position.x, self.position.y + self.size.y, self.size.x, list_height],
                style: FrameStyle::Standard,
                color: Color::BG_DEEP.to_array(),
                temporal: TemporalStrategy::Standard(0.1),
            });
        }

        primitives
    }
}
