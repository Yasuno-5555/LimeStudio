use crate::ir::IrOp;
use serde::{Serialize, Deserialize};

/// グラフ内でのノード識別子 (Slot index)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct NodeId(pub usize); // Slot index 直接指定

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Node({})", self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum PortType {
    AudioMono,
    AudioStereo,
    Control,
    Event,
    Midi,
    Spectrum,
}

impl std::fmt::Display for PortType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortInfo {
    pub name: String,
    pub port_type: PortType,
}

/// グラフのノード定義
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GraphNode {
    /// 外部入力 (DAWからの信号)
    Input { channel: u8 },
    /// 外部出力 (DAWへの信号)
    Output { channel: u8 },
    /// 標準ライブラリノード
    Stdlib(crate::stdlib::StdlibNode),
    /// プリミティブ命令によるカスタムノード
    Custom {
        ops: Vec<IrOp>,
        inputs: Vec<PortInfo>,
        outputs: Vec<PortInfo>,
    },
    /// スクリプトによるIR生成ノード
    Script {
        source: String,
        inputs: Vec<PortInfo>,
        outputs: Vec<PortInfo>,
    },
}

impl GraphNode {
    /// 入力ポート情報を返す
    pub fn input_ports(&self) -> Vec<PortInfo> {
        match self {
            GraphNode::Input { .. } => vec![],
            GraphNode::Output { .. } => vec![PortInfo { name: "in".into(), port_type: PortType::AudioMono }],
            GraphNode::Stdlib(node) => node.input_ports(),
            GraphNode::Custom { inputs, .. } => inputs.clone(),
            GraphNode::Script { inputs, .. } => inputs.clone(),
        }
    }

    /// 出力ポート情報を返す
    pub fn output_ports(&self) -> Vec<PortInfo> {
        match self {
            GraphNode::Input { .. } => vec![PortInfo { name: "out".into(), port_type: PortType::AudioMono }],
            GraphNode::Output { .. } => vec![],
            GraphNode::Stdlib(node) => node.output_ports(),
            GraphNode::Custom { outputs, .. } => outputs.clone(),
            GraphNode::Script { outputs, .. } => outputs.clone(),
        }
    }
}

impl std::fmt::Display for GraphNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphNode::Input { channel } => write!(f, "Input(ch={})", channel),
            GraphNode::Output { channel } => write!(f, "Output(ch={})", channel),
            GraphNode::Stdlib(node) => write!(f, "Stdlib({:?})", node),
            GraphNode::Custom { ops, .. } => write!(f, "Custom({} ops)", ops.len()),
            GraphNode::Script { source, .. } => write!(f, "Script({} chars)", source.len()),
        }
    }
}

/// オーディオグラフ (DAG)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioGraph {
    pub nodes: Vec<GraphNode>, // index = NodeId
    pub edges: Vec<(NodeId, u32, NodeId, u32)>, // from, port, to, port
}

