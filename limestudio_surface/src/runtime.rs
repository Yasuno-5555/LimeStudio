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
use limestudio_core::live::LiveCompiler;
use limestudio_core::engine::DspEngine;
use std::sync::{Arc, Mutex};

pub struct SurfaceRuntime {
    pub engine: SurfaceEngine,
    pub renderer: Option<SurfaceRenderer>,
    pub compiler: LiveCompiler,
    /// リアルタイム実行中のオーディオエンジン
    pub audio_engine: Arc<Mutex<DspEngine>>,
}

impl SurfaceRuntime {
    pub fn new() -> Self {
        let graph = limestudio_core::graph::AudioGraph::new();
        let audio_engine = Arc::new(Mutex::new(
            DspEngine::new(&graph).expect("Failed to initialize default audio engine")
        ));
        
        Self {
            engine: SurfaceEngine::new(),
            renderer: None,
            compiler: LiveCompiler::new(graph),
            audio_engine,
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
                    // Phase 3: Check for graph changes and live-compile
                    if self.engine.canvas.is_dirty {
                        if let Ok(comp) = self.compiler.compile() {
                            let mut audio = self.audio_engine.lock().unwrap();
                            audio.swap_program(Arc::new(comp.program.clone()), comp.graph_version);
                            
                            // Always Show Rust: Generate code for UI
                            let rust_code = limestudio_core::codegen::ir_to_readable_rust(&comp.program);
                            // TODO: Send rust_code to a UI panel
                        }
                        self.engine.canvas.is_dirty = false;
                    }

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
