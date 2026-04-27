use crate::ui_ir::{SurfacePrimitive, ArcKind, TemporalStrategy};
use crate::model::stable_id::SurfaceId;

pub struct CustomWidget;

impl CustomWidget {
    pub fn modulation_arc(id: SurfaceId, center: [f32; 2], radius: f32, value: f32) -> SurfacePrimitive {
        SurfacePrimitive::Arc {
            id,
            center,
            radius,
            thickness: 4.0,
            start_angle: -140.0,
            end_angle: -140.0 + (value * 280.0),
            kind: ArcKind::Modulation,
            temporal: TemporalStrategy::Fast(0.02), // Fast 20ms for modulation
        }
    }
}
