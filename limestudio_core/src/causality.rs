use dirtydata_core::{Graph, StableId};
use std::collections::{HashSet, VecDeque};

#[derive(Debug, Clone)]
pub struct CausalityTrace {
    pub error_source: StableId,
    /// Nodes that contribute to the signal at error_source, sorted by proximity.
    pub contributors: Vec<StableId>,
    /// Critical path of signal flow.
    pub critical_path: Vec<StableId>,
}

pub struct CausalityEngine;

impl CausalityEngine {
    pub fn trace_back(graph: &Graph, target_id: StableId) -> CausalityTrace {
        let mut contributors = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(target_id);
        visited.insert(target_id);

        while let Some(current_id) = queue.pop_front() {
            // Find connections where current_id is the target
            for conn in graph.edges.values() {
                if conn.target.node_id == current_id {
                    let source_id = conn.source.node_id;
                    if !visited.contains(&source_id) {
                        visited.insert(source_id);
                        contributors.push(source_id);
                        queue.push_back(source_id);
                    }
                }
            }
        }

        // For simplicity, critical path is just the first contributors found (BFS order)
        let critical_path = contributors.clone();

        CausalityTrace {
            error_source: target_id,
            contributors,
            critical_path,
        }
    }
}

/// Polyphonic causality tracking.
/// Tracks which voice triggered which DSP state change.
#[derive(Debug, Clone)]
pub struct PolyphonicCausality {
    pub voice_id: u32,
    pub event_source: String, // e.g. "NoteOn(60)"
    pub impact_nodes: Vec<StableId>,
    pub timestamp: u64, // Sample-accurate
}

pub struct CausalityMonitor {
    pub poly_traces: Vec<PolyphonicCausality>,
}

impl Default for CausalityMonitor {
    fn default() -> Self {
        Self::new()
    }
}
impl CausalityMonitor {
    pub fn new() -> Self {
        Self {
            poly_traces: Vec::new(),
        }
    }

    pub fn push_event(&mut self, voice: u32, source: &str, impact: Vec<StableId>, time: u64) {
        self.poly_traces.push(PolyphonicCausality {
            voice_id: voice,
            event_source: source.to_string(),
            impact_nodes: impact,
            timestamp: time,
        });

        // Keep only recent traces
        if self.poly_traces.len() > 1024 {
            self.poly_traces.remove(0);
        }
    }
}
