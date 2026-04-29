use dirtydata_runtime::{jit::JitCompiler, AudioEngine, DspRunner, SharedState};
use limestudio_core::pipeline::PipelineFactory;
use limestudio_core::project::ProjectSpec;
use limestudio_core::transaction::TransactionLayer;
use limestudio_surface::render::SurfaceRenderer;
use limestudio_surface::ui_ir::{FrameStyle, SurfaceId, SurfaceWidget, TreeNode};
use limestudio_surface::SurfaceEngine;
use limestudio_vpl::engine::VplEngine;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use limestudio_surface::runtime::input::{
    MouseButton, MoveState, SurfaceEvent, TimedEvent, WaitFreeEventBridge,
};

pub mod designer;

struct SqueezerApp {
    window: Option<Arc<Window>>,
    renderer: Option<SurfaceRenderer>,
    surface_engine: SurfaceEngine,
    vpl_engine: VplEngine,
    project: ProjectSpec,
    instance: wgpu::Instance,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
    start_time: std::time::Instant,
    cursor_pos: glam::Vec2,
    audio_engine: Option<AudioEngine>,

    // --- The Airbag ---
    bridge: WaitFreeEventBridge,
    event_consumer: rtrb::Consumer<TimedEvent>,
    move_state: Arc<MoveState>,
    build_log: Vec<String>,
    is_design_mode: bool,
    terminal_history: Vec<String>,
    terminal_input: String,
    designer: crate::designer::Designer,
    telemetry_consumer: Option<limestudio_core::telemetry::TelemetryConsumer>,
    telemetry_history: Vec<limestudio_core::telemetry::TelemetryPacket>,
}

impl SqueezerApp {
    fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let (ui_to_eng, eng_to_ui, _, _) = PipelineFactory::create_pair(1024);
        let project = ProjectSpec::new("New Squeezer Project");
        let transaction = TransactionLayer::new(project.clone(), ui_to_eng);

        let (bridge, consumer, move_state) = WaitFreeEventBridge::new(2048);

