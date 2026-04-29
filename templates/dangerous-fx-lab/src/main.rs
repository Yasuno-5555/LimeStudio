use limestudio_core::graph::GraphBuilder;
use limestudio_core::preset::PresetArtifact;
use std::collections::HashMap;

fn main() {
    println!("Dangerous FX Lab - Visible Destruction");

    let builder = GraphBuilder::new();
    let graph = builder.build();
    let _preset = PresetArtifact::new("Chaos Factory".to_string(), graph, HashMap::new(), None);

    println!("Dangerous Preset 'Chaos Factory' generated.");
    println!("Trust UI Focus: Highlighting unstable filter poles and NaN propagation risks.");
}
