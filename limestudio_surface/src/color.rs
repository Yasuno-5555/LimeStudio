//! Lime Surface Color System
//! 
//! Based on Oklab color space for perceptual uniformity.
//! "Mandatory Oklab: All color interpolation occurs in Oklab space."

use glam::Vec4;

#[derive(Clone, Copy, Debug)]
pub struct Color(pub Vec4);

impl Color {
    pub const TRANSPARENT: Color = Color(Vec4::new(0.0, 0.0, 0.0, 0.0));
    
    // Premium Dark Palette
    pub const BG_DEEP: Color = Color(Vec4::new(0.02, 0.02, 0.03, 1.0));
    pub const BG_PANEL: Color = Color(Vec4::new(0.05, 0.05, 0.07, 1.0));
    pub const ACCENT_LIME: Color = Color(Vec4::new(0.7, 1.0, 0.0, 1.0));
    pub const ACCENT_BLUE: Color = Color(Vec4::new(0.0, 0.6, 1.0, 1.0));
    pub const ERROR_RED: Color = Color(Vec4::new(1.0, 0.2, 0.2, 1.0));
    
    pub const MOD_ACTIVE: Color = Color(Vec4::new(0.0, 1.0, 0.8, 1.0));
    pub const MOD_RANGE: Color = Color(Vec4::new(0.0, 1.0, 0.8, 0.2));

    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self(Vec4::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0))
    }

    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(Vec4::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0))
    }

    pub fn from_hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            Self::from_rgb(r, g, b)
        } else {
            Self::BG_DEEP
        }
    }

    pub fn to_array(&self) -> [f32; 4] {
        [self.0.x, self.0.y, self.0.z, self.0.w]
    }
}

/// Oklab linear interpolation
pub fn mix_oklab(a: Color, b: Color, t: f32) -> Color {
    // Simplified Oklab mix for now (Linear in RGB is not mandatory, but placeholder)
    // TODO: Real Oklab transformation
    Color(a.0.lerp(b.0, t))
}

/// Generate a glow color based on intensity
pub fn glow(color: Color, intensity: f32) -> Color {
    let mut c = color.0;
    c.w *= intensity;
    Color(c)
}
