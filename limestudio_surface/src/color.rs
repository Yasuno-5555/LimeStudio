//! Lime Surface Color System
//! 
//! sRGB interpolation is forbidden.
//! Mandatory: Oklab interpolation for perceptual consistency.

use glam::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn from_hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap() as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap() as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap() as f32 / 255.0;
        Self { r, g, b, a: 1.0 }
    }

    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.r, self.g, self.b)
    }
}

/// sRGB to Linear sRGB
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Linear sRGB to sRGB
fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Linear sRGB to Oklab
pub fn linear_srgb_to_oklab(c: Vec3) -> Vec3 {
    let l = 0.4122214708 * c.x + 0.5363325363 * c.y + 0.0514459929 * c.z;
    let m = 0.2119034982 * c.x + 0.6806995451 * c.y + 0.1073969566 * c.z;
    let s = 0.0883024619 * c.x + 0.2817188376 * c.y + 0.6299787005 * c.z;

    let l_ = l.powf(1.0 / 3.0);
    let m_ = m.powf(1.0 / 3.0);
    let s_ = s.powf(1.0 / 3.0);

    Vec3::new(
        0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720403 * s_,
        1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
        0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_,
    )
}

/// Oklab to Linear sRGB
pub fn oklab_to_linear_srgb(c: Vec3) -> Vec3 {
    let l_ = c.x + 0.3963377774 * c.y + 0.2158037573 * c.z;
    let m_ = c.x - 0.1055613458 * c.y - 0.0638541728 * c.z;
    let s_ = c.x - 0.0894841775 * c.y - 1.2914855480 * c.z;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    Vec3::new(
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186143 * m + 1.7076147010 * s,
    )
}

/// Perceptually consistent color interpolation using Oklab
pub fn mix_oklab(color_a: Color, color_b: Color, t: f32) -> Color {
    let lin_a = Vec3::new(srgb_to_linear(color_a.r), srgb_to_linear(color_a.g), srgb_to_linear(color_a.b));
    let lin_b = Vec3::new(srgb_to_linear(color_b.r), srgb_to_linear(color_b.g), srgb_to_linear(color_b.b));

    let lab_a = linear_srgb_to_oklab(lin_a);
    let lab_b = linear_srgb_to_oklab(lin_b);

    let lab_mixed = lab_a + (lab_b - lab_a) * t;
    let lin_mixed = oklab_to_linear_srgb(lab_mixed);

    Color {
        r: linear_to_srgb(lin_mixed.x),
        g: linear_to_srgb(lin_mixed.y),
        b: linear_to_srgb(lin_mixed.z),
        a: color_a.a + (color_b.a - color_a.a) * t,
    }
}
