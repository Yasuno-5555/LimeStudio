

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};
use std::sync::Arc;
use limestudio_surface::SurfaceEngine;
use limestudio_surface::render::SurfaceRenderer;
use limestudio_vpl::engine::VplEngine;
use limestudio_core::transaction::TransactionLayer;
use limestudio_core::pipeline::PipelineFactory;

struct VplApp {
    window: Option<Arc<Window>>,
    renderer: Option<SurfaceRenderer>,
    surface_engine: SurfaceEngine,
    vpl_engine: VplEngine,
    instance: wgpu::Instance,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
    start_time: std::time::Instant,
    cursor_pos: glam::Vec2,
}

impl VplApp {
    fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let (ui_to_eng, eng_to_ui, _, _) = PipelineFactory::create_pair(1024);
        let project = limestudio_core::ProjectSpec::new("Standalone VPL");
        let transaction = TransactionLayer::new(project, ui_to_eng);

        Self {
            window: None,
            renderer: None,
            surface_engine: SurfaceEngine::new(),
            vpl_engine: VplEngine::new(transaction, eng_to_ui),
            instance,
            device: None,
            queue: None,
            surface: None,
            config: None,
            start_time: std::time::Instant::now(),
            cursor_pos: glam::Vec2::ZERO,
        }
    }
}

impl ApplicationHandler for VplApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(Window::default_attributes().with_title("LimeStudio VPL")).unwrap());
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
            WindowEvent::CursorMoved { position, .. } => {
                let pos = glam::Vec2::new(position.x as f32, position.y as f32);
                self.cursor_pos = pos;
                self.vpl_engine.handle_event(limestudio_surface::runtime::input::SurfaceEvent::PointerMove {
                    position: pos,
                    modifiers: Default::default(),
                });
            }
            WindowEvent::MouseInput { state, button, .. } => {
                use winit::event::ElementState;
                use limestudio_surface::runtime::input::MouseButton as SurfaceButton;
                let s_button = match button {
                    winit::event::MouseButton::Left => SurfaceButton::Left,
                    winit::event::MouseButton::Right => SurfaceButton::Right,
                    winit::event::MouseButton::Middle => SurfaceButton::Middle,
                    _ => return,
                };
                
                let pos = self.cursor_pos;
                
                match state {
                    ElementState::Pressed => {
                        self.vpl_engine.handle_event(limestudio_surface::runtime::input::SurfaceEvent::PointerDown {
                            position: pos,
                            button: s_button,
                            modifiers: Default::default(),
                        });
                    }
                    ElementState::Released => {
                        self.vpl_engine.handle_event(limestudio_surface::runtime::input::SurfaceEvent::PointerUp {
                            position: pos,
                            button: s_button,
                            modifiers: Default::default(),
                        });
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let t = self.start_time.elapsed().as_secs_f32();
                
                if let (Some(surface), Some(device), Some(queue), Some(renderer), Some(window)) = 
                    (&self.surface, &self.device, &self.queue, &mut self.renderer, &self.window) 
                {
                    // 1. Build UI from VPL Engine
                    let ui_tree = self.vpl_engine.build_ui();

                    // 2. Sync and Generate Instances
                    self.surface_engine.sync_ui(&ui_tree);
                    let instances = self.surface_engine.generate_instances();

                    // 3. Render
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = VplApp::new();
    event_loop.run_app(&mut app).map_err(|e| anyhow::anyhow!("Winit error: {:?}", e))
}
