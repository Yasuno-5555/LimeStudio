use crate::engine::{VoiceEvent, VoiceManager};
use dirtydata_core::ir::Graph;
use rtrb::RingBuffer;

pub struct TortureSuite {
    manager: VoiceManager,
}

impl TortureSuite {
    pub fn new(num_voices: usize, sample_rate: f32) -> Self {
        let graph = Graph::new();
        let (_prod, cons) = RingBuffer::new(1024);
        let manager = VoiceManager::from_graph(&graph, cons, num_voices, sample_rate, None);
        Self { manager }
    }

    /// Stress test with randomized buffer sizes (FL Studio style)
    pub fn stress_variable_buffers(&mut self, total_samples: usize) {
        let mut rng = crate::math::DeterministicRng::seed_from_u64(0x1337);
        let mut processed = 0;

        while processed < total_samples {
            let block_size = (rng.next_u64() % 2047 + 1) as usize;
            let block_size = block_size.min(total_samples - processed);
            let mut left = vec![0.0; block_size];
            let mut right = vec![0.0; block_size];
            let inputs: &[&[f32]] = &[];
            let outputs: &mut [&mut [f32]] = &mut [&mut left, &mut right];

            self.manager
                .process(inputs, outputs, self.manager.sample_rate);
            processed += block_size;
        }
    }

    /// Stress test with rapid lifecycle events
    pub fn stress_lifecycle(&mut self, iterations: usize) {
        for _ in 0..iterations {
            self.manager.reset();
            self.manager.handle_event(VoiceEvent::NoteOn {
                pitch: 60,
                velocity: 0.8,
            });
            let mut left = vec![0.0; 64];
            let mut right = vec![0.0; 64];
            let outputs: &mut [&mut [f32]] = &mut [&mut left, &mut right];
            self.manager.process(&[], outputs, self.manager.sample_rate);
        }
    }

    /// Stress test with sample rate changes
    pub fn stress_sample_rate(&mut self, iterations: usize) {
        let mut rng = crate::math::DeterministicRng::seed_from_u64(0x42);
        let rates = [44100.0, 48000.0, 88200.0, 96000.0];

        for _ in 0..iterations {
            let sr = rates[(rng.next_u64() as usize) % rates.len()];
            let mut left = vec![0.0; 128];
            let mut right = vec![0.0; 128];
            let outputs: &mut [&mut [f32]] = &mut [&mut left, &mut right];
            self.manager.process(&[], outputs, sr);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_torture_suite() {
        let mut suite = TortureSuite::new(16, 44100.0);
        println!("Running Variable Buffer Torture...");
        suite.stress_variable_buffers(100000);
        println!("Running Lifecycle Torture...");
        suite.stress_lifecycle(1000);
        println!("Running Sample Rate Torture...");
        suite.stress_sample_rate(100);
        println!("Torture Suite Passed!");
    }
}
