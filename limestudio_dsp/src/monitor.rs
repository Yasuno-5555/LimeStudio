use std::sync::Arc;
use ringbuf::{Producer, HeapRb};
use num_complex::Complex64;

pub trait SpectrumMonitor: Send + Sync {
    fn send_spectrum(&mut self, complex_spectrum: &[Complex64]);
}

pub struct SpectrumMonitorSender {
    producer: Producer<Vec<f32>, Arc<HeapRb<Vec<f32>>>>,
    buffer_scratch: Vec<f32>,
    fft_size: usize,
}

impl SpectrumMonitorSender {
    pub fn new(producer: Producer<Vec<f32>, Arc<HeapRb<Vec<f32>>>>, fft_size: usize) -> Self {
        Self {
            producer,
            buffer_scratch: Vec::with_capacity(fft_size),
            fft_size,
        }
    }
}

// Manually implement Sync since Producer is Send but not Sync, 
// and we only use it from the audio thread.
unsafe impl Sync for SpectrumMonitorSender {}

impl SpectrumMonitor for SpectrumMonitorSender {
    fn send_spectrum(&mut self, complex_spectrum: &[Complex64]) {
        if self.producer.is_full() {
            return;
        }

        self.buffer_scratch.clear();
        for c in complex_spectrum.iter().take(self.fft_size) {
            self.buffer_scratch.push(c.norm() as f32);
        }

        let _ = self.producer.push(self.buffer_scratch.clone());
    }
}
