use glam::Vec2;
use crate::motion::MotionState;
use crate::color::Color;
use crate::ui_ir::{SurfacePrimitive, TemporalStrategy, FrameStyle, IndicatorKind, SurfaceId};

pub struct ParamSlider {
    pub id: SurfaceId,
    pub param_id: String,
    pub label: String,
    pub position: Vec2,
    pub size: Vec2,
    pub value: MotionState,
    pub min: f32,
    pub max: f32,
    pub is_vertical: bool,
    pub is_focused: bool,
    pub colors: SliderColors,
}

pub struct SliderColors {
    pub track: Color,
    pub handle: Color,
    pub accent: Color,
}

impl ParamSlider {
    pub fn new(id: SurfaceId, param_id: String, label: String, position: Vec2, is_vertical: bool) -> Self {
        Self {
            id,
            param_id,
            label,
            position,
            size: if is_vertical { Vec2::new(24.0, 128.0) } else { Vec2::new(128.0, 24.0) }, // 8px grid
            value: MotionState::new(0.5),
            min: 0.0,
            max: 1.0,
            is_vertical,
            is_focused: false,
            colors: SliderColors {
                track: Color::BG_PANEL,
                handle: Color::SYNTAX_VAR,
                accent: Color::ACCENT_LIME,
            },
        }
    }

    pub fn build_primitives(&self) -> Vec<SurfacePrimitive> {
        let mut primitives = Vec::new();

        // 0. Focus Ring (The Civilization)
        if self.is_focused {
            primitives.push(SurfacePrimitive::FocusRing {
                id: self.id,
                rect: [self.position.x - 4.0, self.position.y - 4.0, self.size.x + 8.0, self.size.y + 8.0],
                color: Color::ACCENT_BLUE.to_array(),
                temporal: TemporalStrategy::Standard(0.06),
            });
        }
        
        // 1. Track (Frame)
        primitives.push(SurfacePrimitive::Frame {
            id: self.id,
            rect: [self.position.x, self.position.y, self.size.x, self.size.y],
            style: FrameStyle::Standard,
            color: self.colors.track.to_array(),
            temporal: TemporalStrategy::Instant,
        });

        // 2. Active Range (Indicator or another Frame)
        let val = self.value.value;
        let active_rect = if self.is_vertical {
            [self.position.x, self.position.y + self.size.y * (1.0 - val), self.size.x, self.size.y * val]
        } else {
            [self.position.x, self.position.y, self.size.x * val, self.size.y]
        };

        primitives.push(SurfacePrimitive::Indicator {
            id: self.id, // Reuse ID for transitions
            rect: active_rect,
            kind: IndicatorKind::Led, // Using Led kind as a solid block here
            value: val,
            color: self.colors.accent.to_array(),
            temporal: TemporalStrategy::Standard(0.06),
        });

        // 3. Handle (Indicator - Radio style)
        let handle_pos = if self.is_vertical {
            [self.position.x, self.position.y + self.size.y * (1.0 - val) - 4.0, self.size.x, 8.0]
        } else {
            [self.position.x + self.size.x * val - 4.0, self.position.y, 8.0, self.size.y]
        };

        primitives.push(SurfacePrimitive::Indicator {
            id: self.id,
            rect: handle_pos,
            kind: IndicatorKind::Radio,
            value: 1.0,
            color: self.colors.handle.to_array(),
            temporal: TemporalStrategy::Standard(0.06),
        });

        primitives
    }
}
