use limestudio_core::graph::*;
use limestudio_core::stdlib::*;
use limestudio_core::ir::*;
use limestudio_plugin::*;

fn create_poly_graph() -> AudioGraph {
    let mut g = AudioGraph::new();
    
    // P0: Frequency, P1: Gate
    let osc = g.add_node(GraphNode::Stdlib(StdlibNode::Oscillator {
        freq: ParamRef::Param(ParamId(0)),
        wave: Waveform::Sine,
    }));
    
    let adsr = g.add_node(GraphNode::Stdlib(StdlibNode::Adsr {
        attack: ParamRef::Const(0.01),
        decay: ParamRef::Const(0.1),
        sustain: ParamRef::Const(0.8),
        release: ParamRef::Const(0.5),
        gate: ParamRef::Param(ParamId(1)),
    }));
    
    let vca = g.add_node(GraphNode::Stdlib(StdlibNode::Multiply));
    
    let out_l = g.add_node(GraphNode::Output { channel: 0 });
    let out_r = g.add_node(GraphNode::Output { channel: 1 });
    
    // Connections
    g.add_edge(osc, 0, vca, 0); // Osc -> Multiply (in1)
    g.add_edge(adsr, 0, vca, 1); // Adsr -> Multiply (in2)
    
    g.add_edge(vca, 0, out_l, 0);
    g.add_edge(vca, 0, out_r, 0);
    
    g
}

lime_plugin_poly!(
    create_poly_graph,
    PolyConfig {
        max_voices: 8,
        freq_param: Some(ParamId(0)),
        gate_param: Some(ParamId(1)),
        vel_param: None,
    }
);
