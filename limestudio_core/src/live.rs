//! Live Compile Loop — Zero-latency safe IR swapping.
//! 
//! グラフの変更を検知し、安全にオーディオエンジンを更新する。

use crate::graph::{AudioGraph, NodeId};
use crate::compile::{CompiledGraph, compile_graph};
use crate::validate::validate_graph;
use crate::hostile::{validate_hostile, ValidationReport};
use std::sync::{Arc, Mutex};

/// コンパイル結果とメタデータ
pub struct LiveCompilation {
    pub graph_version: u64,
    pub program: CompiledGraph,
    pub node_to_ops: std::collections::HashMap<NodeId, std::ops::Range<usize>>,
    pub hostile_report: ValidationReport,
}

/// ライブコンパイル環境
pub struct LiveCompiler {
    pub current_graph: AudioGraph,
    pub current_version: u64,
    pub last_compilation: Option<Arc<LiveCompilation>>,
}

impl LiveCompiler {
    pub fn new(graph: AudioGraph) -> Self {
        Self {
            current_graph: graph,
            current_version: 0,
            last_compilation: None,
        }
    }

    /// グラフをコンパイルし、メタデータを生成
    pub fn compile(&mut self) -> Result<Arc<LiveCompilation>, String> {
        self.current_version += 1;
        
        let order = validate_graph(&self.current_graph)
            .map_err(|e| format!("Structural Validation failed: {:?}", e))?;
        
        // compile_graph を呼び出す
        let result = compile_graph(&self.current_graph, &order);
        
        let hostile_report = validate_hostile(&self.current_graph, &result.program);
        
        let compilation = Arc::new(LiveCompilation {
            graph_version: self.current_version,
            program: result.program,
            node_to_ops: result.node_to_ops,
            hostile_report,
        });
        
        self.last_compilation = Some(compilation.clone());
        Ok(compilation)
    }

    /// 特定のノードに対応するIR命令列を取得 (Compiler Lens用)
    pub fn get_ops_for_node(&self, node_id: NodeId) -> Vec<String> {
        if let Some(comp) = &self.last_compilation {
            if let Some(range) = comp.node_to_ops.get(&node_id) {
                return comp.program.ops[range.clone()]
                    .iter()
                    .map(|op| format!("{}", op))
                    .collect();
            }
        }
        vec![]
    }
}
