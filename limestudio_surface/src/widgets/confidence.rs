//! Confidence Visualization — "Shape over Color".

use dirtydata_core::types::ConfidenceScore;
use crate::ui_ir::{SurfacePrimitive, TemporalStrategy, SurfaceId};
use glam::Vec2;

pub struct ConfidenceVisualizer {
    pub id: SurfaceId,
    pub score: ConfidenceScore,
    pub position: Vec2,
}

impl ConfidenceVisualizer {
    pub fn build_primitives(&self) -> Vec<SurfacePrimitive> {
        use crate::ui_ir::ProvenanceLevel;
        let level = match self.score {
            ConfidenceScore::Verified => ProvenanceLevel::Verified,
            ConfidenceScore::Inferred => ProvenanceLevel::Inferred,
            ConfidenceScore::Suspicious => ProvenanceLevel::External, // Suspect is external-like
            ConfidenceScore::Unknown => ProvenanceLevel::Stale,
        };

        vec![SurfacePrimitive::ProvenanceBadge {
            id: self.id,
            rect: [self.position.x, self.position.y, 16.0, 16.0],
            level,
            temporal: TemporalStrategy::Standard(0.1),
        }]
    }
}
