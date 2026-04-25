use crate::graph::{PortType, PortInfo};
use crate::stdlib::{StdlibNode, Waveform};
use crate::ir::ParamRef;

#[derive(Clone, Debug)]
pub struct NodeDefinition {
    pub name: String,
    pub category: NodeCategory,
    pub inputs: Vec<PortInfo>,
    pub outputs: Vec<PortInfo>,
    pub template: crate::graph::GraphNode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeCategory {
    IO,
    Stdlib,
    Custom,
    Script,
}

pub struct NodeRegistry {
    pub definitions: Vec<NodeDefinition>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        let mut definitions = Vec::new();

        // 1. I/O Nodes
        definitions.push(NodeDefinition {
            name: "Input".into(),
            category: NodeCategory::IO,
            inputs: vec![],
            outputs: vec![PortInfo { name: "out".into(), port_type: PortType::AudioMono }],
            template: crate::graph::GraphNode::Input { channel: 0 },
        });

        definitions.push(NodeDefinition {
            name: "Output".into(),
            category: NodeCategory::IO,
            inputs: vec![PortInfo { name: "in".into(), port_type: PortType::AudioMono }],
            outputs: vec![],
            template: crate::graph::GraphNode::Output { channel: 0 },
        });

        // 2. Stdlib Nodes
        definitions.push(NodeDefinition {
            name: "Gain".into(),
            category: NodeCategory::Stdlib,
            inputs: vec![PortInfo { name: "in".into(), port_type: PortType::AudioMono }],
            outputs: vec![PortInfo { name: "out".into(), port_type: PortType::AudioMono }],
            template: crate::graph::GraphNode::Stdlib(StdlibNode::Gain { amount: ParamRef::Const(1.0) }),
        });

        definitions.push(NodeDefinition {
            name: "Mix".into(),
            category: NodeCategory::Stdlib,
            inputs: vec![
                PortInfo { name: "in1".into(), port_type: PortType::AudioMono },
                PortInfo { name: "in2".into(), port_type: PortType::AudioMono },
            ],
            outputs: vec![PortInfo { name: "out".into(), port_type: PortType::AudioMono }],
            template: crate::graph::GraphNode::Stdlib(StdlibNode::Mix { ratio: ParamRef::Const(0.5) }),
        });

        definitions.push(NodeDefinition {
            name: "Oscillator".into(),
            category: NodeCategory::Stdlib,
            inputs: vec![],
            outputs: vec![PortInfo { name: "out".into(), port_type: PortType::AudioMono }],
            template: crate::graph::GraphNode::Stdlib(StdlibNode::Oscillator { 
                freq: ParamRef::Const(440.0), 
                wave: Waveform::Sine 
            }),
        });

        definitions.push(NodeDefinition {
            name: "Pan".into(),
            category: NodeCategory::Stdlib,
            inputs: vec![PortInfo { name: "in".into(), port_type: PortType::AudioMono }],
            outputs: vec![
                PortInfo { name: "left".into(), port_type: PortType::AudioMono },
                PortInfo { name: "right".into(), port_type: PortType::AudioMono },
            ],
            template: crate::graph::GraphNode::Stdlib(StdlibNode::Pan { position: ParamRef::Const(0.5) }),
        });
        
        definitions.push(NodeDefinition {
            name: "Multiply".into(),
            category: NodeCategory::Stdlib,
            inputs: vec![
                PortInfo { name: "in1".into(), port_type: PortType::AudioMono },
                PortInfo { name: "in2".into(), port_type: PortType::AudioMono },
            ],
            outputs: vec![PortInfo { name: "out".into(), port_type: PortType::AudioMono }],
            template: crate::graph::GraphNode::Stdlib(StdlibNode::Multiply),
        });

        // 3. Script Nodes
        definitions.push(NodeDefinition {
            name: "Script Gain".into(),
            category: NodeCategory::Script,
            inputs: vec![PortInfo { name: "in".into(), port_type: PortType::AudioMono }],
            outputs: vec![PortInfo { name: "out".into(), port_type: PortType::AudioMono }],
            template: crate::graph::GraphNode::Script {
                source: r#"let x = input(0); let y = mul(x, 0.5); output(0, y);"#.into(),
                inputs: vec![PortInfo { name: "in".into(), port_type: PortType::AudioMono }],
                outputs: vec![PortInfo { name: "out".into(), port_type: PortType::AudioMono }],
            },
        });

        Self { definitions }
    }
}
