use crate::graph::{AudioGraph, NodeId};
use std::collections::VecDeque;

#[derive(Debug, PartialEq)]
pub enum GraphValidationError {
    /// サイクルが検出された
    CycleDetected,
    /// 出力ノードがない
    NoOutputNode,
    /// 入力ノードがない
    NoInputNode,
    /// 存在しないノードへの参照
    InvalidNodeReference(NodeId),
    /// 存在しないポートへの参照
    InvalidPort { node: NodeId, port: u32, is_input: bool },
    /// ポートの型が一致しない
    TypeMismatch { 
        from_node: NodeId, from_port: u32, from_type: crate::graph::PortType,
        to_node: NodeId, to_port: u32, to_type: crate::graph::PortType,
    },
    /// 1つの入力ポートに複数のエッジが接続されている (Explicit Mix 違反)
    MultipleInputConnected { node: NodeId, port: u32 },
}

/// グラフの整合性を検証し、トポロジカルソートされた実行順序を返す
pub fn validate_graph(graph: &AudioGraph) -> Result<Vec<NodeId>, GraphValidationError> {
    if graph.nodes.is_empty() {
        return Ok(Vec::new());
    }

    let num_nodes = graph.nodes.len();

    // 1. 入出力の存在チェック
    let mut has_input = false;
    let mut has_output = false;
    for node in &graph.nodes {
        match node {
            crate::graph::GraphNode::Input { .. } => has_input = true,
            crate::graph::GraphNode::Output { .. } => has_output = true,
            _ => {}
        }
    }
    if !has_input { return Err(GraphValidationError::NoInputNode); }
    if !has_output { return Err(GraphValidationError::NoOutputNode); }

    // 2. ポートの正当性・型チェック・多重入力チェック
    let mut connected_inputs = std::collections::HashSet::new();

    for &(from, from_port, to, to_port) in &graph.edges {
        if from.0 >= num_nodes { return Err(GraphValidationError::InvalidNodeReference(from)); }
        if to.0 >= num_nodes { return Err(GraphValidationError::InvalidNodeReference(to)); }

        let from_node = &graph.nodes[from.0];
        let to_node = &graph.nodes[to.0];

        let out_ports = from_node.output_ports();
        let in_ports = to_node.input_ports();

        let out_info = out_ports.get(from_port as usize).ok_or(GraphValidationError::InvalidPort {
            node: from, port: from_port, is_input: false,
        })?;
        let in_info = in_ports.get(to_port as usize).ok_or(GraphValidationError::InvalidPort {
            node: to, port: to_port, is_input: true,
        })?;

        // 型チェック
        if out_info.port_type != in_info.port_type {
            return Err(GraphValidationError::TypeMismatch {
                from_node: from, from_port, from_type: out_info.port_type,
                to_node: to, to_port, to_type: in_info.port_type,
            });
        }

        // 多重入力チェック (Explicit Mix)
        if !connected_inputs.insert((to, to_port)) {
            return Err(GraphValidationError::MultipleInputConnected { node: to, port: to_port });
        }
    }

    // 3. トポロジカルソート (Kahn's algorithm)
    let mut in_degree = vec![0; num_nodes];
    let mut adj = vec![Vec::new(); num_nodes];

    for &(from, _, to, _) in &graph.edges {
        adj[from.0].push(to.0);
        in_degree[to.0] += 1;
    }

    let mut queue = VecDeque::new();
    for i in 0..num_nodes {
        if in_degree[i] == 0 {
            queue.push_back(i);
        }
    }

    let mut sorted_order = Vec::new();
    while let Some(u) = queue.pop_front() {
        sorted_order.push(NodeId(u));
        for &v in &adj[u] {
            in_degree[v] -= 1;
            if in_degree[v] == 0 {
                queue.push_back(v);
            }
        }
    }

    if sorted_order.len() != num_nodes {
        return Err(GraphValidationError::CycleDetected);
    }

    Ok(sorted_order)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{AudioGraph, GraphNode, PortType, PortInfo};

    #[test]
    fn test_valid_graph() {
        let mut graph = AudioGraph::new();
        let in_n = graph.add_node(GraphNode::Input { channel: 0 });
        let out_n = graph.add_node(GraphNode::Output { channel: 0 });
        graph.add_edge(in_n, 0, out_n, 0);

        let result = validate_graph(&graph);
        assert!(result.is_ok());
        let order = result.unwrap();
        assert_eq!(order.len(), 2);
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = AudioGraph::new();
        let in_n = graph.add_node(GraphNode::Input { channel: 0 });
        let out_n = graph.add_node(GraphNode::Output { channel: 0 });
        let mid = graph.add_node(GraphNode::Custom { 
            ops: vec![],
            inputs: vec![
                PortInfo { name: "in".into(), port_type: PortType::AudioMono },
                PortInfo { name: "feedback".into(), port_type: PortType::AudioMono }
            ],
            outputs: vec![PortInfo { name: "out".into(), port_type: PortType::AudioMono }],
        });
        
        graph.add_edge(in_n, 0, mid, 0);
        graph.add_edge(mid, 0, mid, 1); // Cycle through port 1
        graph.add_edge(mid, 0, out_n, 0);

        let result = validate_graph(&graph);
        assert_eq!(result, Err(GraphValidationError::CycleDetected));
    }

    #[test]
    fn test_type_mismatch() {
        let mut graph = AudioGraph::new();
        let in_n = graph.add_node(GraphNode::Input { channel: 0 });
        let _out_n = graph.add_node(GraphNode::Output { channel: 0 });
        let custom = graph.add_node(GraphNode::Custom {
            ops: vec![],
            inputs: vec![PortInfo { name: "midi".into(), port_type: PortType::Midi }],
            outputs: vec![],
        });
        
        graph.add_edge(in_n, 0, custom, 0); // AudioMono -> Midi!
        
        let result = validate_graph(&graph);
        assert!(matches!(result, Err(GraphValidationError::TypeMismatch { .. })));
    }

    #[test]
    fn test_multiple_input_connected() {
        let mut graph = AudioGraph::new();
        let in1 = graph.add_node(GraphNode::Input { channel: 0 });
        let in2 = graph.add_node(GraphNode::Input { channel: 0 });
        let out = graph.add_node(GraphNode::Output { channel: 0 });
        
        graph.add_edge(in1, 0, out, 0);
        graph.add_edge(in2, 0, out, 0); // Multiple inputs to Port 0!
        
        let result = validate_graph(&graph);
        assert_eq!(result, Err(GraphValidationError::MultipleInputConnected { node: out, port: 0 }));
    }
}
