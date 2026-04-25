//! Lime Surface Runtime
//! 
//! winit + wgpu integration. Handles the window lifecycle and render loop.

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use crate::render::SurfaceRenderer;
use crate::SurfaceEngine;

pub struct SurfaceRuntime {
    pub engine: SurfaceEngine,
    pub renderer: Option<SurfaceRenderer>,
}

impl SurfaceRuntime {
    pub fn new() -> Self {
        Self {
            engine: SurfaceEngine::new(),
            renderer: None,
        }
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        let event_loop = EventLoop::new()?;
        let window = std::sync::Arc::new(WindowBuilder::new()
            .with_title("LimeStudio Surface")
            .build(&event_loop)?);

        // WGPU initialization
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone())?;
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }).await.ok_or_else(|| anyhow::anyhow!("No adapter found"))?;

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default(), None).await?;
        let swapchain_format = surface.get_capabilities(&adapter).formats[0];

        let mut renderer = SurfaceRenderer::new(&device, &queue, swapchain_format).await;

        event_loop.run(move |event, window_target| {
            window_target.set_control_flow(ControlFlow::Poll);

            match event {
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    window_target.exit();
                }
                Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                    self.engine.profiler.begin_frame();
                    renderer.render(&self.engine.scene);
                    self.engine.profiler.end_frame();
                    window.request_redraw();
                }
                _ => {}
            }
        })?;

        Ok(())
    }
}
