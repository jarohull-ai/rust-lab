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

        // 2. Edges: endpoints exist, ports are declared pins, and types are compatible.
        for edge in &graph.edges {
            let from = specs
                .get(edge.from.node.as_str())
                .ok_or_else(|| anyhow!("Edge source node {} does not exist", edge.from.node))?;
            let to = specs
                .get(edge.to.node.as_str())
                .ok_or_else(|| anyhow!("Edge target node {} does not exist", edge.to.node))?;

            let from_pin = from
                .outputs
                .iter()
                .find(|p| p.name == edge.from.port)
                .ok_or_else(|| {
                    anyhow!(
                        "Node '{}' [{}] has no output '{}'",
                        edge.from.node,
                        from.name,
                        edge.from.port
                    )
                })?;

            let to_pin = to
                .inputs
                .iter()
                .find(|p| p.name == edge.to.port)
                .ok_or_else(|| {
                    anyhow!(
                        "Node '{}' [{}] has no input '{}'",
                        edge.to.node,
                        to.name,
                        edge.to.port
                    )
                })?;

            if !from_pin.pin_type.is_compatible_with(&to_pin.pin_type) {
                return Err(anyhow!(
                    "Type mismatch on edge from '{}.{}' [{}] to '{}.{}' [{}]",
                    edge.from.node,
                    edge.from.port,
                    from_pin.pin_type,
                    edge.to.node,
                    edge.to.port,
                    to_pin.pin_type
                ));
            }
        }

        // 3. Static config validation: verify types against declared input pins & check required inputs.
        for node in &graph.nodes {
            let spec = &specs[node.id.as_str()];
            let config_map = match &node.config {
                serde_json::Value::Object(map) => Some(map),
                _ => None,
            };

            if let Some(map) = config_map {
                for (key, val) in map {
                    let key_lower = key.to_lowercase();
                    if let Some(pin) = spec
                        .inputs
                        .iter()
                        .find(|p| p.name.to_lowercase() == key_lower || (p.name == "path" && (key_lower == "file_path" || key_lower == "filepath")))
                    {
                        if !pin.pin_type.matches_value(val) {
                            return Err(anyhow!(
                                "Config type mismatch on node '{}', pin '{}': expected {}, got {}",
                                node.id,
                                key,
                                pin.pin_type,
                                json_type_name(val)
                            ));
                        }
                    }
                }
            }

            // Check that every required input is supplied either by an incoming edge or config.
            for pin in spec.inputs.iter().filter(|p| p.required) {
                let pin_lower = pin.name.to_lowercase();
                let has_incoming_edge = graph.edges.iter().any(|e| {
                    e.to.node == node.id
                        && (e.to.port.to_lowercase() == pin_lower
                            || (pin.name == "path" && (e.to.port.to_lowercase() == "file_path" || e.to.port.to_lowercase() == "filepath")))
                });

                let has_valid_config = config_map.is_some_and(|map| {
                    let val = map
                        .get(&pin.name)
                        .or_else(|| map.get(&pin_lower))
                        .or_else(|| {
                            if pin.name == "path" {
                                map.get("file_path").or_else(|| map.get("filepath"))
                            } else {
                                None
                            }
                        });
                    match val {
                        Some(serde_json::Value::Null) | None => false,
                        Some(serde_json::Value::String(s)) => !s.trim().is_empty(),
                        Some(_) => true,
                    }
                });

                if !has_incoming_edge && !has_valid_config {
                    return Err(anyhow!(
                        "Graph is invalid: node '{}' [{}] missing required input '{}'",
                        node.id,
                        node.kind,
                        pin.name
                    ));
                }
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

fn json_type_name(val: &serde_json::Value) -> &'static str {
    match val {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
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
    fn edge_string_to_string_passes() {
        let res = validate(json!({
            "graph_version": "0.1",
            "nodes": [
                {"id": "get", "kind": "http_get", "config": {"url": "http://x"}},
                {"id": "parse", "kind": "json_parse", "config": {}}
            ],
            "edges": [{"from": {"node": "get", "port": "body"}, "to": {"node": "parse", "port": "json_str"}}]
        }));
        assert!(res.is_ok());
    }

    #[test]
    fn edge_string_to_bool_fails_with_error() {
        let err = validate(json!({
            "graph_version": "0.1",
            "nodes": [
                {"id": "get", "kind": "http_get", "config": {"url": "http://x"}},
                {"id": "check", "kind": "condition", "config": {"true_branch": "a", "false_branch": "b"}},
                {"id": "a", "kind": "delay", "config": {"duration": "1ms"}},
                {"id": "b", "kind": "delay", "config": {"duration": "1ms"}}
            ],
            "edges": [{"from": {"node": "get", "port": "body"}, "to": {"node": "check", "port": "value"}}]
        }))
        .unwrap_err()
        .to_string();
        assert_eq!(
            err,
            "Type mismatch on edge from 'get.body' [string] to 'check.value' [boolean]"
        );
    }

    #[test]
    fn log_output_accepts_any_from_string_json_and_other_types() {
        // http_get.body [string] -> log_output.text [any]
        let string_res = validate(json!({
            "graph_version": "0.1",
            "nodes": [
                {"id": "get", "kind": "http_get", "config": {"url": "http://x"}},
                {"id": "log", "kind": "log_output", "config": {}}
            ],
            "edges": [{"from": {"node": "get", "port": "body"}, "to": {"node": "log", "port": "text"}}]
        }));
        assert!(string_res.is_ok());

        // json_parse.value [json] -> log_output.text [any]
        let json_res = validate(json!({
            "graph_version": "0.1",
            "nodes": [
                {"id": "parse", "kind": "json_parse", "config": {"json_str": "{}"}},
                {"id": "log", "kind": "log_output", "config": {}}
            ],
            "edges": [{"from": {"node": "parse", "port": "value"}, "to": {"node": "log", "port": "text"}}]
        }));
        assert!(json_res.is_ok());
    }

    #[test]
    fn config_validates_pin_types() {
        // bool matches boolean
        let valid_bool = validate(json!({
            "graph_version": "0.1",
            "nodes": [
                {"id": "check", "kind": "condition", "config": {"value": true, "true_branch": "a", "false_branch": "b"}},
                {"id": "a", "kind": "delay", "config": {"duration": "1ms"}},
                {"id": "b", "kind": "delay", "config": {"duration": "1ms"}}
            ],
            "edges": []
        }));
        assert!(valid_bool.is_ok());

        // string does NOT match boolean
        let invalid_bool = validate(json!({
            "graph_version": "0.1",
            "nodes": [
                {"id": "check", "kind": "condition", "config": {"value": "true", "true_branch": "a", "false_branch": "b"}},
                {"id": "a", "kind": "delay", "config": {"duration": "1ms"}},
                {"id": "b", "kind": "delay", "config": {"duration": "1ms"}}
            ],
            "edges": []
        }))
        .unwrap_err()
        .to_string();
        assert!(invalid_bool.contains("Config type mismatch on node 'check', pin 'value': expected boolean, got string"), "{invalid_bool}");

        // number does NOT match string
        let invalid_string = validate(json!({
            "graph_version": "0.1",
            "nodes": [
                {"id": "get", "kind": "http_get", "config": {"url": 12345}}
            ],
            "edges": []
        }))
        .unwrap_err()
        .to_string();
        assert!(invalid_string.contains("Config type mismatch on node 'get', pin 'url': expected string, got number"), "{invalid_string}");
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

    #[test]
    fn rejects_missing_required_input_pin() {
        let err = validate(json!({
            "graph_version": "0.1",
            "nodes": [
                {"id": "reader", "kind": "fs_read", "config": {}}
            ],
            "edges": []
        }))
        .unwrap_err()
        .to_string();

        assert!(
            err.contains("Graph is invalid: node 'reader' [fs_read] missing required input 'path'"),
            "{err}"
        );
    }

    #[test]
    fn accepts_file_path_alias_and_lowercase() {
        let res = validate(json!({
            "graph_version": "0.1",
            "nodes": [
                {"id": "reader", "kind": "fs_read", "config": {"file_path": "test.txt"}}
            ],
            "edges": []
        }));
        assert!(res.is_ok());
    }
}

