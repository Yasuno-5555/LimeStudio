//! LimeStudio Trust UI Design System
//! 
//! Matte & Solid. No gradients, no glows. Shape over Color.

pub mod colors {
    pub const MAIN_BG: &str = "#18181A";
    pub const SURFACE: &str = "#242427";
    pub const GRID: &str = "#333336";
    pub const CALM_LIME: &str = "#8CB369";
    pub const MUTED_AMBER: &str = "#D9735A";
    pub const TEXT_PRIMARY: &str = "#EBEBF5";
    pub const TEXT_SECONDARY: &str = "#8E8E93";
}

pub mod dimen {
    pub const CORNER_RADIUS: f32 = 4.0;
    pub const FONT_SIZE_MAIN: f32 = 14.0;
    pub const FONT_SIZE_SUB: f32 = 11.0;
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
    // Oklab color interpolation helper placeholder
    // Real implementation would involve sRGB -> Linear -> Oklab -> Mix -> sRGB
    pub fn mix_oklab(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
        // TODO: Full implementation of Oklab interpolation for "Trust UI"
        [
            a[0] + (b[0] - a[0]) * t,
            a[1] + (b[1] - a[1]) * t,
            a[2] + (b[2] - a[2]) * t,
        ]
    }
}
