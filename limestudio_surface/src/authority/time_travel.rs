use dirtydata_core::types::StableId;
use dirtydata_runtime::nodes::base::NodeState;
use std::collections::{HashMap, VecDeque};

/// §SSS: Time Travel Engine — Forensic History.
/// "Audio Time Travel Debugger. 人類はこれを待ってたのに誰も作らなかった。怠惰なので。"
pub struct ExecutionSnapshot {
    pub timestamp: u64,
    pub node_states: HashMap<StableId, NodeState>,
}

pub struct TimeTravelEngine {
    /// History of snapshots.
    pub history: VecDeque<ExecutionSnapshot>,
    /// Maximum number of snapshots to keep.
    pub max_history: usize,
    /// Current view index in history.
    pub current_index: usize,
}

impl TimeTravelEngine {
    pub fn new(max_history: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_history),
            max_history,
            current_index: 0,
        }
    }

    pub fn push_snapshot(&mut self, state: ExecutionSnapshot) {
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(state);
        self.current_index = self.history.len().saturating_sub(1);
    }

    pub fn get_current_state(&self) -> Option<&ExecutionSnapshot> {
        self.history.get(self.current_index)
    }

    pub fn scrub(&mut self, index: usize) {
        if index < self.history.len() {
            self.current_index = index;
        }
    }

    pub fn seek_normalized(&mut self, progress: f32) {
        if self.history.is_empty() {
            return;
        }
        let idx = (progress * (self.history.len() - 1) as f32).round() as usize;
        self.current_index = idx;
    }
}
