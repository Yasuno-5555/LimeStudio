//! Pd Compatibility Bridge (Importer/Exporter ONLY)
//!
//! "pd_compat = importer/exporterに限定。runtimeに侵入させない。core型に変換するだけ"
use crate::color::Color;
use crate::ui_ir::{FrameStyle, IndicatorKind, SurfaceId, SurfacePrimitive, TemporalStrategy};

pub struct PdCompat;

impl PdCompat {
    pub fn bang(
        id: SurfaceId,
        pos: [f32; 2],
        size: f32,
        flash_state: f32,
        is_focused: bool,
    ) -> Vec<SurfacePrimitive> {
        let mut primitives = Vec::new();

        if is_focused {
            primitives.push(SurfacePrimitive::FocusRing {
                id,
                rect: [pos[0] - 2.0, pos[1] - 2.0, size + 4.0, size + 4.0],
                color: Color::ACCENT_BLUE.to_array(),
                temporal: TemporalStrategy::Standard,
            });
        }

        primitives.push(SurfacePrimitive::Frame {
            id,
            rect: [pos[0], pos[1], size, size],
            style: FrameStyle::Standard,
            color: Color::BG_PANEL.to_array(),
            temporal: TemporalStrategy::Instant,
        });

        primitives.push(SurfacePrimitive::Indicator {
            id,
            rect: [
                pos[0] + size * 0.15,
                pos[1] + size * 0.15,
                size * 0.7,
                size * 0.7,
            ],
            kind: IndicatorKind::Bang,
            value: flash_state,
            color: Color::ACCENT_LIME.to_array(),
            temporal: TemporalStrategy::Fast,
        });

        primitives
    }

    pub fn toggle(
        id: SurfaceId,
        pos: [f32; 2],
        size: f32,
        is_on: bool,
        is_focused: bool,
    ) -> Vec<SurfacePrimitive> {
        let mut primitives = Vec::new();

        if is_focused {
            primitives.push(SurfacePrimitive::FocusRing {
                id,
                rect: [pos[0] - 2.0, pos[1] - 2.0, size + 4.0, size + 4.0],
                color: Color::ACCENT_BLUE.to_array(),
                temporal: TemporalStrategy::Standard,
            });
        }

        primitives.push(SurfacePrimitive::Frame {
            id,
            rect: [pos[0], pos[1], size, size],
            style: FrameStyle::Standard,
            color: Color::BG_PANEL.to_array(),
            temporal: TemporalStrategy::Instant,
        });

        primitives.push(SurfacePrimitive::Indicator {
            id,
            rect: [
                pos[0] + size * 0.2,
                pos[1] + size * 0.2,
                size * 0.6,
                size * 0.6,
            ],
            kind: IndicatorKind::Toggle,
            value: if is_on { 1.0 } else { 0.0 },
            color: Color::ACCENT_LIME.to_array(),
            temporal: TemporalStrategy::Standard,
        });

        primitives
    }

    pub fn message(id: SurfaceId, pos: [f32; 2], width: f32, height: f32) -> Vec<SurfacePrimitive> {
        vec![SurfacePrimitive::Frame {
            id,
            rect: [pos[0], pos[1], width, height],
            style: FrameStyle::Message,
            color: Color::BG_PANEL.to_array(),
            temporal: TemporalStrategy::Instant,
        }]
    }
}
