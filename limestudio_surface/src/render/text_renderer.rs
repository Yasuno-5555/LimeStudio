//! Glyphon Integration — Measurement-grade Typography.
//!
//! Focus on JetBrains Mono for readable numbers and code.

use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache,
    TextArea, TextAtlas, TextRenderer, Viewport,
};
use wgpu::{Device, MultisampleState, Queue, RenderPass, TextureFormat};
pub struct TextBlock {
    pub buffer: Buffer,
    pub pos: glam::Vec2,
    pub color: glyphon::Color,
}

pub struct TypographySystem {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub atlas: TextAtlas,
    pub renderer: TextRenderer,
    pub cache: Cache,
    pub viewport: Viewport,
    pub blocks: Vec<TextBlock>,
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
            blocks: Vec::new(),
        }
    }

    pub fn resize(&mut self, queue: &Queue, width: u32, height: u32) {
        self.viewport.update(queue, Resolution { width, height });
    }

    /// Clear all temporary text buffers.
    pub fn clear(&mut self) {
        self.blocks.clear();
    }

    /// Prepare a new block of text to be rendered.
    pub fn add_text(
        &mut self,
        text: &str,
        pos: glam::Vec2,
        font_size: f32,
        line_height: f32,
        color: crate::color::Color,
    ) {
        let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(font_size, line_height));
        buffer.set_size(&mut self.font_system, Some(800.0), Some(600.0)); // Max bounds

        let attrs = Attrs::new().family(Family::Monospace);
        let (r, g, b, a) = color.to_rgba_u8();

        buffer.set_text(&mut self.font_system, text, attrs, Shaping::Advanced);
        buffer.shape_until_scroll(&mut self.font_system, true);

        self.blocks.push(TextBlock {
            buffer,
            pos,
            color: Color::rgba(r, g, b, a),
        });
    }

    pub fn render<'a>(&'a mut self, device: &Device, queue: &Queue, rpass: &mut RenderPass<'a>) {
        let text_areas: Vec<TextArea> = self
            .blocks
            .iter()
            .map(|block| {
                TextArea {
                    buffer: &block.buffer,
                    left: block.pos.x,
                    top: block.pos.y,
                    scale: 1.0,
                    bounds: glyphon::TextBounds {
                        left: 0,
                        top: 0,
                        right: 2048, // Large enough for now
                        bottom: 2048,
                    },
                    default_color: block.color,
                    custom_glyphs: &[],
                }
            })
            .collect();

        self.renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                text_areas,
                &mut self.swash_cache,
            )
            .unwrap();

        self.renderer
            .render(&self.atlas, &self.viewport, rpass)
            .unwrap();
    }
}
