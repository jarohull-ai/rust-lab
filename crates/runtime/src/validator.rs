use crate::plan;
use crate::registry::NodeRegistry;
use anyhow::{anyhow, Result};
use lab_node_sdk::GraphSpec;
use std::collections::HashMap;

pub struct Validator<'a> {
    registry: &'a NodeRegistry,
}

impl<'a> Validator<'a> {
    pub fn new(registry: &'a NodeRegistry) -> Self {
        Self { registry }
    }

    pub fn validate(&self, graph: &GraphSpec) -> Result<()> {
        // 1. Nodes: unique ids, known kinds.
        let mut specs = HashMap::new();
        for node in &graph.nodes {
            let node_impl = self
                .registry
                .get(&node.kind)
                .ok_or_else(|| anyhow!("Unknown node kind: {}", node.kind))?;
            if specs.insert(node.id.as_str(), node_impl.spec()).is_some() {
                return Err(anyhow!("Duplicate node ID: {}", node.id));
            }
        }

        // 2. Edges: endpoints exist and ports are declared pins.
        for edge in &graph.edges {
            let from = specs
                .get(edge.from.node.as_str())
                .ok_or_else(|| anyhow!("Edge source node {} does not exist", edge.from.node))?;
            let to = specs
                .get(edge.to.node.as_str())
                .ok_or_else(|| anyhow!("Edge target node {} does not exist", edge.to.node))?;

            if !from.outputs.iter().any(|p| p.name == edge.from.port) {
                return Err(anyhow!(
                    "Node '{}' [{}] has no output '{}'",
                    edge.from.node,
                    from.name,
                    edge.from.port
                ));
            }
            if !to.inputs.iter().any(|p| p.name == edge.to.port) {
                return Err(anyhow!(
                    "Node '{}' [{}] has no input '{}'",
                    edge.to.node,
                    to.name,
                    edge.to.port
                ));
            }
        }

        // 3. Condition branches point at existing, distinct nodes.
        for node in graph
            .nodes
            .iter()
            .filter(|n| n.kind == plan::CONDITION_KIND)
        {
            let mut targets = Vec::new();
            for key in plan::BRANCH_KEYS {
                let target = node
                    .config
                    .get(key)
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Condition '{}' needs '{key}' in config", node.id))?;
                if !specs.contains_key(target) {
                    return Err(anyhow!(
                        "Condition '{}' {key} points to unknown node '{target}'",
                        node.id
                    ));
                }
                if target == node.id {
                    return Err(anyhow!("Condition '{}' cannot branch to itself", node.id));
                }
                targets.push(target);
            }
            if targets[0] == targets[1] {
                return Err(anyhow!(
                    "Condition '{}' has identical true/false branches",
                    node.id
                ));
            }
        }

        // 4. `after` lists point at existing, distinct nodes.
        for node in &graph.nodes {
            let mut seen = std::collections::HashSet::new();
            for dep in &node.after {
                if dep == &node.id {
                    return Err(anyhow!("Node '{}' cannot run after itself", node.id));
                }
                if !specs.contains_key(dep.as_str()) {
                    return Err(anyhow!(
                        "Node '{}' has 'after' on unknown node '{dep}'",
                        node.id
                    ));
                }
                if !seen.insert(dep) {
                    return Err(anyhow!("Node '{}' lists '{dep}' twice in 'after'", node.id));
                }
            }
        }

        // 5. Cycles over data, condition and `after` edges.
        plan::execution_order(graph)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes;
    use serde_json::json;

    fn validate(value: serde_json::Value) -> Result<()> {
        let mut registry = NodeRegistry::new();
        nodes::register_all(&mut registry);
        let graph: GraphSpec = serde_json::from_value(value).unwrap();
        Validator::new(&registry).validate(&graph)
    }

    #[test]
    fn rejects_unknown_ports() {
        let err = validate(json!({
            "graph_version": "0.1",
            "nodes": [
                {"id": "get", "kind": "http_get", "config": {"url": "http://x"}},
                {"id": "parse", "kind": "json_parse", "config": {}}
            ],
            "edges": [{"from": {"node": "get", "port": "body"}, "to": {"node": "parse", "port": "json"}}]
        }))
        .unwrap_err()
        .to_string();
        assert!(err.contains("has no input 'json'"), "{err}");
    }

    #[test]
    fn rejects_bad_condition_branches() {
        let err = validate(json!({
            "graph_version": "0.1",
            "nodes": [{"id": "if", "kind": "condition", "config": {"value": true, "true_branch": "ghost", "false_branch": "if"}}],
            "edges": []
        }))
        .unwrap_err()
        .to_string();
        assert!(err.contains("unknown node 'ghost'"), "{err}");
    }

    #[test]
    fn rejects_bad_after_lists() {
        let unknown = validate(json!({
            "graph_version": "0.1",
            "nodes": [{"id": "log", "kind": "log_output", "config": {"text": "x"}, "after": ["ghost"]}],
            "edges": []
        }))
        .unwrap_err()
        .to_string();
        assert!(
            unknown.contains("'after' on unknown node 'ghost'"),
            "{unknown}"
        );

        let cycle = validate(json!({
            "graph_version": "0.1",
            "nodes": [
                {"id": "a", "kind": "delay", "config": {"duration": "1ms"}, "after": ["b"]},
                {"id": "b", "kind": "delay", "config": {"duration": "1ms"}, "after": ["a"]}
            ],
            "edges": []
        }))
        .unwrap_err()
        .to_string();
        assert!(cycle.contains("Cycle detected"), "{cycle}");
    }
}
