//! ParamKnob — The King of Widgets.
//! 
//! Circular/Vertical drag, Precision mode, Modulation ring, Badges, etc.

use glam::Vec2;
use crate::motion::MotionState;
use crate::color::Color;

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
    pub id: u32,
    pub label: String,
    pub position: Vec2,
    pub radius: f32,
    pub state: KnobState,
    pub config: KnobConfig,
    /// Trust UI: 描画用の色とグロー設定
    pub colors: KnobColors,
}

pub struct KnobColors {
    pub base: Color,
    pub active: Color,
    pub mod_ring: Color,
    pub glow_intensity: f32,
}

impl ParamKnob {
    pub fn new(id: u32, label: String, position: Vec2) -> Self {
        Self {
            id,
            label,
            position,
            radius: 20.0,
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
            },
            config: KnobConfig::default(),
            colors: KnobColors {
                base: Color::BG_PANEL,
                active: Color::ACCENT_LIME,
                mod_ring: Color::MOD_RANGE,
                glow_intensity: 0.0,
            },
        }
    }

    /// モジュレーションリングの描画パラメータを計算
    /// (start_angle, end_angle, is_modulating)
    pub fn get_ring_params(&self) -> (f32, f32, bool) {
        let base = self.state.value.value;
        let mod_val = self.state.modulation.value;
        let is_mod = (mod_val - base).abs() > 0.001;
        
        // 0.0 .. 1.0 を -140度 .. 140度に変換
        let start = -140.0;
        let end = -140.0 + (base * 280.0);
        
        (start, end, is_mod)
    }
}

/// Badge definitions for Trust UI
pub struct KnobBadges {
    pub has_automation: bool,
    pub has_midi_learn: bool,
    pub provenance_source: Option<String>,
}
