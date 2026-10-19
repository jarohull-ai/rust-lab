use lab_node_sdk::ViperNode;
use std::collections::HashMap;
use std::sync::Arc;

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
}
