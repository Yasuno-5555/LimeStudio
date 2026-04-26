use serde::{Serialize, Deserialize};
use crate::graph::{AudioGraph, NodeId};
use crate::compile::CompilationResult;
use crate::ir::IrOp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidenceState {
    /// solid square
    Safe,
    /// chamfered corner
    Warning,
    /// broken edge
    Dangerous,
    /// hollow frame
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceVector {
    /// RT Safety: Real-time safety (allocation, blocking, denormals)
    pub rt_safety: String,
    /// Optimization: Vectorization, loop unrolling, constant folding
    pub optimization: String,
    /// Determinism: Bit-perfect reproducibility
    pub determinism: String,
    /// Modulation Stability: Parameter smoothing, zipper noise
    pub modulation_stability: String,
    /// Latency Risk: Feedback paths, lookahead
    pub latency_risk: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceInfo {
    pub state: ConfidenceState,
    pub score: u8, // 0-100
    pub vector: ConfidenceVector,
    pub details: Vec<String>,
}

pub type ConfidenceMap = std::collections::HashMap<NodeId, ConfidenceInfo>;

/// Calculate confidence for each node in the graph
pub fn calculate_confidence(graph: &AudioGraph, result: &CompilationResult) -> ConfidenceMap {
    let mut map = ConfidenceMap::new();

    for (node_id, ops_range) in &result.node_to_ops {
        let node_ops = &result.program.ops[ops_range.clone()];
        
        let mut details = Vec::new();
        let mut rt_score = 100u8;
        let mut opt_score = 80u8; // Base optimization
        let mut det_score = 100u8;
        let mut mod_score = 100u8;
        let mut lat_score = 100u8;

        // 1. RT Safety Check
        for op in node_ops {
            match op {
                IrOp::LoadConst(v) if v.abs() != 0.0 && v.abs() < 1e-30 => {
                    details.push("Denormal constant detected".into());
                    rt_score = rt_score.saturating_sub(20);
                }
                IrOp::Div => {
                    details.push("Unchecked division (Div) present".into());
                    rt_score = rt_score.saturating_sub(10);
                }
                IrOp::Sqrt => {
                    details.push("Sqrt without safety clamp detected".into());
                    rt_score = rt_score.saturating_sub(5);
                }
                _ => {}
            }
        }

        // 2. Optimization Check
        if node_ops.len() > 10 {
            opt_score = opt_score.saturating_add(10); // Complexity bonus?
        }
        // In a real implementation, we'd check for SIMD usage etc.

        // 3. Modulation Stability
        for (i, op) in node_ops.iter().enumerate() {
            if let IrOp::LoadParam(_) = op {
                if i + 1 < node_ops.len() {
                    match &node_ops[i + 1] {
                        IrOp::Mul | IrOp::Add => {
                            details.push("Direct parameter modulation without smoothing".into());
                            mod_score = mod_score.saturating_sub(30);
                        }
                        _ => {}
                    }
                }
            }
        }

        // 4. Latency Risk
        for op in node_ops {
            if let IrOp::Delay { samples, .. } = op {
                if *samples > 0 {
                    details.push(format!("Intrinsic latency: {} samples", samples));
                    lat_score = lat_score.saturating_sub(10);
                }
            }
        }

        let total_score = ((rt_score as u32 + opt_score as u32 + det_score as u32 + mod_score as u32 + lat_score as u32) / 5) as u8;

        let state = if total_score > 90 {
            ConfidenceState::Safe
        } else if total_score > 70 {
            ConfidenceState::Warning
        } else if total_score > 40 {
            ConfidenceState::Dangerous
        } else {
            ConfidenceState::Unknown
        };

        map.insert(*node_id, ConfidenceInfo {
            state,
            score: total_score,
            vector: ConfidenceVector {
                rt_safety: score_to_grade(rt_score),
                optimization: score_to_grade(opt_score),
                determinism: score_to_grade(det_score),
                modulation_stability: score_to_grade(mod_score),
                latency_risk: score_to_grade(lat_score),
            },
            details,
        });
    }

    map
}

fn score_to_grade(score: u8) -> String {
    if score >= 95 { "S".into() }
    else if score >= 85 { "A".into() }
    else if score >= 75 { "B".into() }
    else if score >= 60 { "C".into() }
    else { "D".into() }
}
