use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceTheme {
    pub accent_color: [f32; 4],
    pub background_color: [f32; 4],
    pub surface_color: [f32; 4],
    pub text_primary: [f32; 4],
    pub text_secondary: [f32; 4],
    pub border_radius: f32,
    pub spacing: f32,
}

impl Default for SurfaceTheme {
    fn default() -> Self {
        Self {
            accent_color: [0.6, 1.0, 0.4, 1.0], // Lime
            background_color: [0.05, 0.05, 0.05, 1.0],
            surface_color: [0.15, 0.15, 0.15, 1.0],
            text_primary: [0.9, 0.9, 0.9, 1.0],
            text_secondary: [0.6, 0.6, 0.6, 1.0],
            border_radius: 4.0,
            spacing: 8.0,
        }
    }
}
