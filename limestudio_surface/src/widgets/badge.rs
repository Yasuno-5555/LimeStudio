use crate::ui_ir::{ProvenanceLevel, SurfaceId, SurfacePrimitive, TemporalStrategy};
use glam::Vec2;

pub struct SurfaceBadge {
    pub id: SurfaceId,
    pub text: String,
    pub position: Vec2,
    pub size: Vec2,
    pub level: ProvenanceLevel,
}

impl SurfaceBadge {
    pub fn new(id: SurfaceId, text: String, position: Vec2, level: ProvenanceLevel) -> Self {
        Self {
            id,
            text,
            position,
            size: Vec2::new(48.0, 16.0),
            level,
        }
    }

    pub fn build_primitives(&self) -> Vec<SurfacePrimitive> {
        vec![SurfacePrimitive::ProvenanceBadge {
            id: self.id,
            rect: [self.position.x, self.position.y, self.size.x, self.size.y],
            level: self.level,
            temporal: TemporalStrategy::Standard,
        }]
    }
}
