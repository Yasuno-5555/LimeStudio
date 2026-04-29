use crate::editor::SurfaceEditor;
use crate::observation::{ObservationConsumer, ObservationProducer};
use crate::Widget;
use nih_plug::prelude::*;
use std::cell::RefCell;
use std::sync::Arc;

use limestudio_core::engine::VoiceManager;
use limestudio_core::graph::GraphBuilder;
use limestudio_core::PatchEvent;
use rtrb::{Producer, RingBuffer};

pub trait LimeProcessor: Send + Sync + 'static {
    type Params: Params;
    const NAME: &'static str;
    const VENDOR: &'static str;
    const URL: &'static str;
    const EMAIL: &'static str;
    const VERSION: &'static str;

    fn build_graph(&self, params: Arc<Self::Params>, builder: &RefCell<GraphBuilder>);

    fn params(&self) -> Arc<Self::Params>;

    fn build_ui(params: &Self::Params, obs_consumer: ObservationConsumer) -> Box<dyn Widget + '_>;

    fn initialize(&mut self, _sample_rate: f32) {}

    fn handle_event(&mut self, _event: NoteEvent<()>) {}
}

/// The Forensic Wall. It wraps the raw plugin logic and records all external
/// interactions for transparency and auditability.
pub struct LimeHost<T: LimeProcessor> {
    inner: T,
    last_processed_sample: u64,
    event_log: Vec<String>, // Placeholder for real forensic log
}

impl<T: LimeProcessor> LimeHost<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            last_processed_sample: 0,
            event_log: Vec::new(),
        }
    }

    pub fn process_event(&mut self, event: NoteEvent<()>) {
        // Audit the event
        self.event_log.push(format!(
            "Event at {}: {:?}",
            self.last_processed_sample, event
        ));
        self.inner.handle_event(event);
    }
}

pub struct LimeAdapter<T: LimeProcessor> {
    pub host: LimeHost<T>,
    pub params: Arc<T::Params>,
    pub engine: Option<VoiceManager>,
    pub patch_producer: Option<Producer<PatchEvent>>,
    pub obs_producer: Option<ObservationProducer>,
    pub obs_consumer: Arc<std::sync::Mutex<Option<ObservationConsumer>>>,
    pub sample_rate: f32,
}

impl<T: LimeProcessor> Default for LimeAdapter<T>
where
    T: Default,
{
    fn default() -> Self {
        let (prod, cons) = crate::observation::create_pipeline(512);
        let inner = T::default();
        let params = inner.params();
        Self {
            host: LimeHost::new(inner),
            params,
            engine: None,
            patch_producer: None,
            obs_producer: Some(prod),
            obs_consumer: Arc::new(std::sync::Mutex::new(Some(cons))),
            sample_rate: 44100.0,
        }
    }
}

impl<T: LimeProcessor + Default + Send + 'static> Plugin for LimeAdapter<T> {
    const NAME: &'static str = T::NAME;
    const VENDOR: &'static str = T::VENDOR;
    const URL: &'static str = T::URL;
    const EMAIL: &'static str = T::EMAIL;
    const VERSION: &'static str = T::VERSION;

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: std::num::NonZeroU32::new(2),
        main_output_channels: std::num::NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone() as Arc<dyn Params>
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;
        self.host.inner.initialize(self.sample_rate);

        // Build the graph
        let builder = std::cell::RefCell::new(GraphBuilder::new());
        self.host.inner.build_graph(self.params.clone(), &builder);
        let graph = builder.into_inner().build();

        // Initialize engine
        let (prod, cons) = RingBuffer::<PatchEvent>::new(1024);
        self.patch_producer = Some(prod);
        self.engine = Some(VoiceManager::from_graph(
            &graph,
            cons,
            16,
            self.sample_rate,
            None,
        ));

        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        if let Some(engine) = &mut self.engine {
            while let Some(event) = context.next_event() {
                self.host.process_event(event);
                match event {
                    NoteEvent::NoteOn { note, velocity, .. } => {
                        engine.handle_event(limestudio_core::engine::VoiceEvent::NoteOn {
                            pitch: note,
                            velocity,
                        });
                    }
                    NoteEvent::NoteOff { note, .. } => {
                        engine.handle_event(limestudio_core::engine::VoiceEvent::NoteOff {
                            pitch: note,
                        });
                    }
                    NoteEvent::PolyPressure { note, pressure, .. } => {
                        engine.handle_event(limestudio_core::engine::VoiceEvent::Pressure {
                            pitch: note,
                            value: pressure,
                        });
                    }
                    NoteEvent::PolyTuning { note, tuning, .. } => {
                        engine.handle_event(limestudio_core::engine::VoiceEvent::Tuning {
                            pitch: note,
                            value: tuning,
                        });
                    }
                    _ => {}
                }
            }

            let _num_samples = buffer.samples();
            let slices = buffer.as_slice();
            let ptr = slices.as_ptr();
            let len = slices.len();

            let inputs: &[&[f32]] =
                unsafe { std::slice::from_raw_parts(ptr as *const &[f32], len) };
            let outputs: &mut [&mut [f32]] = slices;

            engine.process(inputs, outputs, self.sample_rate);
            self.host.last_processed_sample += buffer.samples() as u64;
        }

        ProcessStatus::Normal
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();
        let obs_cons_shared = self.obs_consumer.clone();
        let obs_consumer = self.obs_consumer.lock().unwrap().take();
        Some(Box::new(SurfaceEditor::new(
            params,
            move |params_captured, _obs| {
                let mut obs_lock = obs_cons_shared.lock().unwrap();
                let cons = obs_lock.take().unwrap_or_else(|| {
                    let (_, cons) = crate::observation::create_pipeline(512);
                    cons
                });
                T::build_ui(params_captured, cons)
            },
            obs_consumer,
            800,
            600,
        )))
    }
}

impl<T: LimeProcessor + Default + Send + 'static> Vst3Plugin for LimeAdapter<T> {
    const VST3_CLASS_ID: [u8; 16] = *b"LimeStudioPlugin"; // FIXME: Unique ID
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx];
}

impl<T: LimeProcessor + Default + Send + 'static> ClapPlugin for LimeAdapter<T> {
    const CLAP_ID: &'static str = T::NAME;
    const CLAP_DESCRIPTION: Option<&'static str> = Some(T::NAME);
    const CLAP_MANUAL_URL: Option<&'static str> = Some(T::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = Some(T::URL);
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}
