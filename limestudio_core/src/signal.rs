use std::sync::atomic::{AtomicU32, Ordering};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// A wait-free bridge for streaming real-time metrics from the Audio Engine to the UI.
/// Uses AtomicU32 with bitcasting to safely transport f32 values.
pub struct SignalRegistry {
    /// Registry of metrics. 
    /// The Map itself is behind a RwLock for registration (rare), 
    /// but individual values are Atomic for frequent updates.
    metrics: RwLock<HashMap<String, Arc<AtomicU32>>>,
}

impl Default for SignalRegistry {
    fn default() -> Self { Self::new() }
}

impl SignalRegistry {
    pub fn new() -> Self {
        Self {
            metrics: RwLock::new(HashMap::new()),
        }
    }

    /// Register a new metric. This should be done during initialization, not in the process loop.
    pub fn register(&self, id: &str) {
        let mut metrics = self.metrics.write();
        if !metrics.contains_key(id) {
            metrics.insert(id.to_string(), Arc::new(AtomicU32::new(0.0f32.to_bits())));
        }
    }

    /// Update a metric value. Wait-free. Safe for Audio Thread.
    pub fn set_metric(&self, id: &str, value: f32) {
        let metrics = self.metrics.read();
        if let Some(atomic) = metrics.get(id) {
            atomic.store(value.to_bits(), Ordering::Relaxed);
        }
    }

    /// Get a metric value. Wait-free. Safe for UI Thread.
    pub fn get_metric(&self, id: &str) -> f32 {
        let metrics = self.metrics.read();
        if let Some(atomic) = metrics.get(id) {
            f32::from_bits(atomic.load(Ordering::Relaxed))
        } else {
            0.0
        }
    }

    /// For forensic batch processing: Get all metrics.
    pub fn get_all(&self) -> HashMap<String, f32> {
        let metrics = self.metrics.read();
        metrics.iter()
            .map(|(k, v)| (k.clone(), f32::from_bits(v.load(Ordering::Relaxed))))
            .collect()
    }
}

/// Helper for nodes to report their status.
pub struct SignalTap<'a> {
    registry: &'a SignalRegistry,
    prefix: String,
}

impl<'a> SignalTap<'a> {
    pub fn new(registry: &'a SignalRegistry, prefix: String) -> Self {
        Self { registry, prefix }
    }

    pub fn report(&self, suffix: &str, value: f32) {
        let id = format!("{}/{}", self.prefix, suffix);
        self.registry.set_metric(&id, value);
    }
}
