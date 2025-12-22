use limestudio_core::{AudioProcessor, AudioBuffer, ProcessContext};
use nih_plug::prelude::*;
use std::sync::Arc;
use std::num::NonZeroU32;

/// nih_plugのBufferをLimestudioのAudioBufferとして扱うためのラッパー
struct NihPlugBufferWrapper<'a> {
    buffer: &'a mut Buffer<'a>,
}

impl<'a> AudioBuffer for NihPlugBufferWrapper<'a> {
    #[inline]
    fn channels(&self) -> usize {
        self.buffer.channels()
    }

    #[inline]
    fn samples(&self) -> usize {
        self.buffer.samples()
    }

    #[inline]
    fn channel(&self, ch: usize) -> &[f32] {
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
        let ctx = ProcessContext::new(
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
