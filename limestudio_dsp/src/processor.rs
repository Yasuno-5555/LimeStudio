use rustfft::{Fft, FftPlanner};
use num_complex::{Complex, Complex64};
use std::sync::Arc;
use rayon::prelude::*;
use rustfft::num_traits::Zero;
use crate::wavelets::MotherWavelet;

/// 1. Analysis Stage: 入力のFFTを担当 (Stateless-ish)
pub struct SpectralAnalyzer {
    fft: Arc<dyn Fft<f64>>,
    fft_size: usize,
    
    // Scratch & Output (Reuse allocation)
    fft_input: Vec<Complex64>,
    fft_scratch: Vec<Complex64>,
    
    // Window function
    window: Vec<f64>,
}

impl SpectralAnalyzer {
    pub fn new(fft_size: usize, _hop_size: usize, planner: &mut FftPlanner<f64>) -> Self {
        let fft = planner.plan_fft_forward(fft_size);
        
        // Hann window
        let window = (0..fft_size).map(|i| {
            0.5 * (1.0 - (2.0 * std::f64::consts::PI * i as f64 / (fft_size as f64 - 1.0)).cos())
        }).collect();

        Self {
            fft,
            fft_size,
            fft_input: vec![Complex64::zero(); fft_size],
            fft_scratch: vec![Complex64::zero(); fft_size],
            window,
        }
    }

    /// 固定長ブロックを受け取り、スペクトルを計算して返す
    pub fn compute_spectrum(&mut self, input: &[f32]) -> Vec<Complex64> {
        // Assert input length?
        let len = input.len().min(self.fft_size);
        
        // Windowing & Copy
        for (i, val) in input.iter().enumerate().take(len) {
            self.fft_input[i] = Complex::new(*val as f64 * self.window[i], 0.0);
        }
        for i in len..self.fft_size {
            self.fft_input[i] = Complex64::zero();
        }

        // FFT
        self.fft.process_with_scratch(&mut self.fft_input, &mut self.fft_scratch);

        self.fft_input.clone() // TODO: Avoid clone by returning slice or reference if possible, but lifecycle is tricky
    }
}

/// 2. Synthesis Stage: 特定のスケールの再構成 (IFFT) を担当 (No OLA, just IFFT block)
pub struct ScaleSynthesizer {
    ifft: Arc<dyn Fft<f64>>,
    fft_size: usize,
    
    // Wavelet Kernel
    pub kernel: Vec<Complex64>,
    
    // Scratch
    ifft_buffer: Vec<Complex64>,
    ifft_scratch: Vec<Complex64>,
    
    // Output Buffer (Reused)
    pub time_output: Vec<f32>,
}

impl ScaleSynthesizer {
    pub fn new(fft_size: usize, _hop_size: usize, planner: &mut FftPlanner<f64>, kernel: Vec<Complex64>) -> Self {
        let ifft = planner.plan_fft_inverse(fft_size);
        
        Self {
            ifft,
            fft_size,
            kernel,
            ifft_buffer: vec![Complex64::zero(); fft_size],
            ifft_scratch: vec![Complex64::zero(); fft_size],
            time_output: vec![0.0; fft_size],
        }
    }

    /// スペクトルを受け取り、カーネル乗算 -> IFFT して時間領域ブロックを内部バッファに出力する
    pub fn compute_block(&mut self, spectrum: &[Complex64], gain: f64) {
        // 1. Kernel Multiplication
        for (i, &bin) in spectrum.iter().enumerate() {
            self.ifft_buffer[i] = bin * self.kernel[i] * gain;
        }

        // 2. IFFT
        self.ifft.process_with_scratch(&mut self.ifft_buffer, &mut self.ifft_scratch);

        // 3. Convert to Real
        let scale = 1.0 / self.fft_size as f64; 
        
        for i in 0..self.fft_size {
            self.time_output[i] = (self.ifft_buffer[i].re * scale) as f32;
        }
    }
}

use crate::utils::Smoother;
use crate::monitor::SpectrumMonitor;

/// Main Processor (Block based)
pub struct WaveletProcessor {
    analyzer: SpectralAnalyzer,
    synthesizers: Vec<ScaleSynthesizer>,
    
    // Parameters
    _scales: Vec<f64>,
    smoothers: Vec<Smoother>,
    
    // Visualizer
    monitor_sender: Option<Box<dyn SpectrumMonitor>>,
    
    fft_size: usize,
    hop_size: usize,
    _sample_rate: f64,
}

