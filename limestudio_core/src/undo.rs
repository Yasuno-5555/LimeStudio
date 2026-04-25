//! Semantic Undo System
//! 
//! Just an Undo stack is a toy. 
//! A professional undo shows exactly what will change (Graph Diff).

use crate::preset::Preset;
use crate::diff::{GraphDiff, diff_presets};
use std::collections::VecDeque;

pub struct SemanticUndo {
    history: VecDeque<Preset>,
    max_history: usize,
}

impl SemanticUndo {
    pub fn new(max_history: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    /// 履歴に追加
    pub fn push(&mut self, preset: Preset) {
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(preset);
    }

    /// Undo可能な場合、戻り先のPresetと、戻ることによる差分（Diff）を返す
    pub fn peek_undo(&self, current: &Preset) -> Option<(&Preset, GraphDiff)> {
        self.history.back().map(|prev| {
            let diff = diff_presets(current, prev);
            (prev, diff)
        })
    }

    /// Undo実行
    pub fn undo(&mut self, _current: &Preset) -> Option<Preset> {
        self.history.pop_back()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::AudioGraph;

    #[test]
    fn test_semantic_undo_diff() {
        let mut undo = SemanticUndo::new(10);
        
        let mut g1 = AudioGraph::new();
        g1.add_node(crate::graph::GraphNode::Input { channel: 0 });
        let p1 = Preset { graph_snapshot: g1, ..Default::default() };
        
        undo.push(p1.clone());
        
        let mut g2 = p1.graph_snapshot.clone();
        g2.add_node(crate::graph::GraphNode::Output { channel: 0 });
        let p2 = Preset { graph_snapshot: g2, ..Default::default() };
        
        // p2 から p1 に戻る際の差分を確認
        let (prev, diff) = undo.peek_undo(&p2).unwrap();
        assert_eq!(prev.graph_snapshot.nodes.len(), 1);
        assert_eq!(diff.nodes_removed.len(), 1); // p2にあるOutputノードが「消える」差分
    }
}
