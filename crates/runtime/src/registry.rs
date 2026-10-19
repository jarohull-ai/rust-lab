use lab_node_sdk::ViperNode;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct NodeRegistry {
    nodes: HashMap<String, Arc<dyn ViperNode>>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn register(&mut self, node: Arc<dyn ViperNode>) {
        let spec = node.spec();
        self.nodes.insert(spec.name.clone(), node);
    }

    pub fn get(&self, kind: &str) -> Option<Arc<dyn ViperNode>> {
        self.nodes.get(kind).cloned()
    }

    pub fn all_specs(&self) -> Vec<lab_node_sdk::NodeSpec> {
        let mut specs: Vec<_> = self.nodes.values().map(|n| n.spec()).collect();
        specs.sort_by(|a, b| a.name.cmp(&b.name));
        specs
    }
}

impl Default for NodeRegistry {
    fn default() -> Self {
        let mut reg = Self::new();
        crate::nodes::register_all(&mut reg);
        reg
    }
}
