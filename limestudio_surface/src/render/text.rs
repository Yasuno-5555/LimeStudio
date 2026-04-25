//! Glyphon Integration — Measurement-grade Typography.
//! 
//! Focus on JetBrains Mono for readable numbers.

use glyphon::{FontSystem, SwashCache, TextAtlas, TextRenderer};
use wgpu::{Device, Queue, TextureFormat, MultisampleState};

pub struct TypographySystem {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub atlas: TextAtlas,
    pub renderer: TextRenderer,
}

impl TypographySystem {
    pub fn new(device: &Device, queue: &Queue, format: TextureFormat) -> Self {
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let mut atlas = TextAtlas::new(device, queue, format);
        let renderer = TextRenderer::new(&mut atlas, device, MultisampleState::default(), None);

        Self {
            font_system,
            swash_cache,
            atlas,
            renderer,
        }
    }

    /// Render a string at a specific position
    pub fn prepare_text(
        &mut self,
        _device: &Device,
        _queue: &Queue,
        _text: &str,
        _pos: glam::Vec2,
        _size: f32,
        _color: crate::color::Color,
    ) {
        // glyphon buffering logic here
    }
}
