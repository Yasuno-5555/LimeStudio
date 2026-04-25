use limestudio_core::{AudioProcessor, AudioBuffer, ProcessContext as LimeProcessContext};
use limestudio_core::engine::DspEngine;
use limestudio_core::graph::AudioGraph;
use nih_plug::prelude::*;
use std::sync::Arc;
use std::num::NonZeroU32;

/// nih_plugのBufferをLimestudioのAudioBufferとして扱うためのラッパー
struct NihPlugBufferWrapper<'a, 'b> {
    buffer: &'a mut Buffer<'b>,
}

impl<'a, 'b> AudioBuffer for NihPlugBufferWrapper<'a, 'b> {
    #[inline]
    fn channels(&self) -> usize {
        self.buffer.channels()
    }

    #[inline]
    fn samples(&self) -> usize {
        self.buffer.samples()
    }

    #[inline]
    fn channel(&mut self, ch: usize) -> &[f32] {
        // nih_plugのbuffer.samples()はスライスの長さを保証している
        // unsafeを使ってオーバーヘッドを削ることも可能だが、まずは安全に
        unsafe {
             std::slice::from_raw_parts(
                self.buffer.as_slice()[ch].as_ptr(),
                self.buffer.samples()
            )
        }
    }

    #[inline]
    fn channel_mut(&mut self, ch: usize) -> &mut [f32] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.buffer.as_slice()[ch].as_mut_ptr(),
                self.buffer.samples()
            )
        }
    }
}

/// 汎用プラグインアダプター
/// P: 実際の処理を行うプロセッサ
pub struct LimePluginAdapter<P: AudioProcessor + Default + 'static> {
    pub processor: P,
    pub params: Arc<LimeParams>,
}

#[derive(Params)]
pub struct LimeParams {
    // 将来的にはここでパラメータを動的に管理する仕組みを作る
    // 今はプレースホルダー
    #[id = "gain"]
    pub gain: FloatParam,
}

impl Default for LimeParams {
    fn default() -> Self {
        Self {
            gain: FloatParam::new(
                "Gain",
                1.0,
                FloatRange::Linear { min: 0.0, max: 2.0 },
            ),
        }
    }
}

impl<P: AudioProcessor + Default + 'static> Default for LimePluginAdapter<P> {
    fn default() -> Self {
        Self {
            processor: P::default(),
            params: Arc::new(LimeParams::default()),
        }
    }
}

impl<P: AudioProcessor + Default + 'static> Plugin for LimePluginAdapter<P> {
    const NAME: &'static str = "LimeStudio Plugin";
    const VENDOR: &'static str = "LimeStudio";
    const URL: &'static str = "https://limestudio.dev";
    const EMAIL: &'static str = "contact@limestudio.dev";

    const VERSION: &'static str = "0.0.1";

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            aux_input_ports: &[],
            aux_output_ports: &[],
            names: PortNames::const_default(),
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let ctx = LimeProcessContext::new(
            buffer_config.sample_rate as f64,
            buffer_config.max_buffer_size as usize,
            2, // とりあえずステレオ固定
        );
        self.processor.prepare(&ctx);
        
        // Latency Reporting
        let latency = self.processor.latency();
        if latency > 0 {
             _context.set_latency_samples(latency);
        }
        
        true
    }

    fn reset(&mut self) {
        // P側にresetが必要なら追加する
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // パラメータ更新の反映 (簡易版)
        let gain = self.params.gain.value();
        self.processor.set_parameter(0, gain); // ID:0 はゲインとする(仮)

        // Wrapper経由で渡す
        let mut wrapper = NihPlugBufferWrapper { buffer };
        self.processor.process(&mut wrapper);
        
        ProcessStatus::Normal
    }
}

/// プラグインのエクスポートを簡単にするマクロ
#[macro_export]
macro_rules! limestudio_export {
    ($processor:ty) => {
        nih_export_clap!(limestudio_plugin::LimePluginAdapter<$processor>);
        nih_export_vst3!(limestudio_plugin::LimePluginAdapter<$processor>);
    }
}

/// グラフから直接プラグインを構築するためのアダプター (Level 0用)
pub struct RawGraphPlugin {
    pub engine: DspEngine,
    pub params: Arc<EmptyParams>,
}

#[derive(Params)]
pub struct EmptyParams {}

