use limestudio_core::graph::{AudioGraph, GraphNode, NodeId};
use limestudio_core::ir::IrOp;
use limestudio_plugin::lime_plugin_raw;

fn create_graph() -> AudioGraph {
    let mut graph = AudioGraph::new();
    let in_node = graph.add_node(GraphNode::Input { channel: 0 });
    let out_node = graph.add_node(GraphNode::Output { channel: 0 });

    // 0.5倍のゲインを実装する命令列 (Brutal Level 0)
    let gain_ops = vec![
        // Left Channel
        IrOp::ReadInput { channel: 0 },
        IrOp::MulConst(0.5),
        IrOp::WriteOutput { channel: 0 },
        
        // Right Channel
        IrOp::ReadInput { channel: 1 },
        IrOp::MulConst(0.5),
        IrOp::WriteOutput { channel: 1 },
    ];
    let gain_node = graph.add_node(GraphNode::Custom {
        ops: gain_ops,
        inputs: vec![limestudio_core::graph::PortInfo { 
            name: "in".into(), 
            port_type: limestudio_core::graph::PortType::AudioMono 
        }],
        outputs: vec![limestudio_core::graph::PortInfo { 
            name: "out".into(), 
            port_type: limestudio_core::graph::PortType::AudioMono 
        }],
    });

    // 接続 (Level 0 のコンパイラは edges を見て実行順序を決める)
    graph.add_edge(in_node, 0, gain_node, 0);
    graph.add_edge(gain_node, 0, out_node, 0);

    graph
}

// グラフ生成関数を渡してプラグインをエクスポート
lime_plugin_raw!(create_graph);
