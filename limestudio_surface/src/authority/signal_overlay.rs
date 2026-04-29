use crate::color::Color;
use crate::model::stable_id::SurfaceId;
use glam::Vec2;

#[derive(Debug, Clone)]
pub enum SignalStatus {
    Normal,
    Warning(String),  // Amber
    Critical(String), // Red
}

pub struct SignalBadge {
    pub node_id: SurfaceId,
    pub status: SignalStatus,
    pub latency_samples: u32,
    pub cpu_us: f32,
}

impl SignalBadge {
    pub fn new(node_id: SurfaceId) -> Self {
        Self {
            node_id,
            status: SignalStatus::Normal,
            latency_samples: 0,
            cpu_us: 0.0,
        }
    }

    /// Convert badges to SurfacePrimitives for rendering.
    pub fn to_primitives(&self, screen_pos: Vec2) -> Vec<crate::ui_ir::SurfacePrimitive> {
        let mut primitives = Vec::new();
        let mut offset_y = 0.0;

        // 1. Latency Badge
        if self.latency_samples > 0 {
            let text = format!("+{} smp", self.latency_samples);
            primitives.extend(self.create_mini_badge(
                screen_pos + Vec2::new(0.0, offset_y),
                &text,
                Color::TEXT_SECONDARY,
            ));
            offset_y += 12.0;
        }

        // 2. NaN / Error Badge
        match &self.status {
            SignalStatus::Warning(msg) => {
                primitives.extend(self.create_mini_badge(
                    screen_pos + Vec2::new(0.0, offset_y),
                    msg,
                    Color::AMBER,
                ));
                offset_y += 12.0;
            }
            SignalStatus::Critical(msg) => {
                primitives.extend(self.create_mini_badge(
                    screen_pos + Vec2::new(0.0, offset_y),
                    msg,
                    Color::ERROR,
                ));
                offset_y += 12.0;
            }
            _ => {}
        }

        // 3. CPU Badge
        if self.cpu_us > 0.0 {
            let text = format!("{:.1}μs", self.cpu_us);
            primitives.extend(self.create_mini_badge(
                screen_pos + Vec2::new(0.0, offset_y),
                &text,
                Color::TEXT_SECONDARY,
            ));
        }

        primitives
    }

    fn create_mini_badge(
        &self,
        pos: Vec2,
        _text: &str,
        color: Color,
    ) -> Vec<crate::ui_ir::SurfacePrimitive> {
        let mut prims = Vec::new();
        let rect = [pos.x, pos.y, 40.0, 10.0];

        prims.push(crate::ui_ir::SurfacePrimitive::Frame {
            id: self.node_id,
            rect,
            style: crate::ui_ir::FrameStyle::Standard,
            color: [0.08, 0.08, 0.08, 0.8],
            temporal: crate::ui_ir::TemporalStrategy::Instant,
        });

        // NOTE: Glyph rendering is handled via specialized GlyphRun primitives in the renderer
        // For now we just use a colored indicator as a placeholder for the text badge
        prims.push(crate::ui_ir::SurfacePrimitive::Indicator {
            id: self.node_id,
            rect: [pos.x + 2.0, pos.y + 2.0, 6.0, 6.0],
            kind: crate::ui_ir::IndicatorKind::Led,
            value: 1.0,
            color: color.to_array(),
            temporal: crate::ui_ir::TemporalStrategy::Instant,
        });

        prims
    }
}
