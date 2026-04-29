use dirtydata_core::{ConfigValue, Graph, StableId};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub enum SemanticChange {
    NodeAdded {
        id: StableId,
        kind: String,
    },
    NodeRemoved {
        id: StableId,
        kind: String,
    },
    ParameterChanged {
        node_id: StableId,
        node_kind: String,
        param_name: String,
        old_value: Option<ConfigValue>,
        new_value: Option<ConfigValue>,
    },
    ConnectionAdded {
        source: StableId,
        target: StableId,
    },
    ConnectionRemoved {
        source: StableId,
        target: StableId,
    },
}

pub struct SemanticDiff;

impl SemanticDiff {
    pub fn compare(old: &Graph, new: &Graph) -> Vec<SemanticChange> {
        let mut changes = Vec::new();

        let old_nodes: HashMap<StableId, &dirtydata_core::Node> =
            old.nodes.iter().map(|(id, n)| (*id, n)).collect();
        let new_nodes: HashMap<StableId, &dirtydata_core::Node> =
            new.nodes.iter().map(|(id, n)| (*id, n)).collect();

        let old_ids: HashSet<StableId> = old_nodes.keys().cloned().collect();
        let new_ids: HashSet<StableId> = new_nodes.keys().cloned().collect();

        // 1. Nodes Added
        for id in new_ids.difference(&old_ids) {
            let node = new_nodes.get(id).unwrap();
            changes.push(SemanticChange::NodeAdded {
                id: *id,
                kind: format!("{:?}", node.kind),
            });
        }

        // 2. Nodes Removed
        for id in old_ids.difference(&new_ids) {
            let node = old_nodes.get(id).unwrap();
            changes.push(SemanticChange::NodeRemoved {
                id: *id,
                kind: format!("{:?}", node.kind),
            });
        }

        // 3. Parameters Changed (for existing nodes)
        for id in old_ids.intersection(&new_ids) {
            let old_node = old_nodes.get(id).unwrap();
            let new_node = new_nodes.get(id).unwrap();

            let old_keys: HashSet<&String> = old_node.config.keys().collect();
            let new_keys: HashSet<&String> = new_node.config.keys().collect();

            // All keys present in either node
            let all_keys: HashSet<&String> = old_keys.union(&new_keys).cloned().collect();

            for key in all_keys {
                let old_val = old_node.config.get(key);
                let new_val = new_node.config.get(key);

                if old_val != new_val {
                    changes.push(SemanticChange::ParameterChanged {
                        node_id: *id,
                        node_kind: format!("{:?}", old_node.kind),
                        param_name: key.clone(),
                        old_value: old_val.cloned(),
                        new_value: new_val.cloned(),
                    });
                }
            }
        }

        // 4. Connections (Simplified check)
        // Note: Real connection diffing requires comparing the edge list

        changes
    }
}
