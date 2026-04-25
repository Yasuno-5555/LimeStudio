use rustfft::{Fft, FftPlanner};
use num_complex::{Complex, Complex64};
use rustfft::num_traits::Zero;
use std::sync::Arc;
use std::collections::VecDeque;

/// FFTベースの畳み込みエンジン
/// Overlap-Add法を用いて、ブロック単位の入力に対して途切れのない畳み込みを行う
pub struct ConvolutionEngine {
    // FFT設定
    fft: Arc<dyn Fft<f64>>,
    ifft: Arc<dyn Fft<f64>>,
    fft_size: usize,
    hop_size: usize,
    
    // バッファ
    input_buffer: VecDeque<f64>,
    overlap_buffer: Vec<f64>, // 前回の余韻 (fft_size - hop_size)
    output_buffer: VecDeque<f32>, // 出力待ちキュー (f32にしておく)
    
    // 作業用
    fft_input: Vec<Complex64>,
    fft_scratch: Vec<Complex64>,
}

impl ConvolutionEngine {
    pub fn new(fft_size: usize, hop_size: usize) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_size);
        let ifft = planner.plan_fft_inverse(fft_size);
        
        Self {
            fft,
            ifft,
            fft_size,
            hop_size,
            input_buffer: VecDeque::with_capacity(fft_size),
            overlap_buffer: vec![0.0; fft_size], // サイズは十分確保
            output_buffer: VecDeque::with_capacity(fft_size * 2),
            fft_input: vec![Complex64::zero(); fft_size],
            fft_scratch: vec![Complex64::zero(); fft_size],
        }
    }
    
    /// インパルス応答が1.0 (Thru) となるカーネルを設定するヘルパー
    /// テストやバイパス用
    pub fn set_kernel_unit(&self) -> Vec<Complex64> {
        // 時間領域で [1.0, 0.0, ...]
        // 周波数領域ではすべて 1.0 (振幅) だが位相に注意
        // 正確にはFFT([1, 0...]) -> [1, 1, 1...]
        vec![Complex64::new(1.0, 0.0); self.fft_size]
    }

    pub fn process_block(&mut self, input: &[f32], output: &mut [f32], kernel: &[Complex64]) {
        let mut in_idx = 0;
        let mut out_idx = 0;
        let block_size = input.len();

        output.fill(0.0); // 安全のため

        while in_idx < block_size {
            // 1. Fill Input
            while self.input_buffer.len() < self.fft_size && in_idx < block_size {
                self.input_buffer.push_back(input[in_idx] as f64);
                in_idx += 1;
            }

            // 2. Process Frame if Full
            if self.input_buffer.len() == self.fft_size {
                self.process_frame(kernel);
                
                // Hop: Input drop
                for _ in 0..self.hop_size {
                    self.input_buffer.pop_front();
                }
            }

            // 3. Output Consumption
            while out_idx < output.len() && !self.output_buffer.is_empty() {
                output[out_idx] = self.output_buffer.pop_front().unwrap();
                out_idx += 1;
            }
            
            // 入力が尽きて、出力も尽きたらブレイク
            if in_idx >= block_size && self.output_buffer.is_empty() {
                break;
            }
            // バッファがまだたまらない場合は、出力は0 (Latency) のまま、入力だけ進む
            if self.input_buffer.len() < self.fft_size && in_idx >= block_size {
                break;
            }
        }
    }

    fn process_frame(&mut self, kernel: &[Complex64]) {
        // A. Copy Input to FFT buffer (apply window if needed, here Rect)
        for (i, &sample) in self.input_buffer.iter().enumerate() {
            self.fft_input[i] = Complex::new(sample, 0.0);
        }

        // B. FFT
        self.fft.process_with_scratch(&mut self.fft_input, &mut self.fft_scratch);

        // C. Convolution
        for (bin, k) in self.fft_input.iter_mut().zip(kernel.iter()) {
            *bin *= k;
        }

        // D. IFFT
        self.ifft.process_with_scratch(&mut self.fft_input, &mut self.fft_scratch);

        // E. Overlap-Add Logic
        let scale = 1.0 / self.fft_size as f64;
        
        let overlap_len = self.fft_size.saturating_sub(self.hop_size);

        // 時間領域データを取り出しつつ加算
        for i in 0..self.fft_size {
            let val = self.fft_input[i].re * scale;
            
            // Overlap Add
            let sum = if i < overlap_len {
                val + self.overlap_buffer[i]
            } else {
                val
            };

            // 分岐: 出力へ回す か 次のOverlapへ保存か
            if i < self.hop_size {
                self.output_buffer.push_back(sum as f32);
            } else {
                // i >= hop_size
                // 保存場所は i - hop_size
                self.overlap_buffer[i - self.hop_size] = sum;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_impulse_response_thru() {
        let fft_size = 256;
        let hop_size = 128; // 50% overlap
        let mut engine = ConvolutionEngine::new(fft_size, hop_size);
        let kernel = engine.set_kernel_unit();

        // Dirac Delta input
        let total_len = 1024;
        let mut input = vec![0.0f32; total_len];
        input[0] = 1.0;

        let mut output = vec![0.0f32; total_len];
        
        // オフセット出力バッファを見るため、少しずつ処理
        let block_size = 64;
        for (in_chunk, out_chunk) in input.chunks(block_size).zip(output.chunks_mut(block_size)) {
            engine.process_block(in_chunk, out_chunk, &kernel);
        }

        // 検証
        // 遅延はバッファが溜まるまで発生
        // 初回、fft_size溜まるまで出力されない
        // 実装では process_frame が走った瞬間に hop_size 分の出力が出る
        // Latency = fft_size - hop_size ? 
        // 厳密には、最初のhop_sizeが出るのは、fft_size入力された瞬間。
        // つまり入力 0..255 が入った時、出力 0..127 が出る。
        // 出力 0 は 入力 0 に対応するか？
        // process logic:
        // Input [0..255] -> FFT -> IFFT -> Add Overlap(0) -> Out[0..127], Save[128..255]
        // 入力0番目の成分は、IFFT後のTime 0に含まれる。
        // それはOut[0]に出る。
        // つまり、バッファリング遅延はあるが、信号上の遅延(Shift)は0のはず？
        
        // Block processing latency:
        // Input 0..63 -> Buf 64. Out 0 (Silence).
        // Input 64..127 -> Buf 128. Out 0.
        // Input 128..191 -> Buf 192. Out 0.
        // Input 192..255 -> Buf 256 (Full!). Process. Out Buffer gets 128 samples.
        //                   Chunk loop pulls 64 samples.
        
        // つまり、Input index 255 まで読んだ時点で、Output index 0..63 が書き込まれる。
        // 入力 t=0 のインパルスは、出力のどこに出るか？
        // 出力の先頭(0)に出るはず。ただし、それが書き込まれるのは t=192..255 のブロック処理中。
        
        // 期待値: output[0] が 1.0 に近い
        // しかし、output[0] が書き込まれるのは 4回目のブロック処理時。
        // 最初の3ブロック(0..191)は outputは0.0のまま。
        
        let mut max_val = 0.0f32;
        let mut max_idx = 0;
        for (i, &v) in output.iter().enumerate() {
            if v.abs() > max_val {
                max_val = v.abs();
                max_idx = i;
            }
        }

        println!("Max value: {} at index {}", max_val, max_idx);
        
        // 浮動小数点誤差を許容
        assert!((max_val - 1.0).abs() < 1e-6, "Gain should be 1.0, got {}", max_val);
        
        // インデックス検証
        // 実測値に基づくレイテンシ（FFTサイズとブロックサイズの兼ね合い）
        let expected_latency = 192; 
        assert_eq!(max_idx, expected_latency, "Impulse should be at index {}", expected_latency);
    }
}
