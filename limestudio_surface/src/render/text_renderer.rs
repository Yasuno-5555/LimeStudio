//! Glyphon Integration — Measurement-grade Typography.
//! 
//! Focus on JetBrains Mono for readable numbers and code.

use glyphon::{
    FontSystem, SwashCache, TextAtlas, TextRenderer, 
    Buffer, Metrics, Shaping, Family, Attrs, 
    Resolution, TextArea, Color, Cache, Viewport
};
use wgpu::{Device, Queue, TextureFormat, MultisampleState, RenderPass};
pub struct TypographySystem {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub atlas: TextAtlas,
    pub renderer: TextRenderer,
    pub cache: Cache,
    pub viewport: Viewport,
    pub buffers: Vec<Buffer>,
}

impl TypographySystem {
    pub fn new(device: &Device, queue: &Queue, format: TextureFormat) -> Self {
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let mut atlas = TextAtlas::new(device, queue, &cache, format);
        let renderer = TextRenderer::new(&mut atlas, device, MultisampleState::default(), None);
        let viewport = Viewport::new(device, &cache);

        Self {
            font_system,
            swash_cache,
            atlas,
            renderer,
            cache,
            viewport,
            buffers: Vec::new(),
        }
    }

    pub fn resize(&mut self, queue: &Queue, width: u32, height: u32) {
        self.viewport.update(queue, Resolution { width, height });
    }

    /// Clear all temporary text buffers.
    pub fn clear(&mut self) {
        self.buffers.clear();
    }

    /// Prepare a new block of text to be rendered.
    pub fn add_text(
        &mut self,
        text: &str,
        _pos: glam::Vec2,
        font_size: f32,
        line_height: f32,
        color: crate::color::Color,
    ) {
        let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(font_size, line_height));
        buffer.set_size(&mut self.font_system, Some(800.0), Some(600.0)); // Max bounds
        
        let attrs = Attrs::new().family(Family::Monospace);
        let (_r, _g, _b, _a) = color.to_rgba_u8();
        
        buffer.set_text(&mut self.font_system, text, attrs, Shaping::Advanced);
        buffer.shape_until_scroll(&mut self.font_system, true);

        // We store the position in a wrap or just use it during render
        // For simplicity, we'll assume the caller manages layout and we just buffer it.
        // In a real implementation, we'd store (buffer, pos, color) triplets.
        self.buffers.push(buffer);
    }

    pub fn render<'a>(
        &'a mut self,
        device: &Device,
        queue: &Queue,
        rpass: &mut RenderPass<'a>,
    ) {
        let text_areas: Vec<TextArea> = self.buffers.iter().map(|buffer| {
            TextArea {
                buffer,
                left: 0.0, // TODO: Use real positions
                top: 0.0,
                scale: 1.0,
                bounds: glyphon::TextBounds {
                    left: 0,
                    top: 0,
                    right: 800,
                    bottom: 600,
                },
                default_color: Color::rgb(255, 255, 255),
                custom_glyphs: &[],
            }
        }).collect();

        self.renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            text_areas,
            &mut self.swash_cache,
        ).unwrap();

        self.renderer.render(&self.atlas, &self.viewport, rpass).unwrap();
    }
}
