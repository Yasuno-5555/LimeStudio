use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::sync::RwLock;

pub struct InteractionStore {
    pub param_values: RwLock<HashMap<String, Arc<AtomicU32>>>,
}

impl Default for InteractionStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InteractionStore {
    pub fn new() -> Self {
        Self {
            param_values: RwLock::new(HashMap::new()),
        }
    }

    pub fn set_value(&self, id: &str, value: f32) {
        if !value.is_finite() {
            return;
        }

        let atomic = {
            let read = self.param_values.read().unwrap();
            read.get(id).cloned()
        };

        if let Some(atomic) = atomic {
            atomic.store(value.to_bits(), Ordering::Relaxed);
        } else {
            let mut write = self.param_values.write().unwrap();
            let arc = Arc::new(AtomicU32::new(value.to_bits()));
            write.insert(id.to_string(), arc);
        }
    }

    pub fn get_value(&self, id: &str) -> f32 {
        let read = self.param_values.read().unwrap();
        if let Some(val) = read.get(id) {
            f32::from_bits(val.load(Ordering::Relaxed))
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone)]
pub enum InteractionEvent {
    BeginDrag { param_id: String, seq_id: u64 },
    Drag { param_id: String, seq_id: u64 },
    EndDrag { param_id: String, seq_id: u64 },
}
