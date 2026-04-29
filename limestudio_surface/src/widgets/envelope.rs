use glam::Vec2;
use crate::color::Color;
use crate::ui_ir::{SurfacePrimitive, CurveKind, TemporalStrategy, IndicatorKind, SurfaceId};

pub struct EnvelopePoint {
    pub position: Vec2, // 0.0 to 1.0
    pub tension: f32,   // Curvature
}

pub struct InteractiveEnvelope {
    pub id: SurfaceId,
    pub position: Vec2,
    pub size: Vec2,
    pub points: Vec<EnvelopePoint>,
    pub selected_point: Option<usize>,
    pub colors: EnvelopeColors,
}

pub struct EnvelopeColors {
    pub bg: Color,
    pub line: Color,
    pub point: Color,
    pub selection: Color,
}

impl InteractiveEnvelope {
    pub fn new(id: SurfaceId, position: Vec2, size: Vec2) -> Self {
        Self {
            id,
            position,
            size,
            points: vec![
                EnvelopePoint { position: Vec2::new(0.0, 0.0), tension: 0.0 },
                EnvelopePoint { position: Vec2::new(0.2, 1.0), tension: 0.0 },
                EnvelopePoint { position: Vec2::new(0.5, 0.5), tension: 0.0 },
                EnvelopePoint { position: Vec2::new(1.0, 0.0), tension: 0.0 },
            ],
            selected_point: None,
            colors: EnvelopeColors {
                bg: Color::BG_DEEP,
                line: Color::ACCENT_LIME,
                point: Color::TEXT_PRIMARY,
                selection: Color::MOD_RANGE,
            },
        }
    }

    pub fn build_primitives(&self) -> Vec<SurfacePrimitive> {
        let mut primitives = Vec::new();

        // 1. Envelope Curve
        let curve_points: Vec<[f32; 2]> = self.points.iter().map(|p| {
            [
                self.position.x + p.position.x * self.size.x,
                self.position.y + (1.0 - p.position.y) * self.size.y,
            ]
        }).collect();

        primitives.push(SurfacePrimitive::Curve {
            id: self.id,
            control_points: curve_points.clone(),
            kind: CurveKind::Envelope,
            thickness: 2.0,
            color: self.colors.line.to_array(),
            temporal: TemporalStrategy::Standard,
        });

        // 2. Control Points
        for (i, p) in curve_points.iter().enumerate() {
            let is_selected = self.selected_point == Some(i);
            primitives.push(SurfacePrimitive::Indicator {
                id: self.id, // In practice, points might need sub-IDs
                rect: [p[0] - 4.0, p[1] - 4.0, 8.0, 8.0],
                kind: IndicatorKind::Led,
                value: if is_selected { 1.0 } else { 0.5 },
                color: if is_selected { self.colors.selection.to_array() } else { self.colors.point.to_array() },
                temporal: TemporalStrategy::Fast,
            });
        }

        primitives
    }
}
