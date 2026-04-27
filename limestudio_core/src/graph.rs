pub use dirtydata_core::types::StableId;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum NodeDef {
    Input,
    Output,
    Processor { kind: String, params: HashMap<String, ParamSource> },
}

#[derive(Debug, Clone)]
pub enum ParamSource {
    Constant(f32),
    Parameter(String), // Reference to NIH-plug parameter ID
}

impl From<f32> for ParamSource {
    fn from(f: f32) -> Self {
        ParamSource::Constant(f)
    }
}

#[derive(Debug, Clone)]
pub struct EdgeDef {
    pub from_node: StableId,
    pub from_port: String,
    pub to_node: StableId,
    pub to_port: String,
}

pub struct GraphBuilder {
    pub nodes: HashMap<StableId, NodeDef>,
    pub edges: Vec<EdgeDef>,
    input_node: StableId,
    output_node: StableId,
}

impl GraphBuilder {
    pub fn new() -> Self {
        let input_node = StableId::new();
        let output_node = StableId::new();
        let mut nodes = HashMap::new();
        nodes.insert(input_node, NodeDef::Input);
        nodes.insert(output_node, NodeDef::Output);

        Self {
            nodes,
            edges: Vec::new(),
            input_node,
            output_node,
        }
    }

    pub fn input_node(&self) -> StableId { self.input_node }
    pub fn output_node(&self) -> StableId { self.output_node }

    pub fn add_node(&mut self, def: NodeDef) -> StableId {
        let id = StableId::new();
        self.nodes.insert(id, def);
        id
    }

    pub fn add_processor(&mut self, kind: &str, params: Vec<(&str, ParamSource)>) -> StableId {
        let mut p_map = HashMap::new();
        for (k, v) in params {
            p_map.insert(k.to_string(), v);
        }
        self.add_node(NodeDef::Processor {
            kind: kind.to_string(),
            params: p_map,
        })
    }

    pub fn add_edge(&mut self, from: StableId, from_port: &str, to: StableId, to_port: &str) {
        self.edges.push(EdgeDef {
            from_node: from,
            from_port: from_port.to_string(),
            to_node: to,
            to_port: to_port.to_string(),
        });
    }

    pub fn connect(&mut self, from: StableId, to: StableId) {
        self.add_edge(from, "out", to, "in");
    }

    pub fn build(self) -> dirtydata_core::ir::Graph {
        let mut graph = dirtydata_core::ir::Graph::new();
        
        // Map our IDs to DirtyData nodes
        let mut id_map = HashMap::new();
        
        for (id, def) in self.nodes {
            let node = match def {
                NodeDef::Input => dirtydata_core::ir::Node::new_input_proxy("input"),
                NodeDef::Output => dirtydata_core::ir::Node::new_output_proxy("output"),
                NodeDef::Processor { kind, params } => {
                    let mut node = dirtydata_core::ir::Node::new_processor(&kind);
                    node.config.insert("name".to_string(), dirtydata_core::types::ConfigValue::String(kind.clone()));
                    for (k, v) in params {
                        let val = match v {
                            ParamSource::Constant(f) => dirtydata_core::types::ConfigValue::Float(f as f64),
                            ParamSource::Parameter(id) => dirtydata_core::types::ConfigValue::String(format!("param:{}", id)),
                        };
                        node.config.insert(k.to_string(), val);
                    }
                    node
                }
            };
            let real_id = node.id;
            id_map.insert(id, real_id);
            graph.add_node(node);
        }

        for edge in self.edges {
            let source_id = id_map.get(&edge.from_node).unwrap();
            let target_id = id_map.get(&edge.to_node).unwrap();
            
            let source = dirtydata_core::types::PortRef {
                node_id: *source_id,
                port_name: edge.from_port,
            };
            let target = dirtydata_core::types::PortRef {
                node_id: *target_id,
                port_name: edge.to_port,
            };
            
            let _ = graph.connect(source, target);
        }

        graph
    }
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}
