use crate::ir::{IrOp, ParamId, BufferId, StateId};
use crate::graph::{AudioGraph, NodeId, GraphNode};

use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct CompiledGraph {
    /// 実行命令列
    pub ops: Vec<IrOp>,
    /// 最大バッファ数
    pub buffer_count: u32,
    /// 最大状態数
    pub state_count: u32,
}

impl CompiledGraph {
    pub fn pretty_print(&self) {
        println!("--- CompiledGraph (Linear IR) ---");
        println!("Buffer Count: {}", self.buffer_count);
        println!("State Count:  {}", self.state_count);
        println!("Operations:");
        for (i, op) in self.ops.iter().enumerate() {
            println!("  {:3}: {}", i, op);
        }
        println!("---------------------------------");
    }
}

pub struct CompilationResult {
    pub program: CompiledGraph,
    pub node_to_ops: std::collections::HashMap<NodeId, std::ops::Range<usize>>,
}

/// グラフを IR 命令列にコンパイルする
/// execution_order はトポロジカルソート済みであること
pub fn compile_graph(graph: &AudioGraph, execution_order: &[NodeId]) -> CompilationResult {
    let mut compiled_ops = Vec::new();
    let mut node_to_ops = std::collections::HashMap::new();
    let mut state_count = 0;
    
    // ポートごとのバッファ割り当てマップ
    // (NodeId, output_port_index) -> BufferId
    let mut port_to_buffer = std::collections::HashMap::new();
    let mut buffer_counter = 0;
    
    // バッファの事前割り当て
    for (i, node) in graph.nodes.iter().enumerate() {
        let node_id = NodeId(i);
        for port_idx in 0..node.output_ports().len() {
            port_to_buffer.insert((node_id, port_idx as u32), BufferId(buffer_counter));
            buffer_counter += 1;
        }
    }

    for &node_id in execution_order {
        let start_idx = compiled_ops.len();
        let node = &graph.nodes[node_id.0];
        
        match node {
            GraphNode::Input { channel } => {
                let buf_id = port_to_buffer[&(node_id, 0)];
                compiled_ops.push(IrOp::ReadInput { channel: *channel });
                compiled_ops.push(IrOp::StoreBuffer(buf_id));
            }
            GraphNode::Output { channel } => {
                for &(from, from_p, to, _) in &graph.edges {
                    if to == node_id {
                        let src_buf = port_to_buffer[&(from, from_p)];
                        compiled_ops.push(IrOp::LoadBuffer(src_buf));
                        compiled_ops.push(IrOp::WriteOutput { channel: *channel });
                        break;
                    }
                }
            }
            GraphNode::Stdlib(stdlib_node) => {
                let in_ports = node.input_ports();
                let mut inputs = vec![BufferId(0); in_ports.len()];
                for &(from, from_p, to, to_p) in &graph.edges {
                    if to == node_id {
                        inputs[to_p as usize] = port_to_buffer[&(from, from_p)];
                    }
                }
                let out_ports = node.output_ports();
                let mut output_ids = Vec::new();
                for port_idx in 0..out_ports.len() {
                    output_ids.push(port_to_buffer[&(node_id, port_idx as u32)]);
                }
                let ops = stdlib_node.compile(&output_ids, &inputs);
                for op in ops {
                    compiled_ops.push(op.clone());
                    if let IrOp::Delay { state_id, .. } = op {
                        if state_id.0 >= state_count { state_count = state_id.0 + 1; }
                    }
                }
            }
            GraphNode::Custom { ops, .. } => {
                for op in ops {
                    compiled_ops.push(op.clone());
                    if let IrOp::Delay { state_id, .. } = op {
                        if state_id.0 >= state_count { state_count = state_id.0 + 1; }
                    }
                }
            }
            GraphNode::Script { source, .. } => {
                let in_ports = node.input_ports();
                let mut inputs = vec![BufferId(0); in_ports.len()];
                for &(from, from_p, to, to_p) in &graph.edges {
                    if to == node_id {
                        inputs[to_p as usize] = port_to_buffer[&(from, from_p)];
                    }
                }
                let out_ports = node.output_ports();
                let mut outputs = Vec::new();
                for port_idx in 0..out_ports.len() {
                    outputs.push(port_to_buffer[&(node_id, port_idx as u32)]);
                }
                if let Ok((script_ops, next_temp)) = crate::scripting::run_script(source, inputs, outputs, buffer_counter as usize) {
                    buffer_counter = next_temp as u32;
                    for op in script_ops {
                        compiled_ops.push(op.clone());
                        if let IrOp::Delay { state_id, .. } = op {
                            if state_id.0 >= state_count { state_count = state_id.0 + 1; }
                        }
                    }
                }
            }
        }
        
        let end_idx = compiled_ops.len();
        node_to_ops.insert(node_id, start_idx..end_idx);
    }

    CompilationResult {
        program: CompiledGraph {
            ops: compiled_ops,
            buffer_count: buffer_counter,
            state_count,
        },
        node_to_ops,
    }
}