impl WaveletProcessor {
    pub fn new<W: MotherWavelet>(
        sample_rate: f64,
        fft_size: usize,
        hop_size: usize,
        num_scales: usize,
        mother_wavelet: &W
    ) -> Self {
        let mut planner = FftPlanner::new();
        let analyzer = SpectralAnalyzer::new(fft_size, hop_size, &mut planner);
        
        // Scale Setup
        let min_freq = 20.0;
        let max_freq = sample_rate / 2.0;
        let mut scales = Vec::with_capacity(num_scales);
        
        for i in 0..num_scales {
            let p = i as f64 / (num_scales - 1) as f64;
            let freq = max_freq * (min_freq / max_freq).powf(p);
            scales.push(freq);
        }

        // Kernel Generation & Normalization
        let mut kernels: Vec<Vec<Complex64>> = Vec::with_capacity(num_scales);
        let mut freq_sum = vec![0.0; fft_size];

        for scale_freq in scales.iter().take(num_scales) {
            let freq_target = *scale_freq;
            let w0 = 6.0; 
            let scale = w0 / (2.0 * std::f64::consts::PI * freq_target / sample_rate);
            
            let mut kernel = vec![Complex64::zero(); fft_size];
            for k in 0..fft_size {
                let omega = if k <= fft_size / 2 {
                    k as f64 / fft_size as f64 * 2.0 * std::f64::consts::PI
                } else {
                    (k as f64 - fft_size as f64) / fft_size as f64 * 2.0 * std::f64::consts::PI
                };
                
                let amp = mother_wavelet.frequency_domain(omega, scale); 
                kernel[k] = Complex64::new(amp, 0.0);
                freq_sum[k] += amp; 
            }
            kernels.push(kernel);
        }

        // Apply Normalization
        for k in 0..fft_size {
            if freq_sum[k] > 1e-6 {
                let norm_factor = 1.0 / freq_sum[k];
                for kernel in kernels.iter_mut().take(num_scales) {
                    kernel[k] *= norm_factor;
                }
            }
        }

        let synthesizers = kernels.into_iter()
            .map(|k| ScaleSynthesizer::new(fft_size, hop_size, &mut planner, k))
            .collect();
            
        // Initialize smoothers with 1.0 (Unity Gain)
        // Ramp time: 20ms typically good for gain
        let smoothers = (0..num_scales)
            .map(|_| Smoother::new(1.0, (sample_rate / hop_size as f64) as f32, 20.0)) // Update rate is Block Rate!
            .collect();

        Self {
            analyzer,
            synthesizers,
            _scales: scales,
            smoothers,
            monitor_sender: None,
            fft_size,
            hop_size,
            _sample_rate: sample_rate,
        }
    }

    pub fn set_gain(&mut self, scale_idx: usize, gain: f64) {
        if scale_idx < self.smoothers.len() {
            self.smoothers[scale_idx].set_target(gain as f32);
        }
    }
    
    pub fn set_monitor(&mut self, sender: Box<dyn SpectrumMonitor>) {
        self.monitor_sender = Some(sender);
    }
    
    pub fn latency(&self) -> u32 {
        (self.fft_size - self.hop_size) as u32
    }
    
    /// 固定長の入力ブロック処理し、固定長の出力ブロックを返す (OLAなし)
    /// input: equal to fft_size
    /// output: equal to fft_size (caller must size it correctly)
    pub fn process_block(&mut self, input: &[f32], output: &mut [f32]) {
        // 1. Analysis
        let spectrum = self.analyzer.compute_spectrum(input);

        // 2. Synthesis (Parallel Compute)
        // Collect current gains from smoothers
        // Note: Smoother operates on block rate, so we call next() once per block.
        // We collect gains into a temporary vector to pass to par_iter
        
        let current_gains: Vec<f64> = self.smoothers.iter_mut()
            .map(|s| s.tick() as f64)
            .collect();

        let synthesizers = &mut self.synthesizers;
        
        // Compute IFFT in parallel, storing results in each synthesizer's `time_output`
        synthesizers.par_iter_mut()
            .zip(current_gains.par_iter())
            .for_each(|(synth, &g)| {
                synth.compute_block(&spectrum, g);
            });

        // 3. Monitor (Optional)
        if let Some(monitor) = &mut self.monitor_sender {
            monitor.send_spectrum(&spectrum);
        }

        // 4. Integration (Summation)
        output.fill(0.0);
        
        for synth in synthesizers.iter() {
            for (i, val) in synth.time_output.iter().enumerate() {
                if i < output.len() {
                    output[i] += val;
                }
            }
        }
    }
}

