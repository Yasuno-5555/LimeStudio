use crate::processor::WaveletProcessor;
use crate::wavelets::Morlet;
use crate::gatherer::BlockGatherer;
use limestudio_core::{AudioProcessor, AudioContext, ProcessContext};

use ringbuf::{RingBuffer, Producer, Consumer};
use crate::monitor::SpectrumMonitorSender;

// AudioProcessorトレイトを実装し、WaveletProcessorをラップするアダプター
pub struct WaveletEngineWrapper {
    pub processor: WaveletProcessor,
    pub gatherer: BlockGatherer,
    
    // 現在のサンプルレート
    sample_rate: f64,
}

impl WaveletEngineWrapper {
    pub fn new(sample_rate: f64, num_scales: usize) -> Self {
        // 固定パラメータ
        let fft_size = 2048;
        let hop_size = 512; 
        
        let morlet = Morlet::default();
        let processor = WaveletProcessor::new(
            sample_rate,
            fft_size,
            hop_size,
            num_scales,
            &morlet
        );
        
        let gatherer = BlockGatherer::new(fft_size, hop_size);
        
        Self {
            processor,
            gatherer,
            sample_rate,
        }
    }
    
    pub fn attach_monitor(&mut self) -> Option<Consumer<Vec<f32>>> {
        let fft_size = 2048; // Must match the processor's fft_size
        let ring = RingBuffer::<Vec<f32>>::new(16);
        let (producer, consumer) = ring.split();
        
        let monitor_sender = SpectrumMonitorSender::new(producer, fft_size);
        self.processor.set_monitor(monitor_sender);
        
        Some(consumer)
    }
    
    pub fn set_scale_gain(&mut self, scale_idx: usize, gain: f64) {
        // Smoother handles the transition, just set target
        self.processor.set_gain(scale_idx, gain);
    }
    
    pub fn get_latency(&self) -> u32 {
        self.gatherer.latency_samples() // Latency is determined by the Gatherer structure
    }
    
    /// ストリーム処理のメインエントリ
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        // クロージャ内で self.processor をミュータブルに借りるための儀式
        // self.gatherer と self.processor は構造体の別のフィールドなので
        // 個別に借りれば借用チェッカーを通過できる
        let processor = &mut self.processor;
        
        self.gatherer.process_stream(input, output, |in_block, out_block| {
            // ここで固定長の処理が走る
            // input_block を processor に渡し、 output_block に結果をもらう
            // processor.process_block は入力を受け取り、結果を出力スライスに加算または書き込む
            processor.process_block(in_block, out_block);
        });
    }
}

impl AudioProcessor for WaveletEngineWrapper {
    fn prepare(&mut self, context: &ProcessContext) {
        // サンプルレート変更検知
        if (self.sample_rate - context.sample_rate).abs() > 0.1 {
            self.sample_rate = context.sample_rate;

            let fft_size = 2048;
            let hop_size = 512;
            let morlet = Morlet::default();
            
            // Note: Recreating processor loses Smoother states and Monitor connection!
            // The GUI visualization will stop until the editor is reopened or we implement a reconnection mechanism.
            // For this phase, we accept this limitation.
            
            let new_proc = WaveletProcessor::new(
                self.sample_rate,
                fft_size,
                hop_size,
                5, // TODO: keep previous count
                &morlet
            );
            
            self.processor = new_proc;
            
            // Re-create gatherer to clear buffers
            self.gatherer = BlockGatherer::new(fft_size, hop_size);
        }
    }

    fn process<B: limestudio_core::AudioBuffer>(&mut self, buffer: &mut B) {
        let mut out_slice = vec![0.0; buffer.samples()];
        // Assuming channel 0 for now
        let in_slice = buffer.channel(0).to_vec();
        
        // Use the new process loop via gatherer
        self.process(&in_slice, &mut out_slice);
        
        let dest = buffer.channel_mut(0);
        for (i, &val) in out_slice.iter().enumerate() {
            dest[i] = val;
        }
        
        if buffer.channels() > 1 {
             let dest_r = buffer.channel_mut(1);
             for (i, &val) in out_slice.iter().enumerate() {
                 dest_r[i] = val;
             }
        }
    }

    fn latency(&self) -> u32 {
        self.get_latency()
    }
}
