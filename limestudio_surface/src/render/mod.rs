pub mod sdf;
pub mod text;

// Placeholder for wgpu initialization and main loop
pub struct SurfaceRenderer {
    pub typography: text::TypographySystem,
}

impl SurfaceRenderer {
    pub async fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let typography = text::TypographySystem::new(device, queue, format);
        Self { typography }
    }

    pub fn render(&mut self, _scene: &crate::scene::SurfaceScene) {
        // GPU batching logic here
    }
}
