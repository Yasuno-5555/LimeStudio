use crate::color::Color;
use crate::ui_ir::{FrameStyle, IndicatorKind, SurfaceId, SurfacePrimitive, TemporalStrategy};
use glam::Vec2;

pub struct SurfaceToggle {
    pub id: SurfaceId,
    pub label: String,
    pub position: Vec2,
    pub size: Vec2,
    pub is_on: bool,
    pub is_focused: bool,
    pub colors: ToggleColors,
}

pub struct ToggleColors {
    pub base: Color,
    pub active: Color,
    pub text: Color,
}

impl SurfaceToggle {
    pub fn new(id: SurfaceId, label: String, position: Vec2) -> Self {
        Self {
            id,
            label,
            position,
            size: Vec2::new(48.0, 24.0), // 8px grid
            is_on: false,
            is_focused: false,
            colors: ToggleColors {
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

        // 1. Outer Frame
        primitives.push(SurfacePrimitive::Frame {
            id: self.id,
            rect: [self.position.x, self.position.y, self.size.x, self.size.y],
            style: FrameStyle::Standard,
            color: self.colors.base.to_array(),
            temporal: TemporalStrategy::Instant,
        });

        // 2. Inner Indicator (Switch)
        primitives.push(SurfacePrimitive::Indicator {
            id: self.id,
            rect: [
                self.position.x + 4.0,
                self.position.y + 4.0,
                self.size.x - 8.0,
                self.size.y - 8.0,
            ],
            kind: IndicatorKind::Toggle,
            value: if self.is_on { 1.0 } else { 0.0 },
            color: self.colors.active.to_array(),
            temporal: TemporalStrategy::Standard,
        });

        primitives
    }
}
