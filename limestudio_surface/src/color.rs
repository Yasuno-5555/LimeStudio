//! Lime Surface Color System
//! 
//! Based on Oklab color space for perceptual uniformity.
//! "Mandatory Oklab: All color interpolation occurs in Oklab space."

#![allow(clippy::excessive_precision)]

use glam::{Vec4, Vec4Swizzles};

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
    pub const ERROR: Color = Color(Vec4::new(1.0, 0.0, 0.0, 1.0));
    pub const AMBER: Color = Color(Vec4::new(1.0, 0.6, 0.0, 1.0));
    
    pub const MOD_ACTIVE: Color = Color(Vec4::new(0.0, 1.0, 0.8, 1.0));
    pub const MOD_RANGE: Color = Color(Vec4::new(0.0, 1.0, 0.8, 0.2));

    pub const TEXT_PRIMARY: Color = Color(Vec4::new(0.9, 0.9, 0.95, 1.0));
    pub const TEXT_SECONDARY: Color = Color(Vec4::new(0.6, 0.6, 0.65, 1.0));

    // Authority Layer / Syntax Highlighting (Matte)
    pub const AUTHORITY_BG: Color = Color(Vec4::new(0.03, 0.03, 0.04, 0.95));
    pub const SYNTAX_KEYWORD: Color = Color(Vec4::new(0.6, 0.7, 0.1, 1.0)); // Desaturated Lime
    pub const SYNTAX_TYPE: Color = Color(Vec4::new(0.1, 0.5, 0.8, 1.0));    // Desaturated Blue
    pub const SYNTAX_VAR: Color = Color(Vec4::new(0.8, 0.8, 0.85, 1.0));   // Muted White
    pub const SYNTAX_COMMENT: Color = Color(Vec4::new(0.4, 0.4, 0.45, 1.0)); // Deep Gray
    pub const SYNTAX_VAL: Color = Color(Vec4::new(0.8, 0.5, 0.2, 1.0));     // Desaturated Amber

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

    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", 
            (self.0.x * 255.0) as u8,
            (self.0.y * 255.0) as u8,
            (self.0.z * 255.0) as u8
        )
    }

    pub fn to_array(&self) -> [f32; 4] {
        [self.0.x, self.0.y, self.0.z, self.0.w]
    }

    pub fn to_rgba_u8(&self) -> (u8, u8, u8, u8) {
        (
            (self.0.x * 255.0) as u8,
            (self.0.y * 255.0) as u8,
            (self.0.z * 255.0) as u8,
            (self.0.w * 255.0) as u8,
        )
    }

    pub fn to_glam_vec4(&self) -> glam::Vec4 {
        self.0
    }
}

/// Oklab linear interpolation
pub fn mix_oklab(a: Color, b: Color, t: f32) -> Color {
    let ca = srgb_to_oklab(a.0.xyz());
    let cb = srgb_to_oklab(b.0.xyz());
    let mixed_ok = ca.lerp(cb, t);
    let mixed_srgb = oklab_to_srgb(mixed_ok);
    
    // Maintain alpha in linear space
    let alpha = a.0.w * (1.0 - t) + b.0.w * t;
    Color(glam::Vec4::new(mixed_srgb.x, mixed_srgb.y, mixed_srgb.z, alpha))
}

fn srgb_to_oklab(c: glam::Vec3) -> glam::Vec3 {
    let l = 0.4122214708 * c.x + 0.5363325363 * c.y + 0.0514459929 * c.z;
    let m = 0.2119034982 * c.x + 0.6806995451 * c.y + 0.1073969566 * c.z;
    let s = 0.0883024619 * c.x + 0.2817188376 * c.y + 0.6299787005 * c.z;

    let l_ = l.max(0.0).powf(1.0/3.0);
    let m_ = m.max(0.0).powf(1.0/3.0);
    let s_ = s.max(0.0).powf(1.0/3.0);

    glam::Vec3::new(
        0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720403 * s_,
        1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
        0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_
    )
}

fn oklab_to_srgb(c: glam::Vec3) -> glam::Vec3 {
    let l_ = c.x + 0.3963377774 * c.y + 0.2158037573 * c.z;
    let m_ = c.x - 0.1055613458 * c.y - 0.0638541728 * c.z;
    let s_ = c.x - 0.0894841775 * c.y - 1.2914855480 * c.z;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    glam::Vec3::new(
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s
    )
}
