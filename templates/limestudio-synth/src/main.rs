use limestudio_core::graph::{AudioGraph, GraphNode};
use limestudio_core::preset::Preset;

fn main() {
    println!("LimeStudio Synth Template");
    
    // Create a basic polyphonic sine synth
    let mut graph = AudioGraph::new();
    let osc = graph.add_node(GraphNode::Stdlib(limestudio_core::stdlib::StdlibNode::Oscillator {
        freq: limestudio_core::ir::ParamRef::Const(440.0),
        wave: limestudio_core::stdlib::Waveform::Sine,
    }));
    let out = graph.add_node(GraphNode::Output { channel: 0 });
    graph.add_edge(osc, 0, out, 0);
    
    let preset = Preset {
        graph_snapshot: graph,
        ..Preset::default()
    };
    
    println!("Initial preset generated with {} nodes.", preset.graph_snapshot.nodes.len());
}
