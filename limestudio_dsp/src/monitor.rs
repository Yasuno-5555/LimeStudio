use ringbuf::{Producer, RingBuffer};
use num_complex::Complex64;

pub struct SpectrumMonitorSender {
    producer: Producer<Vec<f32>>,
    buffer_scratch: Vec<f32>,
    fft_size: usize,
}

impl SpectrumMonitorSender {
    pub fn new(producer: Producer<Vec<f32>>, fft_size: usize) -> Self {
        Self {
            producer,
            buffer_scratch: Vec::with_capacity(fft_size),
            fft_size,
        }
    }

    pub fn send_spectrum(&mut self, complex_spectrum: &[Complex64]) {
        // GUIは60fpsあれば十分なので、バッファがいっぱいのときは送らない (Drop frame)
        if self.producer.is_full() {
            return;
        }

        // Magnitude計算
        self.buffer_scratch.clear();
        
        // 効率化: ミラー成分を捨てて半分だけ送る等の最適化も可能だが、
        // ここでは全ビンのMagnitudeを送る
        for c in complex_spectrum.iter().take(self.fft_size) {
            self.buffer_scratch.push(c.norm() as f32);
        }

        // 送信 (Try push)
        // Note: ringbuf::Producer::push takes an element.
        // We are pushing the whole vector. 
        // Allocating a new vector here (clone inside push?) allows the consumer to own data.
        // buffer_scratch is reused for calculation, but we need to send a copy.
        // Producer<Vec<f32>> means we transfer ownership of a Vec.
        let _ = self.producer.push(self.buffer_scratch.clone());
    }
}