/// Level 0: グラフ定義だけでプラグインをエクスポートするマクロ
#[macro_export]
macro_rules! lime_plugin_raw {
    ($graph_fn:expr) => {
        pub struct LimeRawInstance {
            engine: limestudio_core::engine::DspEngine,
            params: std::sync::Arc<limestudio_plugin::EmptyParams>,
        }

        impl Default for LimeRawInstance {
            fn default() -> Self {
                let graph = $graph_fn();
                let engine = limestudio_core::engine::DspEngine::new(&graph).expect("Failed to create DspEngine");
                Self {
                    engine,
                    params: std::sync::Arc::new(limestudio_plugin::EmptyParams {}),
                }
            }
        }

        impl nih_plug::prelude::Plugin for LimeRawInstance {
            const NAME: &'static str = "LimeStudio Raw Plugin";
            const VENDOR: &'static str = "LimeStudio";
            const URL: &'static str = "https://limestudio.dev";
            const EMAIL: &'static str = "contact@limestudio.dev";
            const VERSION: &'static str = "0.1.0";

            const AUDIO_IO_LAYOUTS: &'static [nih_plug::prelude::AudioIOLayout] = &[
                nih_plug::prelude::AudioIOLayout {
                    main_input_channels: std::num::NonZeroU32::new(2),
                    main_output_channels: std::num::NonZeroU32::new(2),
                    ..nih_plug::prelude::AudioIOLayout::const_default()
                },
            ];

            const MIDI_INPUT: nih_plug::prelude::MidiConfig = nih_plug::prelude::MidiConfig::None;
            const SAMPLE_ACCURATE_AUTOMATION: bool = true;

            type SysExMessage = ();
            type BackgroundTask = ();

            fn params(&self) -> std::sync::Arc<dyn nih_plug::prelude::Params> {
                self.params.clone()
            }

            fn initialize(
                &mut self,
                _audio_io_layout: &nih_plug::prelude::AudioIOLayout,
                _buffer_config: &nih_plug::prelude::BufferConfig,
                _context: &mut impl nih_plug::prelude::InitContext<Self>,
            ) -> bool {
                true
            }

            fn process(
                &mut self,
                buffer: &mut nih_plug::prelude::Buffer,
                _aux: &mut nih_plug::prelude::AuxiliaryBuffers,
                _context: &mut impl nih_plug::prelude::ProcessContext<Self>,
            ) -> nih_plug::prelude::ProcessStatus {
                let mut inputs = Vec::new();
                let mut outputs = Vec::new();
                
                let num_channels = buffer.channels();
                let num_samples = buffer.samples();
                
                for i in 0..num_channels {
                    unsafe {
                        // Borrow to avoid move
                        let chan_slice = &buffer.as_slice()[i];
                        inputs.push(std::slice::from_raw_parts(chan_slice.as_ptr(), num_samples));
                        
                        // Brutal cast for outputs
                        let out_ptr = chan_slice.as_ptr() as *mut f32;
                        outputs.push(std::slice::from_raw_parts_mut(out_ptr, num_samples));
                    }
                }
                
                self.engine.process_block(&inputs, &mut outputs);
                
                nih_plug::prelude::ProcessStatus::Normal
            }
        }

        impl nih_plug::prelude::ClapPlugin for LimeRawInstance {
            const CLAP_ID: &'static str = "dev.limestudio.raw";
            const CLAP_DESCRIPTION: Option<&'static str> = Some("LimeStudio Raw Plugin");
            const CLAP_MANUAL_URL: Option<&'static str> = None;
            const CLAP_SUPPORT_URL: Option<&'static str> = None;
            const CLAP_FEATURES: &'static [nih_plug::prelude::ClapFeature] = &[
                nih_plug::prelude::ClapFeature::AudioEffect,
                nih_plug::prelude::ClapFeature::Stereo,
            ];
        }

        impl nih_plug::prelude::Vst3Plugin for LimeRawInstance {
            const VST3_CLASS_ID: [u8; 16] = *b"LimeStudioRawPlu";
            const VST3_SUBCATEGORIES: &'static [nih_plug::prelude::Vst3SubCategory] = &[
                nih_plug::prelude::Vst3SubCategory::Fx,
                nih_plug::prelude::Vst3SubCategory::Stereo,
            ];
        }

        nih_plug::nih_export_clap!(LimeRawInstance);
        nih_plug::nih_export_vst3!(LimeRawInstance);
    };
}