impl AudioGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// ノードを追加し、そのIDを返す
    pub fn add_node(&mut self, node: GraphNode) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(node);
        id
    }

    /// ノード間を接続する
    pub fn add_edge(&mut self, from: NodeId, from_port: u32, to: NodeId, to_port: u32) {
        self.edges.push((from, from_port, to, to_port));
    }

    /// グラフの構造をデバッグ出力する
    pub fn debug_dump(&self) {
        println!("--- AudioGraph Debug Dump ---");
        println!("Nodes:");
        for (i, node) in self.nodes.iter().enumerate() {
            println!("  [{}] {}", i, node);
            if let GraphNode::Custom { ops, .. } = node {
                for (op_idx, op) in ops.iter().enumerate() {
                    println!("    {:3}: {}", op_idx, op);
                }
            }
            
            // ポート情報を表示
            let inputs = node.input_ports();
            let outputs = node.output_ports();
            if !inputs.is_empty() {
                print!("    In: ");
                for (idx, p) in inputs.iter().enumerate() {
                    print!("[{}:{}] ", idx, p.port_type);
                }
                println!();
            }
            if !outputs.is_empty() {
                print!("    Out: ");
                for (idx, p) in outputs.iter().enumerate() {
                    print!("[{}:{}] ", idx, p.port_type);
                }
                println!();
            }
        }
        println!("Edges:");
        for (from, from_p, to, to_p) in &self.edges {
            println!("  {} [port {}] -> {} [port {}]", from, from_p, to, to_p);
        }
        println!("-----------------------------");
    }

    /// JSON に変換する
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// JSON から読み込む
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::IrOp;

    #[test]
    fn test_serialization() {
        let mut graph = AudioGraph::new();
        let in_node = graph.add_node(GraphNode::Input { channel: 0 });
        let out_node = graph.add_node(GraphNode::Output { channel: 0 });
        let gain_node = graph.add_node(GraphNode::Custom {
            ops: vec![
                IrOp::ReadInput { channel: 0 },
                IrOp::MulConst(0.5),
                IrOp::WriteOutput { channel: 0 },
            ],
            inputs: vec![PortInfo { name: "in".into(), port_type: PortType::AudioMono }],
            outputs: vec![PortInfo { name: "out".into(), port_type: PortType::AudioMono }],
        });
        graph.add_edge(in_node, 0, gain_node, 0);
        graph.add_edge(gain_node, 0, out_node, 0);

        let json = graph.to_json().unwrap();
        let graph2 = AudioGraph::from_json(&json).unwrap();
        assert_eq!(graph.nodes.len(), graph2.nodes.len());
        assert_eq!(graph.edges.len(), graph2.edges.len());
        
        std::fs::write("test_graph.json", json).unwrap();
    }

    #[test]
    fn test_stdlib_gain_compilation() {
        use crate::stdlib::StdlibNode;
        use crate::ir::ParamRef;
        
        let mut graph = AudioGraph::new();
        let in_n = graph.add_node(GraphNode::Input { channel: 0 });
        let gain_n = graph.add_node(GraphNode::Stdlib(StdlibNode::Gain {
            amount: ParamRef::Const(0.7)
        }));
        let out_n = graph.add_node(GraphNode::Output { channel: 0 });
        
        graph.add_edge(in_n, 0, gain_n, 0);
        graph.add_edge(gain_n, 0, out_n, 0);
        
        let order = crate::validate::validate_graph(&graph).unwrap();
        let program = crate::compile::compile_graph(&graph, &order);
        
        // IRに MulConst(0.7) が含まれているか確認
        let has_mul_const = program.ops.iter().any(|op| matches!(op, IrOp::MulConst(v) if (*v - 0.7).abs() < 0.001));
        assert!(has_mul_const);
        
        std::fs::write("stdlib_graph.json", graph.to_json().unwrap()).unwrap();
    }

    #[test]
    fn test_json_roundtrip_compilation_parity() {
        use crate::stdlib::*;
        use crate::ir::ParamRef;
        use crate::validate::validate_graph;
        use crate::compile::compile_graph;

        let mut g = AudioGraph::new();
        let in_n = g.add_node(GraphNode::Input { channel: 0 });
        let osc = g.add_node(GraphNode::Stdlib(StdlibNode::Oscillator {
            freq: ParamRef::Const(440.0),
            wave: Waveform::Sine,
        }));
        let mix = g.add_node(GraphNode::Stdlib(StdlibNode::Mix {
            ratio: ParamRef::Const(0.5),
        }));
        let out_n = g.add_node(GraphNode::Output { channel: 0 });
        
        g.add_edge(in_n, 0, mix, 0);
        g.add_edge(osc, 0, mix, 1);
        g.add_edge(mix, 0, out_n, 0);
        
        // Compile original
        let order1 = validate_graph(&g).unwrap();
        let prog1 = compile_graph(&g, &order1);
        
        // Roundtrip
        let json = g.to_json().unwrap();
        let g2 = AudioGraph::from_json(&json).unwrap();
        
        // Compile restored
        let order2 = validate_graph(&g2).unwrap();
        let prog2 = compile_graph(&g2, &order2);
        
        // Verify parity
        assert_eq!(prog1.ops.len(), prog2.ops.len());
        for (op1, op2) in prog1.ops.iter().zip(prog2.ops.iter()) {
            assert_eq!(format!("{:?}", op1), format!("{:?}", op2));
        }
        assert_eq!(prog1.buffer_count, prog2.buffer_count);
        assert_eq!(prog1.state_count, prog2.state_count);
    }
}
