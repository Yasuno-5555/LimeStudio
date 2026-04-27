//! TextLayoutService — The bridge between Semantic Text and GPU Glyphs.
//! 
//! "人生は短いが、法は永遠である。Layout Engine は法に従い、GlyphRun を生成する。"

use crate::ui_ir::{GlyphRun, GlyphPlacement};
use crate::color::Color;

pub struct TextLayoutService {
    // In a real implementation, this would hold font face data, 
    // shaping engine (swash/skribo), and caches.
}

impl TextLayoutService {
    pub fn new() -> Self {
        Self {}
    }

    /// Layout a string into a GlyphRun.
    /// Follows the 8px grid doctrine for positioning.
    pub fn layout_text(&self, text: &str, pos: [f32; 2], scale: f32, color: Color) -> GlyphRun {
        let mut placements = Vec::new();
        let mut x_advance = 0.0;

        for _c in text.chars() {
            // Simplified: every char is 8px wide at scale 1.0
            placements.push(GlyphPlacement {
                glyph_id: 0, // Placeholder
                pos: [pos[0] + x_advance, pos[1]],
                scale,
            });
            x_advance += 8.0 * scale;
        }

        GlyphRun {
            placements,
            color: color.to_array(),
        }
    }
}
