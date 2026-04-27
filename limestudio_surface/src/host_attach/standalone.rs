//! Standalone Verification Tool (The Mirror)
//! 
//! "standalone = 開発者のための鏡。ここで映らなければ、DAWでも映らない。"

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};
use crate::SurfaceEngine;
use crate::render::SurfaceRenderer;
use std::sync::Arc;

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<SurfaceRenderer>,
    engine: SurfaceEngine,
    instance: wgpu::Instance,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
    start_time: std::time::Instant,
}

impl App {
    fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        Self {
            window: None,
            renderer: None,
            engine: SurfaceEngine::new(),
            instance,
            device: None,
            queue: None,
            surface: None,
            config: None,
            start_time: std::time::Instant::now(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(Window::default_attributes().with_title("LimeSurface Mirror")).unwrap());
        self.window = Some(window.clone());

        let surface = self.instance.create_surface(window.clone()).unwrap();
        
        let adapter = pollster::block_on(self.instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })).unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            },
            None,
        )).unwrap();

        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];
        let config = surface.get_default_config(&adapter, size.width, size.height).unwrap();
        surface.configure(&device, &config);

        let renderer = pollster::block_on(SurfaceRenderer::new(&device, &queue, format));

        self.device = Some(device);
        self.queue = Some(queue);
        self.surface = Some(surface);
        self.config = Some(config);
        self.renderer = Some(renderer);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                if let (Some(surface), Some(device), Some(config)) = (&self.surface, &self.device, &mut self.config) {
                    config.width = new_size.width;
                    config.height = new_size.height;
                    surface.configure(device, config);
                }
            }
            WindowEvent::RedrawRequested => {
                let t = self.start_time.elapsed().as_secs_f32();
                
                if let (Some(surface), Some(device), Some(queue), Some(renderer), Some(window)) = 
                    (&self.surface, &self.device, &self.queue, &mut self.renderer, &self.window) 
                {
                    // Generate test UI
                    use crate::ui_ir::SurfaceWidget::*;
                    let test_ui = Column {
                        children: vec![
                            Label { text: "LimeStudio Standalone".to_string(), is_secondary: false },
                            Row {
                                children: vec![
                                    Knob {
                                        id: crate::model::stable_id::SurfaceId(dirtydata_core::types::StableId(ulid::Ulid::nil())),
                                        label: "Test Knob".to_string(),
                                        signal: dirtydata_core::types::DisplaySignal::Linear(t.sin() * 0.5 + 0.5),
                                    },
                                    Slider {
                                        id: crate::model::stable_id::SurfaceId(dirtydata_core::types::StableId(ulid::Ulid::nil())),
                                        label: "Test Slider".to_string(),
                                        signal: dirtydata_core::types::DisplaySignal::Linear(t.cos() * 0.5 + 0.5),
                                        is_vertical: true,
                                    }
                                ]
                            }
                        ]
                    };

                    self.engine.sync_ui(&test_ui);
                    let instances = self.engine.generate_instances();

                    if let Ok(frame) = surface.get_current_texture() {
                        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                        renderer.render_scene(device, queue, &view, t, &instances, &[]);
                        frame.present();
                    }
                    window.request_redraw();
                }
            }
            _ => (),
        }
    }
}

pub async fn run_standalone() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new();
    event_loop.run_app(&mut app).map_err(|e| anyhow::anyhow!("Winit error: {:?}", e))
}
