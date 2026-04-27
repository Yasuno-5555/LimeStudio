//! Causality Tracker for Forensic DSP.
//! Tracks the relationship between inputs (MIDI/UI) and DSP state changes across voices.

use std::collections::HashMap;
use crate::math::df32;

pub struct CausalityTracker {
    pub voices: HashMap<u32, VoiceCausality>,
}

pub struct VoiceCausality {
    pub active_links: Vec<CausalLink>,
    pub velocity: f32,
    pub last_activity: std::time::Instant,
}

pub struct CausalLink {
    pub source: String,
    pub target: String,
    pub intensity: f32,
}

impl CausalityTracker {
    pub fn new() -> Self {
        Self { voices: HashMap::new() }
    }

    pub fn record_event(&mut self, voice_id: u32, source: &str, target: &str, intensity: f32) {
        let voice = self.voices.entry(voice_id).or_insert(VoiceCausality {
            active_links: Vec::new(),
            velocity: 0.0,
            last_activity: std::time::Instant::now(),
        });

        voice.active_links.push(CausalLink {
            source: source.to_string(),
            target: target.to_string(),
            intensity,
        });
        voice.last_activity = std::time::Instant::now();
    }

    pub fn prune_stale(&mut self) {
        let now = std::time::Instant::now();
        self.voices.retain(|_, v| now.duration_since(v.last_activity).as_secs_f32() < 1.0);
    }
}