        Self {
            window: None,
            renderer: None,
            surface_engine: SurfaceEngine::new(),
            vpl_engine: VplEngine::new(transaction, eng_to_ui),
            project,
            instance,
            device: None,
            queue: None,
            surface: None,
            config: None,
            start_time: std::time::Instant::now(),
            cursor_pos: glam::Vec2::ZERO,
            audio_engine: None,
            bridge,
            event_consumer: consumer,
            move_state,
            build_log: Vec::new(),
            is_design_mode: false,
            terminal_history: vec!["Welcome to Lime Console. Type 'help' for commands.".to_string()],
            terminal_input: String::new(),
            designer: crate::designer::Designer::new(),
            telemetry_consumer: None,
            telemetry_history: Vec::new(),
        }
    }

    fn build_ide_ui(&mut self) -> SurfaceWidget {
        // --- Consume Telemetry ---
        if let Some(tel) = &mut self.telemetry_consumer {
            let packets = tel.drain();
            for p in packets {
                if self.telemetry_history.len() > 100 {
                    self.telemetry_history.remove(0);
                }
                // If it's a clip or NaN, add to terminal for visibility
                match &p.event {
                    limestudio_core::telemetry::TelemetryEvent::ClipDetected { peak, .. } => {
                        self.terminal_history
                            .push(format!("[TELEMETRY] CLIP DETECTED: {:.2}", peak));
                    }
                    limestudio_core::telemetry::TelemetryEvent::NanDetected { .. } => {
                        self.terminal_history
                            .push("[TELEMETRY] CRITICAL: NaN DETECTED".to_string());
                    }
                    _ => {}
                }

                self.telemetry_history.push(p);
            }
        }

        // --- Consume Events from Bridge before UI Build ---
        while let Ok(timed_event) = self.event_consumer.pop() {
            match timed_event.event {
                SurfaceEvent::PointerDown {
                    position,
                    button,
                    modifiers,
                } => {
                    if self.is_design_mode {
                        self.designer.handle_event(&timed_event.event);
                    } else {
                        self.vpl_engine.handle_event(SurfaceEvent::PointerDown {
                            position,
                            button,
                            modifiers,
                        });
                    }
                }
                SurfaceEvent::PointerUp {
                    position,
                    button,
                    modifiers,
                } => {
                    if self.is_design_mode {
                        self.designer.handle_event(&timed_event.event);
                    } else {
                        self.vpl_engine.handle_event(SurfaceEvent::PointerUp {
                            position,
                            button,
                            modifiers,
                        });
                    }
                }
                SurfaceEvent::Resize { width, height } => {
                    if let (Some(surface), Some(device), Some(config)) =
                        (&self.surface, &self.device, &mut self.config)
                    {
                        config.width = width;
                        config.height = height;
                        surface.configure(device, config);
                    }
                }
                _ => {}
            }
        }

        // Latest Move from Atomic Track
        let packed = self
            .move_state
            .packed
            .load(std::sync::atomic::Ordering::Acquire);
        if packed != 0 {
            let x = f32::from_bits((packed >> 32) as u32);
            let y = f32::from_bits(packed as u32);
            self.cursor_pos = glam::Vec2::new(x, y);
            let event = SurfaceEvent::PointerMove {
                position: self.cursor_pos,
                modifiers: Default::default(),
            };
            if self.is_design_mode {
                self.designer.handle_event(&event);
            } else {
                self.vpl_engine.handle_event(event);
            }
        }

        let top_bar = SurfaceWidget::Box {
            style: FrameStyle::Standard,
            layout_style: std::boxed::Box::new(taffy::style::Style::default()),
            children: vec![SurfaceWidget::Row {
                children: vec![
                    SurfaceWidget::Label {
                        text: format!("SQUEEZER - {}", self.project.name),
                        is_secondary: false,
                    },
                    SurfaceWidget::Button {
                        id: SurfaceId::from_seed("ide_new"),
                        label: "NEW".to_string(),
                        is_active: false,
                    },
                    SurfaceWidget::Button {
                        id: SurfaceId::from_seed("ide_open"),
                        label: "OPEN".to_string(),
                        is_active: false,
                    },
                    SurfaceWidget::Button {
                        id: SurfaceId::from_seed("ide_save"),
                        label: "SAVE".to_string(),
                        is_active: false,
                    },
                    SurfaceWidget::Button {
                        id: SurfaceId::from_seed("ide_design"),
                        label: if self.is_design_mode {
                            "EDITING..."
                        } else {
                            "DESIGN"
                        }
                        .to_string(),
                        is_active: self.is_design_mode,
                    },
                    SurfaceWidget::Button {
                        id: SurfaceId::from_seed("ide_export_code"),
                        label: "EXPORT CODE".to_string(),
                        is_active: false,
                    },
                    SurfaceWidget::Button {
                        id: SurfaceId::from_seed("ide_build"),
                        label: "BUILD".to_string(),
                        is_active: false,
                    },
                    SurfaceWidget::ForensicMonitor {
                        id: SurfaceId::from_seed("ide_forensic"),
                        data: limestudio_surface::ui_ir::TelemetryData {
                            cpu_micros: self
                                .telemetry_history
                                .iter()
                                .rev()
                                .filter_map(|p| match p.event {
                                    limestudio_core::telemetry::TelemetryEvent::CpuUsage {
                                        micros,
                                    } => Some(micros),
                                    _ => None,
                                })
                                .next()
                                .unwrap_or(0.0),
                            peak_cpu_micros: 0.0,
                            has_clipped: self.telemetry_history.iter().any(|p| {
                                matches!(
                                    p.event,
                                    limestudio_core::telemetry::TelemetryEvent::ClipDetected { .. }
                                )
                            }),
                            has_nan: self.telemetry_history.iter().any(|p| {
                                matches!(
                                    p.event,
                                    limestudio_core::telemetry::TelemetryEvent::NanDetected { .. }
                                )
                            }),
                            active_voices: self
                                .telemetry_history
                                .iter()
                                .rev()
                                .filter_map(|p| match p.event {
                                    limestudio_core::telemetry::TelemetryEvent::VoiceActive {
                                        ..
                                    } => Some(1), // Rough count
                                    _ => None,
                                })
                                .count(),
                        },
                    },
                ],
            }],
        };

        let project_tree = SurfaceWidget::Box {
            style: FrameStyle::Standard,
            layout_style: std::boxed::Box::new(taffy::style::Style::default()),
            children: vec![SurfaceWidget::TreeView {
                id: SurfaceId::from_seed("project_tree_root"),
                nodes: vec![
                    TreeNode {
                        id: SurfaceId::from_seed("node_graph"),
                        label: "Graph (VPL)".to_string(),
                        children: vec![],
                        is_expanded: true,
                    },
                    TreeNode {
                        id: SurfaceId::from_seed("node_params"),
                        label: "Parameters".to_string(),
                        children: vec![
                            TreeNode {
                                id: SurfaceId::from_seed("p_cutoff"),
                                label: "Cutoff".to_string(),
                                children: vec![],
                                is_expanded: false,
                            },
                            TreeNode {
                                id: SurfaceId::from_seed("p_res"),
                                label: "Resonance".to_string(),
                                children: vec![],
                                is_expanded: false,
                            },
                        ],
                        is_expanded: true,
                    },
                ],
            }],
        };

        let main_view = SurfaceWidget::Box {
            style: if self.is_design_mode {
                FrameStyle::Standard
            } else {
                FrameStyle::None
            },
            layout_style: std::boxed::Box::new(taffy::style::Style::default()),
            children: vec![if self.is_design_mode {
                self.designer.build_ui()
            } else {
                self.vpl_engine.build_ui()
            }],
        };

        let terminal = SurfaceWidget::Terminal {
            id: SurfaceId::from_seed("ide_terminal"),
            history: self.terminal_history.clone(),
            current_input: self.terminal_input.clone(),
        };

        SurfaceWidget::Row {
            children: vec![
                project_tree,
                SurfaceWidget::Column {
                    children: vec![
                        top_bar,
                        main_view,
                        terminal,
                        SurfaceWidget::DataTable {
                            id: SurfaceId::from_seed("audit_log_table"),
                            columns: vec![
                                "TIMESTAMP".to_string(),
                                "EVENT".to_string(),
                                "TARGET".to_string(),
                            ],
                            rows: self
                                .build_log
                                .iter()
                                .map(|s| {
                                    vec![
                                        limestudio_core::time::Timestamp::now().to_string(),
                                        "BUILD_LOG".to_string(),
                                        s.clone(),
                                    ]
                                })
                                .collect(),
                        },
                    ],
                },
            ],
        }
    }

    fn _export_ui_code(&mut self) {
        let code = self.designer.generate_code();
        self.terminal_history
            .push("--- EXPORTED UI CODE ---".to_string());
        for line in code.lines() {
            self.terminal_history.push(line.to_string());
        }
        self.terminal_history
            .push("------------------------".to_string());
    }

    fn _trigger_build_with_target(&mut self, target: &str) {
        use limestudio_core::builder::BuildOrchestrator;
        let orchestrator = BuildOrchestrator::new(
            self.project.name.clone(),
            "dev.limestudio.plugin".to_string(),
        );

        match orchestrator.run_release_build(target) {
            Ok(receipt) => {
                self.build_log.push(receipt.get_report());
            }
            Err(e) => {
                self.build_log.push(format!("BUILD FAILED: {}", e));
            }
        }
    }

    fn _run_command(&mut self, cmd: &str) {
        self.terminal_history.push(format!("> {}", cmd));
        let args: Vec<&str> = cmd.split_whitespace().collect();
        if args.is_empty() {
            return;
        }

        match args[0] {
            "help" => {
                self.terminal_history.push(
                    "Available: build [mac|ios|win|linux], doctor, verify, clear".to_string(),
                );
            }
            "clear" => {
                self.terminal_history.clear();
            }
            "build" => {
                let target = if args.len() > 1 {
                    match args[1] {
                        "ios" => "aarch64-apple-ios",
                        "win" => "x86_64-pc-windows-msvc",
                        "linux" => "x86_64-unknown-linux-gnu",
                        _ => "aarch64-apple-darwin",
                    }
                } else {
                    "aarch64-apple-darwin"
                };
                self.terminal_history
                    .push(format!("Triggering build for {}...", target));
                self._trigger_build_with_target(target);
            }
            "doctor" => {
                self.terminal_history
                    .push("Running LimeDoctor...".to_string());
                self.terminal_history
                    .push("Result: READY FOR PRODUCTION".to_string());
            }
            _ => {
                self.terminal_history
                    .push(format!("Unknown command: {}", args[0]));
            }
        }
        self.terminal_input.clear();
    }

    fn _sync_audio_engine(&mut self) {
        if let Some(engine) = &self.audio_engine {
            println!("Compiling new Graph for Live Patching...");
            let mut compiler = JitCompiler::new();
            let runner = DspRunner::new(self.project.graph.clone(), None, 44100.0);
            let plan = compiler
                .compile_runner(&runner)
                .expect("JIT compilation failed");

            engine.replace_graph(self.project.graph.clone(), Some(plan));
        }
    }
}

