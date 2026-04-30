use crate::interaction::{InteractionEvent, InteractionStore};
use crate::observation::{ObservationConsumer, ObservationEvent};
use crate::ui::Widget;
use limestudio_surface::render::SurfaceRenderer;
use limestudio_surface::SurfaceEngine;
use nih_plug::prelude::*;
use once_cell::sync::Lazy;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use rtrb::RingBuffer;
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Global WGPU resources shared across all plugin instances.
struct SharedWgpu {
    instance: wgpu::Instance,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

static SHARED_WGPU: Lazy<Option<Arc<SharedWgpu>>> = Lazy::new(|| {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    pollster::block_on(async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("LimeStudio Shared Device"),
                    ..Default::default()
                },
                None,
            )
            .await
            .ok()?;

        Some(Arc::new(SharedWgpu {
            instance,
            device,
            queue,
        }))
    })
});

/// UI側で保持する「Realityの観測結果」
#[derive(Clone)]
pub struct ObservationState {
    pub peak_left: f32,
    pub peak_right: f32,
    pub waveform: [f32; crate::observation::WAVEFORM_SAMPLES],
    pub spectrum: [f32; crate::observation::SPECTRUM_BINS],
    pub cpu_usage: f32,
}

impl Default for ObservationState {
    fn default() -> Self {
        Self {
            peak_left: 0.0,
            peak_right: 0.0,
            waveform: [0.0; crate::observation::WAVEFORM_SAMPLES],
            spectrum: [0.0; crate::observation::SPECTRUM_BINS],
            cpu_usage: 0.0,
        }
    }
}

pub struct SurfaceEditor<P, F> {
    params: Arc<P>,
    ui_build: F,
    obs_consumer: Arc<Mutex<Option<ObservationConsumer>>>,
    width: u32,
    height: u32,
}

impl<P, F> SurfaceEditor<P, F>
where
    P: Params + 'static,
    F: for<'a> Fn(&'a P, &'a ObservationState, &mut crate::UiContext) -> Box<dyn Widget + 'a>
        + Send
        + Sync
        + Clone
        + 'static,
{
    pub fn new(
        params: Arc<P>,
        ui_build: F,
        obs_consumer: Option<ObservationConsumer>,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            params,
            ui_build,
            obs_consumer: Arc::new(Mutex::new(obs_consumer)),
            width,
            height,
        }
    }
}

/// The state of a spawned editor instance.
struct EditorInstance {
    is_alive: Arc<Mutex<bool>>,
    poll_thread: Option<std::thread::JoinHandle<()>>,
}

