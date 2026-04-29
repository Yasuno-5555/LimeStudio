use crate::model::stable_id::SurfaceId;
use std::collections::VecDeque;
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum CausalEvent {
    RankChange {
        link_id: SurfaceId,
        old_rank: u32,
        new_rank: u32,
    },
    ThresholdCrossed {
        link_id: SurfaceId,
        value: f32,
        threshold: f32,
    },
    ConfidenceDowngrade {
        link_id: SurfaceId,
        old_conf: f32,
        new_conf: f32,
        reason: String,
    },
    TraceLost {
        source: String,
        duration_ms: u32,
    },
}

#[derive(Debug, Clone)]
pub struct CausalSnapshot {
    pub timestamp: Instant,
    pub active_links: Vec<(SurfaceId, f32)>, // (ID, Score)
}

pub struct CausalReplayEngine {
    /// Layer 1: Responsibility Events (The Court Record)
    event_log: VecDeque<(Instant, CausalEvent)>,

    /// Layer 2: Sparse Snapshots (10Hz motivation)
    snapshots: VecDeque<CausalSnapshot>,

    max_duration_secs: f32,
    is_replaying: bool,
    scrub_pos_secs: f32, // 0.0 is live, >0.0 is past
}

impl CausalReplayEngine {
    pub fn new(duration_secs: f32) -> Self {
        Self {
            event_log: VecDeque::new(),
            snapshots: VecDeque::new(),
            max_duration_secs: duration_secs,
            is_replaying: false,
            scrub_pos_secs: 0.0,
        }
    }

    pub fn record_event(&mut self, event: CausalEvent) {
        if self.is_replaying {
            return;
        } // Read-only during replay

        self.event_log.push_back((Instant::now(), event));
        self.cleanup();
    }

    pub fn push_snapshot(&mut self, snapshot: CausalSnapshot) {
        if self.is_replaying {
            return;
        }

        self.snapshots.push_back(snapshot);
        self.cleanup();
    }

    fn cleanup(&mut self) {
        let now = Instant::now();
        let cutoff = now
            .checked_sub(std::time::Duration::from_secs_f32(self.max_duration_secs))
            .unwrap_or(now);

        while self.event_log.front().is_some_and(|(t, _)| *t < cutoff) {
            self.event_log.pop_front();
        }
        while self.snapshots.front().is_some_and(|s| s.timestamp < cutoff) {
            self.snapshots.pop_front();
        }
    }

    pub fn enter_replay(&mut self) {
        self.is_replaying = true;
    }

    pub fn exit_replay(&mut self) {
        self.is_replaying = false;
        self.scrub_pos_secs = 0.0;
    }

    pub fn scrub_to(&mut self, offset_secs: f32) {
        self.scrub_pos_secs = offset_secs.clamp(0.0, self.max_duration_secs);
    }

    pub fn get_replay_data(&self) -> Option<CausalSnapshot> {
        if !self.is_replaying {
            return None;
        }

        let target_time = Instant::now() - std::time::Duration::from_secs_f32(self.scrub_pos_secs);

        // Find the nearest snapshot and interpolate (Simple version: nearest)
        self.snapshots
            .iter()
            .min_by_key(|s| {
                if s.timestamp > target_time {
                    s.timestamp.duration_since(target_time).as_micros()
                } else {
                    target_time.duration_since(s.timestamp).as_micros()
                }
            })
            .cloned()
    }

    pub fn get_events_near(&self, window_secs: f32) -> Vec<&CausalEvent> {
        let target_time = Instant::now() - std::time::Duration::from_secs_f32(self.scrub_pos_secs);
        let half_window = std::time::Duration::from_secs_f32(window_secs * 0.5);

        self.event_log
            .iter()
            .filter(|(t, _)| {
                let diff = if *t > target_time {
                    *t - target_time
                } else {
                    target_time - *t
                };
                diff < half_window
            })
            .map(|(_, e)| e)
            .collect()
    }
}
