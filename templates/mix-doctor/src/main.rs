use limestudio_core::graph::{AudioGraph, GraphNode, PortInfo, PortType};
use limestudio_core::ir::IrOp;
use limestudio_core::preset::Preset;

fn main() {
    println!("Mix Doctor - Diagnostic Plugin Template");
    
    let mut graph = AudioGraph::new();
    
    // 1. Inputs (Stereo)
    let in_l = graph.add_node(GraphNode::Input { channel: 0 });
    let in_r = graph.add_node(GraphNode::Input { channel: 1 });
    
    // 2. DC Offset Detector (Custom Node)
    // Using a simple low-pass to extract DC component for visualization
    let dc_detect = graph.add_node(GraphNode::Custom {
        ops: vec![
            IrOp::ReadInput { channel: 0 },
            IrOp::LoadConst(0.999), // Simple leaky integrator
            IrOp::Mul,
            IrOp::StoreBuffer(limestudio_core::ir::BufferId(0)),
        ],
        inputs: vec![PortInfo { name: "in".into(), port_type: PortType::AudioMono }],
        outputs: vec![PortInfo { name: "dc_val".into(), port_type: PortType::Control }],
    });
    
    // 3. Phase Correlation Node
    // (in_l * in_r) -> Smoothing -> Correlation
    let phase_corr = graph.add_node(GraphNode::Custom {
        ops: vec![
            IrOp::ReadInput { channel: 0 },
            IrOp::ReadInput { channel: 1 },
            IrOp::Mul,
            IrOp::StoreBuffer(limestudio_core::ir::BufferId(1)),
        ],
        inputs: vec![
            PortInfo { name: "left".into(), port_type: PortType::AudioMono },
            PortInfo { name: "right".into(), port_type: PortType::AudioMono },
        ],
        outputs: vec![PortInfo { name: "corr".into(), port_type: PortType::Control }],
    });
    
    // 4. Loudness Monitor (Peak)
    let peak_monitor = graph.add_node(GraphNode::Custom {
        ops: vec![
            IrOp::ReadInput { channel: 0 },
            IrOp::Abs,
            IrOp::StoreBuffer(limestudio_core::ir::BufferId(2)),
        ],
        inputs: vec![PortInfo { name: "in".into(), port_type: PortType::AudioMono }],
        outputs: vec![PortInfo { name: "peak".into(), port_type: PortType::Control }],
    });
    
    // Connections (Diagnostics only, usually passed through for processing)
    graph.add_edge(in_l, 0, dc_detect, 0);
    graph.add_edge(in_l, 0, phase_corr, 0);
    graph.add_edge(in_r, 0, phase_corr, 1);
    graph.add_edge(in_l, 0, peak_monitor, 0);
    
    let preset = Preset {
        metadata: limestudio_core::preset::PresetMetadata {
            name: "Default Diagnostic".into(),
            description: "Monitors DC, Phase, and Peak levels with Trust UI transparency.".into(),
            ..Default::default()
        },
        graph_snapshot: graph,
        ..Default::default()
    };
    
    println!("Mix Doctor Preset generated.");
    println!("Trust UI Focus: Real-time visualization of phase correlation and DC offset issues.");
}
