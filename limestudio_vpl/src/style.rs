#![allow(clippy::excessive_precision)]

pub mod colors {
    pub const MAIN_BG: &str = "#111112";
    pub const SURFACE: &str = "#1C1C1E";
    pub const SURFACE_GLASS: &str = "#1C1C1EAA";
    pub const GRID: &str = "#2C2C2E";
    pub const CALM_LIME: &str = "#A3D977"; // Slightly brighter for polish
    pub const MUTED_AMBER: &str = "#FF6B6B"; // More vibrant trust alert
    pub const TEXT_PRIMARY: &str = "#FFFFFF";
    pub const TEXT_SECONDARY: &str = "#A1A1AA";
    pub const ACCENT_BLUE: &str = "#0A84FF";
}

pub mod dimen {
    pub const CORNER_RADIUS: f32 = 6.0; // Softer corners
    pub const FONT_SIZE_MAIN: f32 = 13.0;
    pub const FONT_SIZE_SUB: f32 = 10.0;
    pub const PADDING: f32 = 12.0;
}

pub mod anim {
    pub const TRANSITION_MS: f32 = 60.0;
    
    /// Linear Lerp for UI transitions
    pub fn lerp(current: f32, target: f32, dt: f32) -> f32 {
        let lerp_factor = dt / (TRANSITION_MS / 1000.0);
        current + (target - current) * lerp_factor.clamp(0.0, 1.0)
    }
}

pub mod oklab {
    /// Oklab color interpolation for "Trust UI"
    /// Converts sRGB to OKLab, mixes, then back to sRGB.
    pub fn mix_oklab(a_hex: &str, b_hex: &str, t: f32) -> [u8; 3] {
        let a = hex_to_linear(a_hex);
        let b = hex_to_linear(b_hex);
        
        let a_ok = linear_srgb_to_oklab(a);
        let b_ok = linear_srgb_to_oklab(b);
        
        let mixed_ok = [
            a_ok[0] + (b_ok[0] - a_ok[0]) * t,
            a_ok[1] + (b_ok[1] - a_ok[1]) * t,
            a_ok[2] + (b_ok[2] - a_ok[2]) * t,
        ];
        
        let mixed_linear = oklab_to_linear_srgb(mixed_ok);
        linear_to_srgb_bytes(mixed_linear)
    }

    fn hex_to_linear(hex: &str) -> [f32; 3] {
        let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(0) as f32 / 255.0;
        let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(0) as f32 / 255.0;
        let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(0) as f32 / 255.0;
        [r.powf(2.2), g.powf(2.2), b.powf(2.2)]
    }

    fn linear_to_srgb_bytes(l: [f32; 3]) -> [u8; 3] {
        [
            (l[0].powf(1.0 / 2.2).clamp(0.0, 1.0) * 255.0) as u8,
            (l[1].powf(1.0 / 2.2).clamp(0.0, 1.0) * 255.0) as u8,
            (l[2].powf(1.0 / 2.2).clamp(0.0, 1.0) * 255.0) as u8,
        ]
    }

    fn linear_srgb_to_oklab(c: [f32; 3]) -> [f32; 3] {
        let l = 0.4122214708 * c[0] + 0.5363325363 * c[1] + 0.0514459929 * c[2];
        let m = 0.2119034982 * c[0] + 0.6806995451 * c[1] + 0.1073969566 * c[2];
        let s = 0.0883024619 * c[0] + 0.2817188376 * c[1] + 0.6299787005 * c[2];

        let l_ = l.powf(1.0 / 3.0);
        let m_ = m.powf(1.0 / 3.0);
        let s_ = s.powf(1.0 / 3.0);

        [
            0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720401 * s_,
            1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
            0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_,
        ]
    }

    fn oklab_to_linear_srgb(c: [f32; 3]) -> [f32; 3] {
        let l_ = c[0] + 0.3963377774 * c[1] + 0.2158037573 * c[2];
        let m_ = c[0] - 0.1055613458 * c[1] - 0.0638541728 * c[2];
        let s_ = c[0] - 0.0894841775 * c[1] - 1.2914855480 * c[2];

        let l = l_.powf(3.0);
        let m = m_.powf(3.0);
        let s = s_.powf(3.0);

        [
            4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
            -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
            -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
        ]
    }
}
