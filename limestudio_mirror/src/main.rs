use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};
use limestudio_surface::SurfaceEngine;
use limestudio_surface::render::SurfaceRenderer;
use limestudio_surface::ui_ir::{SurfaceWidget, DisplaySignal};
use limestudio_surface::model::stable_id::SurfaceId;
use std::sync::Arc;

struct MirrorApp {
    window: Option<Arc<Window>>,
    renderer: Option<SurfaceRenderer>,
    engine: SurfaceEngine,
    instance: wgpu::Instance,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
    start_time: std::time::Instant,
    cursor_pos: glam::Vec2,
}

impl MirrorApp {
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
            cursor_pos: glam::Vec2::ZERO,
        }
    }

    fn init_wgpu(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> {
        let window = Arc::new(event_loop.create_window(Window::default_attributes().with_title("LimeSurface Mirror v1.0"))?);
        self.window = Some(window.clone());

        let surface = self.instance.create_surface(window.clone())?;
        
        let adapter = pollster::block_on(self.instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })).ok_or_else(|| anyhow::anyhow!("Failed to find a suitable GPU adapter"))?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            },
            None,
        ))?;

        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats.first().copied().ok_or_else(|| anyhow::anyhow!("No supported surface formats"))?;
        let config = surface.get_default_config(&adapter, size.width, size.height).ok_or_else(|| anyhow::anyhow!("Failed to get default surface config"))?;
        surface.configure(&device, &config);

        let renderer = pollster::block_on(SurfaceRenderer::new(&device, &queue, format));

        self.device = Some(device);
        self.queue = Some(queue);
        self.surface = Some(surface);
        self.config = Some(config);
        self.renderer = Some(renderer);

        window.request_redraw();

        Ok(())
}

    fn build_demo_ui(&self, t: f32) -> SurfaceWidget {
        use limestudio_surface::ui_ir::SurfaceWidget::*;

        Column {
            children: vec![
                Box {
                    style: limestudio_surface::ui_ir::FrameStyle::Standard,
                    children: vec![
                        Label { text: "LIME SURFACE MIRROR".to_string(), is_secondary: false },
                        Label { text: format!("T: {:.2}s | Forensic Trace Active", t), is_secondary: true },
                    ],
                },
                Row {
                    children: vec![
                        Column {
                            children: vec![
                                Label { text: "CONTROL RACK".to_string(), is_secondary: false },
                                Knob {
                                    id: SurfaceId::generate(),
                                    label: "CUTOFF".to_string(),
                                    signal: DisplaySignal::Linear((t * 0.5).sin() * 0.5 + 0.5),
                                },
                                Knob {
                                    id: SurfaceId::generate(),
                                    label: "RESO".to_string(),
                                    signal: DisplaySignal::Linear((t * 0.7).cos() * 0.5 + 0.5),
                                },
                                Slider {
                                    id: SurfaceId::generate(),
                                    label: "VOLUME".to_string(),
                                    signal: DisplaySignal::Linear(0.8),
                                    is_vertical: false,
                                },
                            ]
                        },
                        Column {
                            children: vec![
                                Label { text: "ANALYSIS".to_string(), is_secondary: false },
                                Waveform {
                                    id: "mirror_wave".to_string(),
                                    data: (0..128).map(|i| (i as f32 * 0.2 + t * 5.0).sin() * 0.5 + (i as f32 * 0.5).sin() * 0.2).collect(),
                                },
                                Spectrum {
                                    id: "mirror_spec".to_string(),
                                    data: (0..64).map(|i| ((i as f32 * 0.1 - t).sin() * 0.5 + 0.5) * (1.0 - i as f32 / 64.0)).collect(),
                                },
                            ]
                        }
                    ]
                },
                ForensicMonitor {
                    id: SurfaceId::generate(),
                    data: limestudio_surface::ui_ir::TelemetryData {
                        cpu_micros: 42.0 + (t * 10.0).sin() * 10.0,
                        peak_cpu_micros: 60.0,
                        has_nan: false,
                        has_clipped: (t * 2.0).sin() > 0.9,
                        active_voices: 1,
                    },
                }
            ]
        }
    }
}

impl ApplicationHandler for MirrorApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(e) = self.init_wgpu(event_loop) {
            eprintln!("Initialization failed: {:?}", e);
            event_loop.exit();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                if let (Some(surface), Some(device), Some(config)) = (&self.surface, &self.device, &mut self.config) {
                    config.width = new_size.width;
                    config.height = new_size.height;
                    surface.configure(device, config);
                    if let Some(renderer) = &mut self.renderer {
                        renderer.typography.resize(self.queue.as_ref().unwrap(), new_size.width, new_size.height);
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = glam::Vec2::new(position.x as f32, position.y as f32);
            }
            WindowEvent::RedrawRequested => {
                let t = self.start_time.elapsed().as_secs_f32();
                
                    let demo_ui = self.build_demo_ui(t);

                    if let (Some(surface), Some(device), Some(queue), Some(renderer), Some(window)) = 
                        (&self.surface, &self.device, &self.queue, &mut self.renderer, &self.window) 
                    {
                        let size = window.inner_size();
                        let view_proj = glam::Mat4::orthographic_rh(0.0, size.width as f32, size.height as f32, 0.0, -1.0, 1.0);

                        self.engine.sync_ui(&demo_ui);
                        
                        let instances = self.engine.generate_instances();
                        let primitives = self.engine.primitive_stream.lock()
                            .map(|guard| guard.clone())
                            .unwrap_or_default();

                        if let Ok(frame) = surface.get_current_texture() {
                            let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                            renderer.render_scene(device, queue, &view, t, view_proj, &instances, &primitives);
                            frame.present();
                        }

                        window.request_redraw();
                    }
            }
            _ => (),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    println!("Launching LimeSurface Mirror...");
    
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = MirrorApp::new();
    event_loop.run_app(&mut app).map_err(|e| anyhow::anyhow!("Winit error: {:?}", e))
}