/// Level 1.5: ポリフォニックシンセを構築するためのアダプター
#[macro_export]
macro_rules! lime_plugin_poly {
    ($graph_fn:expr, $config:expr) => {
        pub struct LimePolyInstance {
            allocator: limestudio_core::polyphony::VoiceAllocator,
            params: std::sync::Arc<limestudio_plugin::EmptyParams>,
        }

        pub struct PolyConfig {
            pub max_voices: usize,
            pub freq_param: Option<limestudio_core::ir::ParamId>,
            pub gate_param: Option<limestudio_core::ir::ParamId>,
            pub vel_param: Option<limestudio_core::ir::ParamId>,
        }

        impl Default for LimePolyInstance {
            fn default() -> Self {
                let graph = $graph_fn();
                let order = limestudio_core::validate::validate_graph(&graph).expect("Invalid graph");
                let program = limestudio_core::compile::compile_graph(&graph, &order).program;
                
                let config: PolyConfig = $config;
                let mut allocator = limestudio_core::polyphony::VoiceAllocator::new(&program, config.max_voices);
                allocator.set_parameter_mapping(config.freq_param, config.gate_param, config.vel_param);
                
                Self {
                    allocator,
                    params: std::sync::Arc::new(limestudio_plugin::EmptyParams {}),
                }
            }
        }

        impl nih_plug::prelude::Plugin for LimePolyInstance {
            const NAME: &'static str = "LimeStudio Poly Plugin";
            const VENDOR: &'static str = "LimeStudio";
            const URL: &'static str = "https://limestudio.dev";
            const EMAIL: &'static str = "contact@limestudio.dev";
            const VERSION: &'static str = "0.1.0";

            const AUDIO_IO_LAYOUTS: &'static [nih_plug::prelude::AudioIOLayout] = &[
                nih_plug::prelude::AudioIOLayout {
                    main_input_channels: std::num::NonZeroU32::new(2),
                    main_output_channels: std::num::NonZeroU32::new(2),
                    ..nih_plug::prelude::AudioIOLayout::const_default()
                },
            ];

            const MIDI_INPUT: nih_plug::prelude::MidiConfig = nih_plug::prelude::MidiConfig::MidiCCs;
            const SAMPLE_ACCURATE_AUTOMATION: bool = true;

            type SysExMessage = ();
            type BackgroundTask = ();

            fn params(&self) -> std::sync::Arc<dyn nih_plug::prelude::Params> {
                self.params.clone()
            }

            fn initialize(
                &mut self,
                _audio_io_layout: &nih_plug::prelude::AudioIOLayout,
                _buffer_config: &nih_plug::prelude::BufferConfig,
                _context: &mut impl nih_plug::prelude::InitContext<Self>,
            ) -> bool {
                true
            }

            fn process(
                &mut self,
                buffer: &mut nih_plug::prelude::Buffer,
                _aux: &mut nih_plug::prelude::AuxiliaryBuffers,
                context: &mut impl nih_plug::prelude::ProcessContext<Self>,
            ) -> nih_plug::prelude::ProcessStatus {
                let num_samples = buffer.samples();
                
                // Sample-accurate MIDI processing loop (simplified to block-start for now)
                while let Some(event) = context.next_event() {
                    use nih_plug::prelude::NoteEvent::*;
                    match event {
                        NoteOn { note, velocity, .. } => {
                            self.allocator.note_on(note, velocity);
                        }
                        NoteOff { note, .. } => {
                            self.allocator.note_off(note);
                        }
                        _ => {}
                    }
                }

                // Output buffer summation
                for i in 0..num_samples {
                    let (out_l, out_r) = self.allocator.process_sample(0.0, 0.0);
                    
                    buffer.as_slice()[0][i] = out_l;
                    buffer.as_slice()[1][i] = out_r;
                }
                
                nih_plug::prelude::ProcessStatus::Normal
            }
        }

        impl nih_plug::prelude::ClapPlugin for LimePolyInstance {
            const CLAP_ID: &'static str = "dev.limestudio.poly";
            const CLAP_DESCRIPTION: Option<&'static str> = Some("LimeStudio Poly Plugin");
            const CLAP_MANUAL_URL: Option<&'static str> = None;
            const CLAP_SUPPORT_URL: Option<&'static str> = None;
            const CLAP_FEATURES: &'static [nih_plug::prelude::ClapFeature] = &[
                nih_plug::prelude::ClapFeature::Instrument,
                nih_plug::prelude::ClapFeature::Synthesizer,
                nih_plug::prelude::ClapFeature::Stereo,
            ];
        }

        impl nih_plug::prelude::Vst3Plugin for LimePolyInstance {
            const VST3_CLASS_ID: [u8; 16] = *b"LimeStudioPolyPl";
            const VST3_SUBCATEGORIES: &'static [nih_plug::prelude::Vst3SubCategory] = &[
                nih_plug::prelude::Vst3SubCategory::Instrument,
                nih_plug::prelude::Vst3SubCategory::Stereo,
            ];
        }

        nih_plug::nih_export_clap!(LimePolyInstance);
        nih_plug::nih_export_vst3!(LimePolyInstance);
    };
}
