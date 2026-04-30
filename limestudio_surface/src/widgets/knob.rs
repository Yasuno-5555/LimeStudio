//! ParamKnob — The King of Widgets.
//!
//! Circular/Vertical drag, Precision mode, Modulation ring, Badges, etc.

use crate::color::Color;
use crate::motion::MotionState;
use crate::ui_ir::{
    ArcKind, ContradictionSeverity, IndicatorKind, SurfaceId, SurfacePrimitive, TemporalStrategy,
};
use glam::Vec2;

pub struct KnobConfig {
    pub min: f32,
    pub max: f32,
    pub default: f32,
    pub step: Option<f32>,
    pub vertical_sensitivity: f32,
    pub circular_sensitivity: f32,
}

impl Default for KnobConfig {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 1.0,
            default: 0.5,
            step: None,
            vertical_sensitivity: 0.005,
            circular_sensitivity: 0.01,
        }
    }
}

#[derive(Debug, Clone)]
pub struct KnobState {
    pub value: MotionState,
    pub modulation: MotionState,
    pub is_dragging: bool,
    pub drag_mode: DragMode,
    pub interaction: KnobInteraction,
    pub contradiction: Option<(ContradictionSeverity, String)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragMode {
    Circular,
    Vertical,
}

#[derive(Debug, Clone)]
pub struct KnobInteraction {
    pub drag_origin: Vec2,
    pub value_at_start: f32,
    pub is_precision: bool,
}

pub struct ParamKnob {
    pub id: SurfaceId,
    pub param_id: String,
    pub label: String,
    pub position: Vec2,
    pub radius: f32,
    pub state: KnobState,
    pub config: KnobConfig,
    pub is_focused: bool,
    /// Trust UI: 描画用の色 (Glow は法により禁止)
    pub colors: KnobColors,
}

pub struct KnobColors {
    pub base: Color,
    pub active: Color,
    pub mod_ring: Color,
}

impl ParamKnob {
    pub fn new(id: SurfaceId, param_id: String, label: String, position: Vec2) -> Self {
        Self {
            id,
            param_id,
            label,
            position,
            radius: 24.0, // HIG: 8px grid (24 = 8 * 3)
            state: KnobState {
                value: MotionState::new(0.5),
                modulation: MotionState::new(0.0),
                is_dragging: false,
                drag_mode: DragMode::Vertical,
                interaction: KnobInteraction {
                    drag_origin: Vec2::ZERO,
                    value_at_start: 0.5,
                    is_precision: false,
                },
                contradiction: None,
            },
            config: KnobConfig::default(),
            is_focused: false,
            colors: KnobColors {
                base: Color::BG_PANEL,
                active: Color::ACCENT_LIME,
                mod_ring: Color::MOD_RANGE,
            },
        }
    }

    /// Convert the knob state into low-level surface primitives.
    /// Follows the "Law of Lime" (HIG v3.0) with Sentient Enhancements.
    pub fn build_primitives(&self) -> Vec<SurfacePrimitive> {
        let mut primitives = Vec::new();
        let center = [self.position.x, self.position.y];

        // 0. Sentient Aura (The Life)
        primitives.push(SurfacePrimitive::Organic {
            id: SurfaceId::from_seed(&format!("aura_{}", self.id.0 .0)),
            kind: crate::ui_ir::OrganicKind::Aura {
                center,
                radius: self.radius * 1.5,
                pulsation: self.state.value.value * 0.2, // Pulse based on value
                harmonics: 2,
            },
            brush: crate::ui_ir::BespokeBrush::Solid([0.6, 1.0, 0.4, 0.1]),
            temporal: TemporalStrategy::Slow,
        });

        // 0.1 Focus Ring (The Civilization)
        if self.is_focused {
            primitives.push(SurfacePrimitive::FocusRing {
                id: self.id,
                rect: [
                    self.position.x - self.radius,
                    self.position.y - self.radius,
                    self.radius * 2.0,
                    self.radius * 2.0,
                ],
                color: Color::ACCENT_BLUE.to_array(),
                temporal: TemporalStrategy::Standard,
            });
        }

        // 0.1 Contradiction Marker (The Discord)
        if let Some((severity, desc)) = &self.state.contradiction {
            primitives.push(SurfacePrimitive::ContradictionMarker {
                id: self.id,
                rect: [
                    self.position.x - self.radius - 8.0,
                    self.position.y - self.radius - 8.0,
                    (self.radius + 8.0) * 2.0,
                    (self.radius + 8.0) * 2.0,
                ],
                severity: *severity,
                description: desc.clone(),
            });
        }

        // 1. Value Ring (The Law: 280-degree sweep)
        let base_val = self.state.value.value;
        let start_angle = -140.0;
        let end_angle = -140.0 + (base_val * 280.0);

        primitives.push(SurfacePrimitive::Arc {
            id: self.id,
            center,
            radius: self.radius,
            thickness: 4.0,
            start_angle,
            end_angle,
            kind: ArcKind::Value,
            temporal: TemporalStrategy::Standard, // 60ms
        });

        // 2. Modulation Range (Forensic Trace)
        let mod_val = self.state.modulation.value;
        if (mod_val - base_val).abs() > 0.001 {
            let mod_start = end_angle;
            let mod_end = -140.0 + (mod_val * 280.0);

            primitives.push(SurfacePrimitive::Arc {
                id: self.id,
                center,
                radius: self.radius + 6.0,
                thickness: 2.0,
                start_angle: mod_start,
                end_angle: mod_end,
                kind: ArcKind::Modulation,
                temporal: TemporalStrategy::Fast, // 20ms for modulation
            });

            // Add Persistence Trail for modulation
            primitives.push(SurfacePrimitive::PersistenceTrail {
                id: SurfaceId::from_seed(&format!("mod_trail_{}", self.id.0 .0)),
                source_id: self.id,
                depth: 8,
                decay: 0.8,
            });
        }

        // 3. Knob Cap (Center Indicator)
        primitives.push(SurfacePrimitive::Indicator {
            id: self.id,
            rect: [center[0] - 8.0, center[1] - 8.0, 16.0, 16.0],
            kind: IndicatorKind::Led,
            value: if self.state.is_dragging { 1.0 } else { 0.5 },
            color: if self.state.is_dragging {
                self.colors.active.to_array()
            } else {
                self.colors.base.to_array()
            },
            temporal: TemporalStrategy::Standard,
        });

        primitives
    }
}
