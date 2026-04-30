//! Semantic Cable Renderer — Line = Semantics.
//!
//! Different visual styles for Audio, Control, and Event rates.
//! Highlights feedback dangers and latency paths.

use crate::color::Color;
use crate::ui_ir::{CurveKind, SurfaceId, SurfacePrimitive, TemporalStrategy};
use glam::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticRate {
    Audio,   // Solid heavy line
    Control, // Dashed line
    Event,   // Pulse line
}

pub struct SemanticCable {
    pub id: SurfaceId,
    pub start: Vec2,
    pub end: Vec2,
    pub rate: SemanticRate,
    pub is_feedback: bool,
    pub latency_samples: u32,
    pub energy: f32, // For pulse/flow visualization
    pub flow_phase: f32,
}

impl SemanticCable {
    pub fn build_primitives(&self) -> Vec<SurfacePrimitive> {
        let color = match self.rate {
            SemanticRate::Audio => Color::ACCENT_LIME,
            SemanticRate::Control => Color::ACCENT_BLUE,
            SemanticRate::Event => Color::ACCENT_BLUE,
        };

        let color = if self.is_feedback {
            Color::ERROR_RED
        } else {
            color
        };

        let thickness = match self.rate {
            SemanticRate::Audio => 3.0,
            _ => 1.5,
        } + self.energy * 2.0;

        let kind = if self.energy > 0.01 {
            CurveKind::Flow {
                direction: 1.0,
                phase: self.flow_phase,
                density: (self.energy * 5.0).clamp(1.0, 10.0),
            }
        } else {
            CurveKind::Cable
        };

        let mut primitives = vec![SurfacePrimitive::Curve {
            id: self.id,
            control_points: vec![[self.start.x, self.start.y], [self.end.x, self.end.y]],
            kind,
            thickness,
            color: color.to_array(),
            temporal: TemporalStrategy::Fast,
        }];

        // Add Persistence Trail for signal flow
        if self.energy > 0.1 {
            primitives.push(SurfacePrimitive::PersistenceTrail {
                id: SurfaceId::from_seed(&format!("cable_trail_{}", self.id.0 .0)),
                source_id: self.id,
                depth: 10,
                decay: 0.6,
            });
        }

        primitives
    }
}
