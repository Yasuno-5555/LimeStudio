//! Semantic Cable Renderer — Line = Semantics.
//! 
//! Different visual styles for Audio, Control, and Event rates.
//! Highlights feedback dangers and latency paths.

use glam::Vec2;
use crate::color::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticRate {
    Audio,   // Solid heavy line
    Control, // Dashed line
    Event,   // Pulse line
}

pub struct SemanticCable {
    pub start: Vec2,
    pub end: Vec2,
    pub rate: SemanticRate,
    pub is_feedback: bool,
    pub latency_samples: u32,
    pub energy: f32, // For pulse/flow visualization
}

impl SemanticCable {
    pub fn get_style(&self) -> (Color, f32, Vec<f32>) {
        let base_color = match self.rate {
            SemanticRate::Audio => Color::from_hex("#8CB369"),   // Calm Lime
            SemanticRate::Control => Color::from_hex("#8E8E93"), // Secondary Text
            SemanticRate::Event => Color::from_hex("#EBEBF5"),   // Primary Text
        };

        let thickness = match self.rate {
            SemanticRate::Audio => 3.0,
            _ => 1.5,
        };

        let dash_array = match self.rate {
            SemanticRate::Control => vec![10.0, 5.0],
            SemanticRate::Event => vec![2.0, 8.0],
            _ => vec![],
        };

        let final_color = if self.is_feedback {
            Color::from_hex("#D9735A") // Muted Amber (Warning)
        } else {
            base_color
        };

        (final_color, thickness, dash_array)
    }
}

/// GPU data for semantic cables
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SemanticCableInstance {
    pub start: [f32; 2],
    pub end: [f32; 2],
    pub color: [f32; 4],
    pub thickness: f32,
    pub dash_phase: f32,
    pub rate_type: u32,
    pub warning_flags: u32,
}