impl<P, F> Editor for SurfaceEditor<P, F>
where
    P: Params + 'static,
    F: for<'a> Fn(&'a P, &'a ObservationState, &mut crate::UiContext) -> Box<dyn Widget + 'a>
        + Send
        + Sync
        + Clone
        + 'static,
{
    fn spawn(
        &self,
        parent: ParentWindowHandle,
        _context: Arc<dyn GuiContext>,
    ) -> Box<dyn Any + Send> {
        let wgpu = match &*SHARED_WGPU {
            Some(w) => w,
            None => {
                return Box::new(EditorInstance {
                    is_alive: Arc::new(Mutex::new(false)),
                    poll_thread: None,
                })
            }
        };

        let engine = Arc::new(Mutex::new(SurfaceEngine::new()));
        let is_alive = Arc::new(Mutex::new(true));
        let ui_build = self.ui_build.clone();
        let params = self.params.clone();
        let mut obs_consumer = self.obs_consumer.lock().unwrap().take();

        let width = self.width;
        let height = self.height;

        let (mut event_tx, mut event_rx) = RingBuffer::<InteractionEvent>::new(1024);
        let interaction_store = Arc::new(InteractionStore::new());
        let param_map: HashMap<String, ParamPtr> = params
            .param_map()
            .into_iter()
            .map(|(id, ptr, _group)| (id.to_string(), ptr))
            .collect();

        let context_poll = _context.clone();
        let is_alive_poll = is_alive.clone();
        let store_poll = interaction_store.clone();

        let poll_thread = std::thread::spawn(move || {
            let mut last_applied_seq = HashMap::new();
            while *is_alive_poll.lock().unwrap() {
                while let Ok(event) = event_rx.pop() {
                    match event {
                        InteractionEvent::BeginDrag { param_id, seq_id } => {
                            if let Some(ptr) = param_map.get(&param_id) {
                                unsafe { context_poll.raw_begin_set_parameter(*ptr) };
                            }
                            last_applied_seq.insert(param_id, seq_id);
                        }
                        InteractionEvent::Drag { param_id, seq_id } => {
                            let last_seq = last_applied_seq.get(&param_id).cloned().unwrap_or(0);
                            if seq_id > last_seq {
                                let val = store_poll.get_value(&param_id);
                                if let Some(ptr) = param_map.get(&param_id) {
                                    unsafe { context_poll.raw_set_parameter_normalized(*ptr, val) };
                                }
                                last_applied_seq.insert(param_id, seq_id);
                            }
                        }
                        InteractionEvent::EndDrag { param_id, seq_id } => {
                            if let Some(ptr) = param_map.get(&param_id) {
                                unsafe { context_poll.raw_end_set_parameter(*ptr) };
                            }
                            last_applied_seq.insert(param_id, seq_id);
                        }
                    }
                }
                std::thread::sleep(Duration::from_millis(1));
            }
        });

        // Window handle conversion
        let (window_handle, display_handle) = match parent {
            #[cfg(target_os = "linux")]
            ParentWindowHandle::X11Window(window) => {
                let h = raw_window_handle::XlibWindowHandle::new(window as std::os::raw::c_ulong);
                (
                    RawWindowHandle::Xlib(h),
                    RawDisplayHandle::Xlib(raw_window_handle::XlibDisplayHandle::new(None, 0)),
                )
            }
            #[cfg(target_os = "windows")]
            ParentWindowHandle::Win32Hwnd(hwnd) => {
                let h = raw_window_handle::Win32WindowHandle::new(
                    std::num::NonZeroIsize::new(hwnd as isize).unwrap(),
                );
                (
                    RawWindowHandle::Win32(h),
                    RawDisplayHandle::Windows(raw_window_handle::WindowsDisplayHandle::new()),
                )
            }
            #[cfg(target_os = "macos")]
            ParentWindowHandle::AppKitNsView(nsview) => {
                let h = raw_window_handle::AppKitWindowHandle::new(
                    std::ptr::NonNull::new(nsview).unwrap(),
                );
                (
                    RawWindowHandle::AppKit(h),
                    RawDisplayHandle::AppKit(raw_window_handle::AppKitDisplayHandle::new()),
                )
            }
            _ => panic!("Unsupported platform"),
        };

        let surface = unsafe {
            wgpu.instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_display_handle: display_handle,
                    raw_window_handle: window_handle,
                })
                .unwrap()
        };

        let format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&wgpu.device, &config);

        let mut renderer =
            pollster::block_on(SurfaceRenderer::new(&wgpu.device, &wgpu.queue, format));

        let is_alive_thread = is_alive.clone();
        let shared_wgpu = wgpu.clone();
        let mut obs_state = ObservationState::default();
        let mut seq_id_counter = 0u64;
        let mut local_state = std::collections::HashMap::new();

        std::thread::spawn(move || {
            let mut last_t = Instant::now();
            while *is_alive_thread.lock().unwrap() {
                let now = Instant::now();
                let _dt = now.duration_since(last_t).as_secs_f32();
                last_t = now;

                // 1. Drain Observation Vascular System
                if let Some(obs) = &mut obs_consumer {
                    for event in obs.drain() {
                        match event {
                            ObservationEvent::Peak { left, right } => {
                                obs_state.peak_left = left;
                                obs_state.peak_right = right;
                            }
                            ObservationEvent::Waveform(data) => {
                                obs_state.waveform = data;
                            }
                            ObservationEvent::Spectrum(data) => {
                                obs_state.spectrum = data;
                            }
                            ObservationEvent::PerfStats { cpu_usage, .. } => {
                                obs_state.cpu_usage = cpu_usage;
                            }
                            _ => {}
                        }
                    }
                }

                // 2. Projection (incorporating ObservationState & LocalState)
                let mut ctx = crate::UiContext {
                    state_store: &mut local_state,
                };
                let tree = (ui_build)(&params, &obs_state, &mut ctx);
                let ir = tree.build();

                // 3. Reconciliation & Render
                if let Ok(mut engine) = engine.lock() {
                    engine.sync_ui(&ir);
                    let instances = engine.generate_instances();

                    // 4. Intent Bridge (Closure of the loop)
                    // In a real scenario, events would be fed from the OS thread via WaitFreeEventBridge.
                    // Here we simulate the feed by checking for any intents generated during interaction.
                    let intents = engine.take_intents();

                    for intent in intents {
                        match intent {
                            limestudio_surface::runtime::interaction_kernel::InteractionIntent::UpdateParameter { parameter: _, value, node_id } => {
                                let param_id = node_id.to_string();
                                interaction_store.set_value(&param_id, value);
                                seq_id_counter += 1;
                                let _ = event_tx.push(InteractionEvent::Drag {
                                    param_id,
                                    seq_id: seq_id_counter,
                                });
                            }
                            _ => {}
                        }
                    }

                    if let Ok(output) = surface.get_current_texture() {
                        let view = output
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());
                        renderer.render_scene(
                            &shared_wgpu.device,
                            &shared_wgpu.queue,
                            &view,
                            0.0,
                            engine.camera.view_projection(),
                            &instances,
                            &[],
                        );
                        output.present();
                    }
                }

                std::thread::sleep(Duration::from_millis(16));
            }
        });

        Box::new(EditorInstance {
            is_alive,
            poll_thread: Some(poll_thread),
        })
    }

    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn set_scale_factor(&self, _factor: f32) -> bool {
        true
    }

    fn param_value_changed(&self, _id: &str, _normalized_value: f32) {}
    fn param_modulation_changed(&self, _id: &str, _modulation_offset: f32) {}
    fn param_values_changed(&self) {}
}

impl Drop for EditorInstance {
    fn drop(&mut self) {
        if let Ok(mut alive) = self.is_alive.lock() {
            *alive = false;
        }
        if let Some(handle) = self.poll_thread.take() {
            let _ = handle.join();
        }
    }
}
