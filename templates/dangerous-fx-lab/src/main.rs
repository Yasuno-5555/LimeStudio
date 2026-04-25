use limestudio_core::graph::{AudioGraph, GraphNode, PortInfo, PortType};
use limestudio_core::ir::IrOp;
use limestudio_core::preset::Preset;

fn main() {
    println!("Dangerous FX Lab - Visible Destruction");
    
    let mut graph = AudioGraph::new();
    
    // 1. Input
    let in_n = graph.add_node(GraphNode::Input { channel: 0 });
    
    // 2. Unstable Filter (Custom IR)
    // Intentionally allowing resonance to go beyond 1.0
    let filter = graph.add_node(GraphNode::Custom {
        ops: vec![
            IrOp::ReadInput { channel: 0 },
            IrOp::LoadParam(limestudio_core::ir::ParamId(0)), // Resonance (Dangerous)
            IrOp::Mul,
            IrOp::Add, // Simplified feedback-like behavior
        ],
        inputs: vec![PortInfo { name: "in".into(), port_type: PortType::AudioMono }],
        outputs: vec![PortInfo { name: "out".into(), port_type: PortType::AudioMono }],
    });
    
    // 3. Spectral Destruction Node
    let crusher = graph.add_node(GraphNode::Custom {
        ops: vec![
            IrOp::LoadBuffer(limestudio_core::ir::BufferId(0)),
            IrOp::Sin, // Chaotic transformation
            IrOp::MulConst(10.0),
            IrOp::Clamp { min: -1.0, max: 1.0 },
        ],
        inputs: vec![PortInfo { name: "in".into(), port_type: PortType::AudioMono }],
        outputs: vec![PortInfo { name: "out".into(), port_type: PortType::AudioMono }],
    });
    
    // 4. Output
    let out_n = graph.add_node(GraphNode::Output { channel: 0 });
    
    graph.add_edge(in_n, 0, filter, 0);
    graph.add_edge(filter, 0, crusher, 0);
    graph.add_edge(crusher, 0, out_n, 0);
    
    let preset = Preset {
        metadata: limestudio_core::preset::PresetMetadata {
            name: "Chaos Factory".into(),
            description: "Visible sound destruction with unstable feedback.".into(),
            ..Default::default()
        },
        graph_snapshot: graph,
        ..Default::default()
    };
    
    println!("Dangerous Preset 'Chaos Factory' generated.");
    println!("Trust UI Focus: Highlighting unstable filter poles and NaN propagation risks.");
}