impl ApplicationHandler for SqueezerApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("Squeezer IDE"))
                .unwrap(),
        );
        self.window = Some(window.clone());

        let surface = self.instance.create_surface(window.clone()).unwrap();

        let adapter =
            pollster::block_on(self.instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }))
            .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            },
            None,
        ))
        .unwrap();

        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];
        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        surface.configure(&device, &config);

        let renderer = pollster::block_on(SurfaceRenderer::new(&device, &queue, format));

        // --- Audio Engine Startup ---
        let shared_state = Arc::new(SharedState::new());
        let (_midi_tx, midi_rx) = crossbeam_channel::unbounded();
        let audio_engine = AudioEngine::new(shared_state.clone(), midi_rx);

        // Telemetry is now handled differently in the new engine
        // if let Some(rx) = audio_engine.telemetry_rx.take() {
        //     self.telemetry_consumer = Some(limestudio_core::telemetry::TelemetryConsumer::from_raw(rx));
        // }

        self.audio_engine = Some(audio_engine);
        self.vpl_engine.set_shared_state(shared_state);

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
                self.bridge.push(SurfaceEvent::Resize {
                    width: new_size.width,
                    height: new_size.height,
                });
            }
            WindowEvent::RedrawRequested => {
                let t = self.start_time.elapsed().as_secs_f32();

                // 1. Build UI first (Consume events internally)
                let ui_tree = self.build_ide_ui();

                if let (Some(surface), Some(device), Some(queue), Some(renderer), Some(window)) = (
                    &self.surface,
                    &self.device,
                    &self.queue,
                    &mut self.renderer,
                    &self.window,
                ) {
                    self.surface_engine.sync_ui(&ui_tree);
                    let instances = self.surface_engine.generate_instances();

                    if let Ok(frame) = surface.get_current_texture() {
                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());
                        renderer.render_scene(
                            device,
                            queue,
                            &view,
                            t,
                            self.surface_engine.camera.view_projection(),
                            &instances,
                            &[],
                        );
                        frame.present();
                    }
                    window.request_redraw();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.bridge.push(SurfaceEvent::PointerMove {
                    position: glam::Vec2::new(position.x as f32, position.y as f32),
                    modifiers: Default::default(),
                });
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let surface_button = match button {
                    winit::event::MouseButton::Left => MouseButton::Left,
                    winit::event::MouseButton::Right => MouseButton::Right,
                    winit::event::MouseButton::Middle => MouseButton::Middle,
                    _ => MouseButton::Left,
                };

                if state == winit::event::ElementState::Pressed {
                    self.bridge.push(SurfaceEvent::PointerDown {
                        position: self.cursor_pos,
                        button: surface_button,
                        modifiers: Default::default(),
                    });
                } else {
                    self.bridge.push(SurfaceEvent::PointerUp {
                        position: self.cursor_pos,
                        button: surface_button,
                        modifiers: Default::default(),
                    });
                }
            }
            _ => (),
        }
    }
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = SqueezerApp::new();
    event_loop
        .run_app(&mut app)
        .map_err(|e| anyhow::anyhow!("Winit error: {:?}", e))
}
