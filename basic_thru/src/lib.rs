use limestudio_dsp::wrapper::WaveletEngineWrapper;
use nih_plug::prelude::*;
use std::sync::Arc;

mod editor;
mod spectrogram;
use editor::default_state;

pub struct BasicThru {
    params: Arc<BasicThruParams>,

    // DSP Engine
    engine: Option<WaveletEngineWrapper>,

    // Sample Rate
    sample_rate: f32,
}

impl Default for BasicThru {
    fn default() -> Self {
        Self {
            params: Arc::new(BasicThruParams::default()),
            engine: None,
            sample_rate: 44100.0,
        }
    }
}

#[derive(Params)]
pub struct BasicThruParams {
    /// エディタの状態
    #[persist = "editor-state"]
    editor_state: Arc<nih_plug_vizia::ViziaState>,

    #[id = "gain_0"]
    pub gain_0: FloatParam,

    #[id = "gain_1"]
    pub gain_1: FloatParam,

    #[id = "gain_2"]
    pub gain_2: FloatParam,

    #[id = "gain_3"]
    pub gain_3: FloatParam,

    #[id = "gain_4"]
    pub gain_4: FloatParam,
}

impl Default for BasicThruParams {
    fn default() -> Self {
        fn make_gain(name: &str) -> FloatParam {
            FloatParam::new(name, 1.0, FloatRange::Linear { min: 0.0, max: 2.0 })
                .with_smoother(SmoothingStyle::Linear(50.0))
        }

        Self {
            editor_state: default_state(),
            gain_0: make_gain("Low (Scale 0)"),
            gain_1: make_gain("Low-Mid (Scale 1)"),
            gain_2: make_gain("Mid (Scale 2)"),
            gain_3: make_gain("High-Mid (Scale 3)"),
            gain_4: make_gain("High (Scale 4)"),
        }
    }
}

impl Plugin for BasicThru {
    const NAME: &'static str = "Limestudio Thru";
    const VENDOR: &'static str = "Limestudio";
    const URL: &'static str = "https://limestudio.dev";
    const EMAIL: &'static str = "info@limestudio.dev";

    const VERSION: &'static str = "0.1.0";

    // Audio I/O
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        // Monitor Consumer needs to be created fresh each time editor is opened
        let consumer = self.engine.as_mut().and_then(|e| e.attach_monitor());

        editor::create_editor(
            self.params.clone(),
            consumer,
            self.params.editor_state.clone(),
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;

        // Waveletエンジンの初期化
        let engine = WaveletEngineWrapper::new(self.sample_rate as f64, 5); // 5 scales

        // レイテンシ報告 (Context経由ではなく戻り値/trait methodはない、set_latency_samplesはcontextにある)
        // initializeにはcontextがない。processまで待つか？
        // いえ、initializeの `buffer_config` 等からは報告できない。
        // `Plugin::initialize` returns bool.
        // `context` is passed to `process`.
        // However, `nih_plug` handles latency reporting via `context.set_latency_samples` in `process`.
        // We ensure we call it in the first process call or whenever it changes.

        // Note: Monitor creation is deferred to editor() call.

        self.engine = Some(engine);
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl nih_plug::prelude::ProcessContext<Self>,
    ) -> ProcessStatus {
        if let Some(engine) = &mut self.engine {
            // レイテンシ報告 (一度だけで良いが、毎回呼んでもコストは低い)
            // 変更があった場合のみ呼ぶのがベストだが、ここではシンプルに。
            context.set_latency_samples(engine.get_latency());

            // パラメータ反映
            engine.set_scale_gain(0, self.params.gain_0.value() as f64);
            engine.set_scale_gain(1, self.params.gain_1.value() as f64);
            engine.set_scale_gain(2, self.params.gain_2.value() as f64);
            engine.set_scale_gain(3, self.params.gain_3.value() as f64);
            engine.set_scale_gain(4, self.params.gain_4.value() as f64);

            // 処理
            let slices = buffer.as_slice();

            // Channel 0 (Left)
            if !slices.is_empty() {
                // slices[0] is channel 0 data
                let input = slices[0].to_vec();
                engine.process(&input, slices[0]);

                // Copy to Right channel if exists (Mono to Stereo Thru)
                if slices.len() > 1 {
                    // Simple logic: if stereo out, copy L to R
                    let (l, r) = slices.split_at_mut(1);
                    // l[0] is processed.
                    r[0].copy_from_slice(l[0]);
                }
            }
        }
        ProcessStatus::Normal
    }
}

// エクスポート
impl Vst3Plugin for BasicThru {
    const VST3_CLASS_ID: [u8; 16] = *b"LimestudioThru01";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

impl ClapPlugin for BasicThru {
    const CLAP_ID: &'static str = "dev.limestudio.basic-thru";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Basic Thru Plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect];
}

nih_export_clap!(BasicThru);
nih_export_vst3!(BasicThru);

// End of file
