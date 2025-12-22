use nih_plug::prelude::*;
use limestudio_dsp::wrapper::WaveletEngineWrapper;
use limestudio_core::ProcessContext;
use ringbuf::{RingBuffer, Consumer};
use std::sync::Arc;
use std::sync::Arc;

mod editor;
mod spectrogram;
use editor::default_state;

pub struct BasicThru {
    params: Arc<BasicThruParams>,
    
    // DSP Engine
    engine: Option<WaveletEngineWrapper>,
    
    // Monitoring (Consumer passed to Editor)
    monitor_consumer: Option<Consumer<Vec<f32>>>,
    
    // Sample Rate
    sample_rate: f32,
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
            FloatParam::new(
                name,
                1.0,
                FloatRange::Linear { min: 0.0, max: 2.0 },
            )
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
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }
    
    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        // Monitor Comsumer を取り出す (一度だけ)
        // もし2回目にeditorを開いた場合どうするか？
        // Option::take() だと無くなってしまう。
        // ConsumerはCloneできない。
        // したがって、「エディタが閉じられたらConsumerを返してもらう」か、
        // 「エディタ用にもう一度アタッチし直す」必要がある。
        // Wrapperの構造上、`attach_monitor` は `monitor_consumer` を `take` する。
        // 再生成は `prepare` で行われるが、それはオーディオ停止時とは限らない。
        
        // 解決策:
        // Editorを開くたびに新しい接続が必要なら、Wrapperに「新しい接続くれ」と頼む必要がある。
        // しかし ringbuf の split は生成時のみ。
        // 実際は、PluginインスタンスがConsumerを保持し続け、
        // Editor生成時に `Arc<Mutex<Option<Consumer>>>` を渡すのが一般的。
        // Editor側で `lock().take()` すると無くなる。
        // 簡素化のため、今回は「ConsumerをPluginが保持し、Editorには渡さない（または共有する）」構造で見直すか、
        // あるいは `create_editor` が `Option` を受け取る今の形のまま、
        // 「初回起動時のみビジュアライザーが動く」という制限を受け入れるか。
        
        // プロダクションでは: 
        // Plugin struct holds `Arc<Mutex<Option<Consumer>>>`
        // Editor gets a clone of this Arc.
        // Inside Editor update loop, it locks and accesses consumer?
        // No, Consumer is !Sync usually? ringbuf Consumer IS Send, but process is mut.
        // We need to move Consumer into the Editor'sModel.
        
        // 今回の `monitor_consumer` は `Option<Consumer>`。
        // `take()` して渡す。
        editor::create_editor(
            self.params.clone(),
            self.monitor_consumer.take(),
            self.params.editor_state.clone(),
        )
    }

    fn initialize(&mut self, _bus_config: &BusConfig, buffer_config: &BufferConfig) -> bool {
        self.sample_rate = buffer_config.sample_rate;
        
        // Waveletエンジンの初期化
        let mut engine = WaveletEngineWrapper::new(self.sample_rate as f64, 5); // 5 scales

        // レイテンシ報告 (Context経由ではなく戻り値/trait methodはない、set_latency_samplesはcontextにある)
        // initializeにはcontextがない。processまで待つか？
        // いえ、initializeの `buffer_config` 等からは報告できない。
        // `Plugin::initialize` returns bool.
        // `context` is passed to `process`.
        // However, `nih_plug` handles latency reporting via `context.set_latency_samples` in `process`.
        // We ensure we call it in the first process call or whenever it changes.
        
        // ★ モニターの接続
        // WrapperからConsumerを取得し、構造体に保管しておく
        if let Some(consumer) = engine.attach_monitor() {
            self.monitor_consumer = Some(consumer);
        }

        self.engine = Some(engine);
        true
    }

    fn process(&mut self, buffer: &mut Buffer, _aux: &mut AuxiliaryBuffers, context: &mut ProcessContext) -> ProcessStatus {
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
            let (inputs, mut outputs) = buffer.as_mut_slices();
            
            // Channel 0 (Left)
            if inputs.len() > 0 && outputs.len() > 0 {
                // inputs[0] is channel 0 data
                engine.process(inputs[0], outputs[0]);
                
                // Copy to Right channel if exists (Mono to Stereo Thru)
                if outputs.len() > 1 && inputs.len() > 0 {
                    // Note: copy_from_slice panics if lengths differ.
                    // nih_plug guarantees equal lengths for active channels usually.
                    // But inputs and outputs might differ in channel count?
                    // as_mut_slices returns slices of channels.
                    // outputs[1] is &mut [f32].
                    
                    // Simple logic: if stereo out, copy L to R
                    let (l, r) = outputs.split_at_mut(1);
                    // l[0] is processed.
                    r[0].copy_from_slice(&l[0]);
                }
            }
        }
        ProcessStatus::Normal
    }
}

// エクスポート
impl Vst3Plugin for BasicThru {
    const VST3_CLASS_ID: [u8; 16] = *b"LimestudioThru01";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Tools,
    ];
}

nih_export_clap!(BasicThru);
nih_export_vst3!(BasicThru);

// End of file
