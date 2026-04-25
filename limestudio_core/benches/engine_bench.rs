use criterion::{black_box, criterion_group, criterion_main, Criterion};
use limestudio_core::ir::{IrOp, BufferId};
use limestudio_core::graph::{AudioGraph, GraphNode};
use limestudio_core::compile::compile_graph;
use limestudio_core::validate::validate_graph;
use limestudio_core::engine::DspEngine;

fn bench_engine(c: &mut Criterion) {
    // 1. Basic Gain Graph
    let mut g = AudioGraph::new();
    let input = g.add_node(GraphNode::Input);
    let output = g.add_node(GraphNode::Output);
    let gain = g.add_node(GraphNode::Custom {
        ops: vec![
            IrOp::LoadBuffer(BufferId(input.0 as u32)),
            IrOp::MulConst(0.5),
            IrOp::StoreBuffer(BufferId(output.0 as u32)),
        ]
    });
    g.add_edge(input, 0, gain, 0);
    g.add_edge(gain, 0, output, 0);
    
    let order = validate_graph(&g).unwrap();
    let program = compile_graph(&g, &order).program;
    let mut engine = DspEngine::new_from_program(program);
    
    let in_l = vec![0.1; 512];
    let in_r = vec![0.2; 512];
    let mut out_l = vec![0.0; 512];
    let mut out_r = vec![0.0; 512];
    let inputs = [&in_l[..], &in_r[..]];
    let mut outputs = [&mut out_l[..], &mut out_r[..]];

    c.bench_function("engine_process_block_basic_gain", |b| {
        b.iter(|| {
            engine.process_block(black_box(&inputs), black_box(&mut outputs));
        })
    });

    // 2. 1000 Node Graph
    let mut g_big = AudioGraph::new();
    let mut prev = g_big.add_node(GraphNode::Input);
    for _ in 0..1000 {
        let node = g_big.add_node(GraphNode::Custom {
            ops: vec![
                IrOp::LoadBuffer(BufferId(prev.0 as u32)),
                IrOp::MulConst(0.999),
                IrOp::StoreBuffer(BufferId(9999)), // placeholder, compile will fix buffer ids if we do it right
                // In this brutal Level 0, we use fixed buffer IDs for now.
            ]
        });
        // Correcting for the compiler's simple logic: NodeId(i) writes to BufferId(i)
        if let GraphNode::Custom { ref mut ops } = g_big.nodes[node.0] {
             ops[2] = IrOp::StoreBuffer(BufferId(node.0 as u32));
        }
        
        g_big.add_edge(prev, 0, node, 0);
        prev = node;
    }
    let output_node = g_big.add_node(GraphNode::Output);
    g_big.add_edge(prev, 0, output_node, 0);
    
    let order_big = validate_graph(&g_big).unwrap();
    let program_big = compile_graph(&g_big, &order_big).program;
    let mut engine_big = DspEngine::new_from_program(program_big);

    c.bench_function("engine_process_block_1000_nodes", |b| {
        b.iter(|| {
            engine_big.process_block(black_box(&inputs), black_box(&mut outputs));
        })
    });
}

criterion_group!(benches, bench_engine);
criterion_main!(benches);
