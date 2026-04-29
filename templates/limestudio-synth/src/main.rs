use limestudio_core::graph::{GraphBuilder, ParamSource};
use limestudio_core::preset::PresetArtifact;
use std::collections::HashMap;

fn main() {
    println!("LimeStudio Synth Template");

    // Create a basic polyphonic sine synth
    let mut builder = GraphBuilder::new();
    let osc = builder.add_processor("Oscillator", vec![("freq", ParamSource::Constant(440.0))]);
    builder.connect(osc, builder.output_node());

    let graph = builder.build();
    let preset = PresetArtifact::new("Synth Template".to_string(), graph, HashMap::new(), None);

    println!(
        "Initial preset generated with {} nodes.",
        preset.graph.nodes.len()
    );
}
