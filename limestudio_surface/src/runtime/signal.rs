use std::sync::atomic::{AtomicU32, Ordering};

/// A single-value atomic signal (e.g., peak level, control value).
/// "Everything is bits. Interpretation is the observer's burden."
pub struct AtomicSignal {
    pub value: AtomicU32,
}

impl AtomicSignal {
    pub fn new(initial: f32) -> Self {
        Self {
            value: AtomicU32::new(initial.to_bits()),
        }
    }

    pub fn set(&self, val: f32) {
        self.value.store(val.to_bits(), Ordering::Relaxed);
    }

    pub fn get(&self) -> f32 {
        f32::from_bits(self.value.load(Ordering::Relaxed))
    }
}

/// A buffer of samples for waveform or spectrum display.
pub struct SignalBuffer {
    pub data: Vec<AtomicU32>,
    pub write_ptr: std::sync::atomic::AtomicUsize,
}

impl SignalBuffer {
    pub fn new(size: usize) -> Self {
        let mut data = Vec::with_capacity(size);
        for _ in 0..size {
            data.push(AtomicU32::new(0.0f32.to_bits()));
        }
        Self {
            data,
            write_ptr: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    pub fn push(&self, sample: f32) {
        let ptr = self.write_ptr.fetch_add(1, Ordering::Relaxed) % self.data.len();
        self.data[ptr].store(sample.to_bits(), Ordering::Relaxed);
    }

    pub fn read_all(&self) -> Vec<f32> {
        self.data
            .iter()
            .map(|s| f32::from_bits(s.load(Ordering::Relaxed)))
            .collect()
    }
}
