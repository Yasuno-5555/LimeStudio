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
    pub fn get_style(&self, time: f32) -> (Color, f32, f32) {
        let base_color = match self.rate {
            SemanticRate::Audio => Color::ACCENT_LIME,
            SemanticRate::Control => Color::ACCENT_BLUE,
            SemanticRate::Event => Color::ACCENT_BLUE,
        };

        let mut thickness = match self.rate {
            SemanticRate::Audio => 3.0,
            _ => 1.5,
        };

        // 信号のエネルギー（RMS）に応じて太さを微細に変化させる
        thickness += self.energy * 2.0;

        let mut final_color = if self.is_feedback {
            // フィードバック経路は赤く警告（点滅）
            let pulse = (time * 5.0).sin() * 0.5 + 0.5;
            crate::color::mix_oklab(Color::ERROR_RED, Color::BG_PANEL, pulse)
        } else {
            base_color
        };

        // 信号が流れているパルス効果の位相計算
        let pulse_phase = (time * 2.0) % 1.0;

        (final_color, thickness, pulse_phase)
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
