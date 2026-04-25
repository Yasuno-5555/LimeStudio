//! ParamKnob — The King of Widgets.
//! 
//! Circular/Vertical drag, Precision mode, Modulation ring, Badges, etc.

use glam::Vec2;
use crate::motion::MotionState;
use crate::color::Color;

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
}

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
        }
    }

    pub fn handle_drag(&mut self, current_pos: Vec2, is_precision: bool) {
        if !self.state.is_dragging { return; }

        let delta = current_pos - self.state.interaction.drag_origin;
        let sensitivity = if is_precision { 0.1 } else { 1.0 };

        match self.state.drag_mode {
            DragMode::Vertical => {
                let change = -delta.y * self.config.vertical_sensitivity * sensitivity;
                let new_val = (self.state.interaction.value_at_start + change).clamp(0.0, 1.0);
                self.state.value.value = new_val; // Direct set during drag, or let spring catch up? 
                                                // Trust UI: Drag is direct, release is spring.
            }
            DragMode::Circular => {
                // Calculate angle change relative to knob center
                // This would be more complex math
            }
        }
    }

    pub fn reset_to_default(&mut self) {
        self.state.value.value = self.config.default;
        self.state.value.velocity = 0.0;
    }
}

/// Badge definitions for Trust UI
pub struct KnobBadges {
    pub has_automation: bool,
    pub has_midi_learn: bool,
    pub provenance_source: Option<String>,
}
