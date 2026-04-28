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
    cursor_pos: glam::Vec2,
    modifiers: crate::runtime::input::Modifiers,
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
            cursor_pos: glam::Vec2::ZERO,
            modifiers: crate::runtime::input::Modifiers::default(),
        }
    }
}


impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(e) = self.init_wgpu(event_loop) {
            eprintln!("Failed to initialize WGPU: {:?}", e);
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
            WindowEvent::ModifiersChanged(modifiers) => {
                let state = modifiers.state();
                self.modifiers = crate::runtime::input::Modifiers {
                    shift: state.shift_key(),
                    ctrl: state.control_key(),
                    alt: state.alt_key(),
                };
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = glam::Vec2::new(position.x as f32, position.y as f32);
                let intents = self.engine.input.handle_event(
                    &crate::runtime::input::SurfaceEvent::PointerMove {
                        position: self.cursor_pos,
                        modifiers: self.modifiers,
                    },
                    &self.engine.camera,
                    &self.engine.node_geometry,
                    &self.engine.port_geometry,
                    &self.engine.widget_geometry,
                );
                self.engine.handle_intents(intents);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let s_button = match button {
                    winit::event::MouseButton::Left => crate::runtime::input::MouseButton::Left,
                    winit::event::MouseButton::Right => crate::runtime::input::MouseButton::Right,
                    winit::event::MouseButton::Middle => crate::runtime::input::MouseButton::Middle,
                    _ => return,
                };

                let event = if state == winit::event::ElementState::Pressed {
                    crate::runtime::input::SurfaceEvent::PointerDown {
                        position: self.cursor_pos,
                        button: s_button,
                        modifiers: self.modifiers,
                    }
                } else {
                    crate::runtime::input::SurfaceEvent::PointerUp {
                        position: self.cursor_pos,
                        button: s_button,
                        modifiers: self.modifiers,
                    }
                };
                let intents = self.engine.input.handle_event(
                    &event,
                    &self.engine.camera,
                    &self.engine.node_geometry,
                    &self.engine.port_geometry,
                    &self.engine.widget_geometry,
                );
                self.engine.handle_intents(intents);
            }



            WindowEvent::RedrawRequested => {
                let t = self.start_time.elapsed().as_secs_f32();
                
                if let (Some(surface), Some(device), Some(queue), Some(renderer), Some(window)) = 
                    (&self.surface, &self.device, &self.queue, &mut self.renderer, &self.window) 
                {
                    let size = window.inner_size();
                    let view_proj = glam::Mat4::orthographic_rh(0.0, size.width as f32, size.height as f32, 0.0, -1.0, 1.0);

                    // Generate test UI
                    use crate::ui_ir::SurfaceWidget::*;
                    use crate::ui_ir::SurfacePrimitive;
                    let test_ui = Column {
                        children: vec![
                            Label { text: "LimeStudio Mirror (V7)".to_string(), is_secondary: false },
                            Row {
                                children: vec![
                                    Knob {
                                        id: crate::model::stable_id::SurfaceId::generate(),
                                        label: "Frequency".to_string(),
                                        signal: crate::ui_ir::DisplaySignal::Linear(t.sin() * 0.5 + 0.5),
                                    },
                                    Knob {
                                        id: crate::model::stable_id::SurfaceId::generate(),
                                        label: "Resonance".to_string(),
                                        signal: crate::ui_ir::DisplaySignal::Linear(t.cos() * 0.5 + 0.5),
                                    },
                                ]
                            },
                            PrimitiveStream {
                                primitives: vec![
                                    SurfacePrimitive::Curve {
                                        id: crate::model::stable_id::SurfaceId::generate(),
                                        control_points: vec![[100.0, 100.0], [500.0, 400.0]], // Will use auto CP logic
                                        kind: crate::ui_ir::CurveKind::Flow { direction: 1.0, phase: 0.0, density: 1.0 },
                                        thickness: 4.0,
                                        color: [0.0, 0.6, 1.0, 0.8],
                                        temporal: crate::ui_ir::TemporalStrategy::Instant,
                                    }
                                ]
                            }
                        ]
                    };

                    self.engine.sync_ui(&test_ui);
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

impl App {
    fn init_wgpu(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> {
        let window = Arc::new(event_loop.create_window(Window::default_attributes().with_title("LimeSurface Mirror"))?);
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

        Ok(())
    }
}

pub async fn run_standalone() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new();
    event_loop.run_app(&mut app).map_err(|e| anyhow::anyhow!("Winit error: {:?}", e))
}

