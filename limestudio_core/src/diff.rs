//! Graph Diff System (Tier Ω)
//! 
//! Preset A -> Preset B の差分を可視化。
//! version review, collaboration, debugging に必須。

use serde::{Serialize, Deserialize};
use crate::graph::{NodeId, AudioGraph, GraphNode};
use crate::preset::Preset;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDiff {
    pub nodes_added: Vec<(NodeId, GraphNode)>,
    pub nodes_removed: Vec<(NodeId, GraphNode)>,
    pub nodes_modified: Vec<NodeModification>,
    pub edges_added: Vec<(NodeId, u32, NodeId, u32)>,
    pub edges_removed: Vec<(NodeId, u32, NodeId, u32)>,
    pub parameters_changed: Vec<ParamChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeModification {
    pub node_id: NodeId,
    pub old_node: GraphNode,
    pub new_node: GraphNode,
    pub field_changes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamChange {
    pub param_id: String,
    pub old_value: f32,
    pub new_value: f32,
}

impl GraphDiff {
    pub fn is_empty(&self) -> bool {
        self.nodes_added.is_empty() &&
        self.nodes_removed.is_empty() &&
        self.nodes_modified.is_empty() &&
        self.edges_added.is_empty() &&
        self.edges_removed.is_empty() &&
        self.parameters_changed.is_empty()
    }

    pub fn print_summary(&self) {
        println!("═══ Graph Diff Summary ═══");
        if self.is_empty() {
            println!("  No changes detected.");
            return;
        }

        if !self.nodes_added.is_empty() {
            println!("  Nodes Added: {}", self.nodes_added.len());
            for (id, node) in &self.nodes_added {
                println!("    [+] {:?}: {}", id, node);
            }
        }
        if !self.nodes_removed.is_empty() {
            println!("  Nodes Removed: {}", self.nodes_removed.len());
            for (id, node) in &self.nodes_removed {
                println!("    [-] {:?}: {}", id, node);
            }
        }
        if !self.nodes_modified.is_empty() {
            println!("  Nodes Modified: {}", self.nodes_modified.len());
            for modif in &self.nodes_modified {
                println!("    [*] {:?}: {:?}", modif.node_id, modif.field_changes);
            }
        }
        if !self.edges_added.is_empty() {
            println!("  Edges Added: {}", self.edges_added.len());
        }
        if !self.edges_removed.is_empty() {
            println!("  Edges Removed: {}", self.edges_removed.len());
        }
        if !self.parameters_changed.is_empty() {
            println!("  Parameters Changed: {}", self.parameters_changed.len());
            for pc in &self.parameters_changed {
                println!("    [~] {}: {:.3} -> {:.3}", pc.param_id, pc.old_value, pc.new_value);
            }
        }
        println!("══════════════════════════");
    }
}

pub fn diff_presets(old: &Preset, new: &Preset) -> GraphDiff {
    let mut diff = diff_graphs(&old.graph_snapshot, &new.graph_snapshot);

    // Parameter changes
    for (id, &new_val) in &new.parameters.values {
        let old_val = old.parameters.values.get(id).cloned().unwrap_or(0.0);
        if (old_val - new_val).abs() > 1e-6 {
            diff.parameters_changed.push(ParamChange {
                param_id: id.clone(),
                old_value: old_val,
                new_value: new_val,
            });
        }
    }
    
    // Check for removed params
    for (id, &old_val) in &old.parameters.values {
        if !new.parameters.values.contains_key(id) {
            diff.parameters_changed.push(ParamChange {
                param_id: id.clone(),
                old_value: old_val,
                new_value: 0.0,
            });
        }
    }

    diff
}

pub fn diff_graphs(old: &AudioGraph, new: &AudioGraph) -> GraphDiff {
    let mut nodes_added = Vec::new();
    let mut nodes_removed = Vec::new();
    let mut nodes_modified = Vec::new();

    let max_nodes = old.nodes.len().max(new.nodes.len());
    for i in 0..max_nodes {
        let node_id = NodeId(i);
        let old_node = old.nodes.get(i);
        let new_node = new.nodes.get(i);

        match (old_node, new_node) {
            (None, Some(n)) => nodes_added.push((node_id, n.clone())),
            (Some(n), None) => nodes_removed.push((node_id, n.clone())),
            (Some(o), Some(n)) => {
                let o_str = format!("{:?}", o);
                let n_str = format!("{:?}", n);
                if o_str != n_str {
                    nodes_modified.push(NodeModification {
                        node_id,
                        old_node: o.clone(),
                        new_node: n.clone(),
                        field_changes: vec!["properties".into()],
                    });
                }
            }
            (None, None) => unreachable!(),
        }
    }

    let old_edges: HashSet<_> = old.edges.iter().cloned().collect();
    let new_edges: HashSet<_> = new.edges.iter().cloned().collect();

    let edges_added: Vec<_> = new_edges.difference(&old_edges).cloned().collect();
    let edges_removed: Vec<_> = old_edges.difference(&new_edges).cloned().collect();

    GraphDiff {
        nodes_added,
        nodes_removed,
        nodes_modified,
        edges_added,
        edges_removed,
        parameters_changed: Vec::new(),
    }
}
