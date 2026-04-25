use limestudio_core::graph::{AudioGraph, GraphNode, PortType, PortInfo};
use limestudio_core::stdlib::StdlibNode;
use limestudio_core::ir::{IrOp, ParamRef};
use limestudio_core::compile::compile_graph;
use limestudio_core::validate::validate_graph;
use std::fs;
use std::path::PathBuf;

fn run_snapshot_test(name: &str, graph: &AudioGraph) {
    let order = validate_graph(graph).expect("Validation failed");
    let program = compile_graph(graph, &order).program;
    
    let mut output = String::new();
    output.push_str(&format!("--- Snapshot: {} ---\n", name));
    output.push_str(&format!("Buffer Count: {}\n", program.buffer_count));
    output.push_str(&format!("State Count:  {}\n", program.state_count));
    output.push_str("Operations:\n");
    for (i, op) in program.ops.iter().enumerate() {
        output.push_str(&format!("  {:3}: {}\n", i, op));
    }
    output.push_str("-------------------\n");

    let snapshot_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots");
    if !snapshot_dir.exists() {
        fs::create_dir_all(&snapshot_dir).unwrap();
    }
    
    let snapshot_path = snapshot_dir.join(format!("{}.txt", name));
    
    if std::env::var("LIME_SNAPSHOT_UPDATE").is_ok() {
        fs::write(&snapshot_path, &output).unwrap();
        println!("Updated snapshot: {:?}", snapshot_path);
    } else {
        if !snapshot_path.exists() {
            panic!("Snapshot file missing: {:?}. Run with LIME_SNAPSHOT_UPDATE=1 to generate.", snapshot_path);
        }
        let expected = fs::read_to_string(&snapshot_path).unwrap();
        if output != expected {
            let diff_path = snapshot_dir.join(format!("{}.actual.txt", name));
            fs::write(&diff_path, &output).unwrap();
            panic!("Snapshot mismatch for {}. Actual output written to {:?}", name, diff_path);
        }
    }
}

#[test]
fn test_snapshot_basic_gain() {
    let mut g = AudioGraph::new();
    let in_n = g.add_node(GraphNode::Input { channel: 0 });
    let out_n = g.add_node(GraphNode::Output { channel: 0 });
    let gain_n = g.add_node(GraphNode::Stdlib(StdlibNode::Gain {
        amount: ParamRef::Const(0.5)
    }));
    g.add_edge(in_n, 0, gain_n, 0);
    g.add_edge(gain_n, 0, out_n, 0);
    
    run_snapshot_test("basic_gain", &g);
}

#[test]
fn test_snapshot_pan_split() {
    let mut g = AudioGraph::new();
    let in_n = g.add_node(GraphNode::Input { channel: 0 });
    let pan_n = g.add_node(GraphNode::Stdlib(StdlibNode::Pan {
        position: ParamRef::Const(0.3)
    }));
    let out_l = g.add_node(GraphNode::Output { channel: 0 });
    let out_r = g.add_node(GraphNode::Output { channel: 1 });
    
    g.add_edge(in_n, 0, pan_n, 0);
    g.add_edge(pan_n, 0, out_l, 0); // Left
    g.add_edge(pan_n, 1, out_r, 0); // Right
    
    run_snapshot_test("pan_split", &g);
}
