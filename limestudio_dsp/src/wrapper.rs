use crate::processor::WaveletProcessor;
use crate::wavelets::Morlet;
use crate::gatherer::BlockGatherer;
use limestudio_core::{AudioProcessor, ProcessContext};

use std::sync::Arc;
use ringbuf::{HeapRb, Consumer};
use crate::monitor::SpectrumMonitorSender;

// AudioProcessorトレイトを実装し、WaveletProcessorをラップするアダプター
pub struct WaveletEngineWrapper {
    pub processor: WaveletProcessor,
    pub gatherer: BlockGatherer,
    
    // 現在のサンプルレート
    sample_rate: f64,
    
    // アロケーションを避けるためのスクラッチバッファ
    input_scratch: Vec<f32>,
    output_scratch: Vec<f32>,
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
            input_scratch: Vec::with_capacity(4096), // 十分な初期量を確保
            output_scratch: Vec::with_capacity(4096),
        }
    }
    
    #[allow(clippy::type_complexity)]
    pub fn attach_monitor(&mut self) -> Option<Consumer<Vec<f32>, Arc<HeapRb<Vec<f32>>>>> {
        let fft_size = 2048; // Must match the processor's fft_size
        let ring = HeapRb::<Vec<f32>>::new(16);
        let (producer, consumer) = ring.split();
        
        let monitor_sender = SpectrumMonitorSender::new(producer, fft_size);
        self.processor.set_monitor(Box::new(monitor_sender));
        
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
        // スクラッチバッファのサイズを事前に確保 (アロケーションをここで行う)
        if self.input_scratch.len() < context.max_block_size {
            self.input_scratch.resize(context.max_block_size, 0.0);
            self.output_scratch.resize(context.max_block_size, 0.0);
        }

        // サンプルレート変更検知
        if (self.sample_rate - context.sample_rate).abs() > 0.1 {
            self.sample_rate = context.sample_rate;

            let fft_size = 2048;
            let hop_size = 512;
            let morlet = Morlet::default();
            
            let new_proc = WaveletProcessor::new(
                self.sample_rate,
                fft_size,
                hop_size,
                5,
                &morlet
            );
            
            self.processor = new_proc;
            self.gatherer = BlockGatherer::new(fft_size, hop_size);
        }
    }

    fn process<B: limestudio_core::AudioBuffer>(&mut self, buffer: &mut B) {
        let num_samples = buffer.samples();
        let num_channels = buffer.channels();

        // 1. 入力データの取得 (Channel 0のみを暫定的に使用)
        self.input_scratch[..num_samples].copy_from_slice(buffer.get_channel(0));

        // 2. 処理の実行
        // self.process() を呼び出す代わりに、フィールドを分解して借用することで
        // 借用チェッカーの制限（self全体を借用しつつその一部を引数に渡せない）を回避する
        let input = &self.input_scratch[..num_samples];
        let output = &mut self.output_scratch[..num_samples];
        let processor = &mut self.processor;
        
        self.gatherer.process_stream(input, output, |in_block, out_block| {
            processor.process_block(in_block, out_block);
        });

        // 3. 出力データへの書き戻し
        for ch in 0..num_channels {
            let dest = buffer.get_channel_mut(ch);
            dest.copy_from_slice(&self.output_scratch[..num_samples]);
        }
    }

    fn latency(&self) -> u32 {
        self.get_latency()
    }
}
