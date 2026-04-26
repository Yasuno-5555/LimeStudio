//! Confidence Badge — The Proof of Quality.
//! 
//! "Shape over Color. Color is auxiliary."

use glam::Vec2;
use crate::color::Color;
pub use limestudio_core::confidence::{ConfidenceInfo, ConfidenceState};

pub struct ConfidenceBadge {
    pub position: Vec2,
    pub size: Vec2,
    pub info: ConfidenceInfo,
    pub alpha: f32,
}

impl ConfidenceBadge {
    pub fn new(position: Vec2, info: ConfidenceInfo) -> Self {
        Self {
            position,
            size: Vec2::new(12.0, 12.0),
            info,
            alpha: 1.0,
        }
    }

    pub fn update(&mut self, _dt: f32) {
        // Subtle animation if needed
    }

    /// Helper to get color for Surface runtime
    pub fn get_color(&self) -> Color {
        match self.info.state {
            ConfidenceState::Safe => Color::from_hex("#8CB369"),    // CALM_LIME
            ConfidenceState::Warning => Color::from_hex("#D9735A"), // MUTED_AMBER
            ConfidenceState::Dangerous => Color::from_rgb(200, 50, 50),
            ConfidenceState::Unknown => Color::from_hex("#8E8E93"), // TEXT_SECONDARY
        }
    }
}
