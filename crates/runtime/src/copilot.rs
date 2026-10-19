//! Copilot AI / SEED Rule Engine for Rust Lab.
//! Provides intelligent, deterministic, real-time contextual hints and suggestions
//! based on static graph analysis without any external LLM dependencies.

use crate::plan;
use crate::registry::NodeRegistry;
use lab_node_sdk::{GraphSpec, PinType};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HintSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hint {
    pub severity: HintSeverity,
    pub message: String,
    pub node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

pub fn generate_hints(graph: &GraphSpec, registry: &NodeRegistry) -> Vec<Hint> {
    let mut hints = Vec::new();
    let node_map: HashMap<&str, &lab_node_sdk::NodeInstance> =
        graph.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    // 1. Cycle detection (Rule 3 - Error)
    if let Err(e) = plan::execution_order(graph) {
        let err_str = e.to_string();
        if err_str.contains("Cycle detected") {
            hints.push(Hint {
                severity: HintSeverity::Error,
                message: format!(
                    "Wykryto cykl w grafie ({err_str}). Executor wymaga grafu acyklicznego (DAG) – rozerwij pętlę."
                ),
                node_id: None,
                suggestion: Some("Usuń lub przekieruj krawędź tworzącą pętlę.".into()),
            });
        }
    }

    // Index incoming and outgoing edges per node
    let mut incoming_edges: HashMap<&str, Vec<&lab_node_sdk::Edge>> = HashMap::new();
    let mut outgoing_edges: HashMap<&str, Vec<&lab_node_sdk::Edge>> = HashMap::new();
    for edge in &graph.edges {
        incoming_edges.entry(&edge.to.node).or_default().push(edge);
        outgoing_edges.entry(&edge.from.node).or_default().push(edge);
    }

    // Condition branch targets
    let mut condition_targets: HashSet<&str> = HashSet::new();
    for node in &graph.nodes {
        if node.kind == "condition" {
            if let Some(t) = node.config.get("true_branch").and_then(|v| v.as_str()) {
                condition_targets.insert(t);
            }
            if let Some(f) = node.config.get("false_branch").and_then(|v| v.as_str()) {
                condition_targets.insert(f);
            }
        }
    }

    // Check each node
    for node in &graph.nodes {
        let handler = match registry.get(&node.kind) {
            Some(h) => h,
            None => {
                hints.push(Hint {
                    severity: HintSeverity::Error,
                    message: format!("Nieznany typ węzła `{}` w `{}`.", node.kind, node.id),
                    node_id: Some(node.id.clone()),
                    suggestion: Some("Wybierz poprawny rodzaj węzła z palety.".into()),
                });
                continue;
            }
        };
        let spec = handler.spec();
        let config_map = node.config.as_object();

        // Rule 1: Missing required inputs (Warning)
        for pin in spec.inputs.iter().filter(|p| p.required) {
            let pin_lower = pin.name.to_lowercase();
            let has_edge = incoming_edges
                .get(node.id.as_str())
                .map(|edges| {
                    edges.iter().any(|e| {
                        e.to.port.to_lowercase() == pin_lower
                            || (pin.name == "path"
                                && (e.to.port.to_lowercase() == "file_path"
                                    || e.to.port.to_lowercase() == "filepath"))
                    })
                })
                .unwrap_or(false);

            let has_config = config_map.is_some_and(|map| {
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

            if !has_edge && !has_config {
                let suggestion = match (node.kind.as_str(), pin.name.as_str()) {
                    ("fs_read", "path") => {
                        "Wpisz ścieżkę do pliku w konfiguracji (np. `data/input.txt`) lub połącz z `string_to_path`."
                    }
                    ("fs_write", "path") => {
                        "Wpisz ścieżkę do pliku wyjściowego (np. `output.txt`) lub połącz z `string_to_path`."
                    }
                    ("fs_write", "content") => {
                        "Połącz wejście `content` z wyjściem węzła tekstowego (np. `text_clean`, `json_to_string`)."
                    }
                    ("http_get", "url") => {
                        "Podaj poprawny URL w konfiguracji (np. `https://httpbin.org/json`)."
                    }
                    ("json_parse", "json_str") => {
                        "Połącz z wyjściem String lub wpisz ciąg JSON w konfiguracji."
                    }
                    _ => "Podaj wartość w konfiguracji lub połącz z odpowiednim wyjściem.",
                };
                hints.push(Hint {
                    severity: HintSeverity::Warning,
                    message: format!(
                        "Węzeł `{}` [{}] nie ma podanej wartości dla wymaganego wejścia `{}`.",
                        node.id, node.kind, pin.name
                    ),
                    node_id: Some(node.id.clone()),
                    suggestion: Some(suggestion.into()),
                });
            }
        }

        // Rule 2: Orphan node (Warning)
        let in_count = incoming_edges
            .get(node.id.as_str())
            .map(|v| v.len())
            .unwrap_or(0);
        let out_count = outgoing_edges
            .get(node.id.as_str())
            .map(|v| v.len())
            .unwrap_or(0);
        let is_condition_target = condition_targets.contains(node.id.as_str());
        let has_after = !node.after.is_empty();
        let is_after_target = graph
            .nodes
            .iter()
            .any(|n| n.after.iter().any(|a| a == &node.id));

        if graph.nodes.len() > 1
            && in_count == 0
            && out_count == 0
            && !is_condition_target
            && !has_after
            && !is_after_target
        {
            hints.push(Hint {
                severity: HintSeverity::Warning,
                message: format!(
                    "Węzeł `{}` [{}] jest osierocony (brak jakichkolwiek połączeń w grafie).",
                    node.id, node.kind
                ),
                node_id: Some(node.id.clone()),
                suggestion: Some(
                    "Połącz go z resztą pipeline'u lub usuń z canvasu klawiszem Delete.".into(),
                ),
            });
        }

        // Rule 6: Next node suggestion for unconsumed outputs (Info)
        if out_count == 0 && !spec.outputs.is_empty() {
            let suggestion = match node.kind.as_str() {
                "http_get" => Some(
                    "Masz wyjście `body` (String). Połącz z `json_parse.json_str` lub `log_output.text`.",
                ),
                "fs_read" => Some(
                    "Masz wyjście `content` (String). Połącz z `text_clean.text`, `json_parse.json_str` lub `log_output.text`.",
                ),
                "json_parse" => Some(
                    "Masz wyjście `value` (Json). Użyj `json_to_string` przed zapisem lub `json_to_bool` przed warunkiem.",
                ),
                "text_clean" => Some(
                    "Masz wyjście `cleaned` (String). Połącz z `log_output.text` lub `fs_write.content` (poprzez `string_to_path`).",
                ),
                "string_to_path" => {
                    Some("Masz wyjście `path` (FilePath). Połącz z `fs_read.path` lub `fs_write.path`.")
                }
                _ => None,
            };

            if let Some(sugg) = suggestion {
                hints.push(Hint {
                    severity: HintSeverity::Info,
                    message: format!(
                        "Węzeł `{}` [{}] zakończył przetwarzanie, ale jego wyjście nie jest połączone.",
                        node.id, node.kind
                    ),
                    node_id: Some(node.id.clone()),
                    suggestion: Some(sugg.into()),
                });
            }
        }
    }

    // Rule 4: Type mismatch on edges (Error)
    for edge in &graph.edges {
        if let (Some(from_node), Some(to_node)) = (
            node_map.get(edge.from.node.as_str()),
            node_map.get(edge.to.node.as_str()),
        ) {
            if let (Some(from_handler), Some(to_handler)) = (
                registry.get(&from_node.kind),
                registry.get(&to_node.kind),
            ) {
                let from_spec = from_handler.spec();
                let to_spec = to_handler.spec();

                let from_pin = from_spec.outputs.iter().find(|p| p.name == edge.from.port);
                let to_pin = to_spec.inputs.iter().find(|p| p.name == edge.to.port);

                match (from_pin, to_pin) {
                    (None, _) => {
                        hints.push(Hint {
                            severity: HintSeverity::Error,
                            message: format!(
                                "Węzeł `{}` [{}] nie posiada wyjścia `{}`.",
                                edge.from.node, from_node.kind, edge.from.port
                            ),
                            node_id: Some(edge.from.node.clone()),
                            suggestion: Some("Sprawdź dostępne wyjścia w specyfikacji węzła.".into()),
                        });
                    }
                    (_, None) => {
                        hints.push(Hint {
                            severity: HintSeverity::Error,
                            message: format!(
                                "Węzeł `{}` [{}] nie posiada wejścia `{}`.",
                                edge.to.node, to_node.kind, edge.to.port
                            ),
                            node_id: Some(edge.to.node.clone()),
                            suggestion: Some("Sprawdź dostępne wejścia w specyfikacji węzła.".into()),
                        });
                    }
                    (Some(fp), Some(tp)) => {
                        if !fp.pin_type.is_compatible_with(&tp.pin_type) {
                            let conv_suggestion = match (&fp.pin_type, &tp.pin_type) {
                                (PinType::String, PinType::FilePath) => {
                                    "Wstaw konwerter `string_to_path` pomiędzy tymi węzłami."
                                }
                                (PinType::Json, PinType::String) => {
                                    "Wstaw konwerter `json_to_string` pomiędzy tymi węzłami."
                                }
                                (PinType::Json, PinType::Boolean) => {
                                    "Wstaw konwerter `json_to_bool` pomiędzy tymi węzłami."
                                }
                                _ => "Użyj dedykowanego węzła konwertera lub zmień połączenie.",
                            };
                            hints.push(Hint {
                                severity: HintSeverity::Error,
                                message: format!(
                                    "Niezgodność typów: `{}.{}` [{}] → `{}.{}` [{}].",
                                    edge.from.node,
                                    edge.from.port,
                                    fp.pin_type,
                                    edge.to.node,
                                    edge.to.port,
                                    tp.pin_type
                                ),
                                node_id: Some(edge.to.node.clone()),
                                suggestion: Some(conv_suggestion.into()),
                            });
                        }
                    }
                }
            }
        }
    }

    hints
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes;
    use serde_json::json;

    fn test_registry() -> NodeRegistry {
        let mut reg = NodeRegistry::new();
        nodes::register_all(&mut reg);
        reg
    }

    #[test]
    fn copilot_detects_missing_required_input() {
        let reg = test_registry();
        let graph: GraphSpec = serde_json::from_value(json!({
            "graph_version": "0.1",
            "nodes": [{"id": "reader", "kind": "fs_read", "config": {}}],
            "edges": []
        }))
        .unwrap();

        let hints = generate_hints(&graph, &reg);
        assert!(hints.iter().any(|h| h.severity == HintSeverity::Warning
            && h.message.contains("nie ma podanej wartości dla wymaganego wejścia `path`")));
    }

    #[test]
    fn copilot_detects_orphan_node() {
        let reg = test_registry();
        let graph: GraphSpec = serde_json::from_value(json!({
            "graph_version": "0.1",
            "nodes": [
                {"id": "cleaner", "kind": "text_clean", "config": {"text": "hello"}},
                {"id": "orphan", "kind": "log_output", "config": {"text": "lonely"}}
            ],
            "edges": []
        }))
        .unwrap();

        let hints = generate_hints(&graph, &reg);
        assert!(hints
            .iter()
            .any(|h| h.node_id.as_deref() == Some("orphan") && h.message.contains("jest osierocony")));
    }

    #[test]
    fn copilot_detects_type_mismatch() {
        let reg = test_registry();
        let graph: GraphSpec = serde_json::from_value(json!({
            "graph_version": "0.1",
            "nodes": [
                {"id": "cleaner", "kind": "text_clean", "config": {"text": "hello"}},
                {"id": "cond", "kind": "condition", "config": {"true_branch": "a", "false_branch": "b"}}
            ],
            "edges": [
                {"from": {"node": "cleaner", "port": "cleaned"}, "to": {"node": "cond", "port": "value"}}
            ]
        }))
        .unwrap();

        let hints = generate_hints(&graph, &reg);
        assert!(hints.iter().any(|h| h.severity == HintSeverity::Error
            && h.message.contains("Niezgodność typów")
            && h.message.contains("string")
            && h.message.contains("boolean")));
    }

    #[test]
    fn copilot_suggests_next_node_for_http_get() {
        let reg = test_registry();
        let graph: GraphSpec = serde_json::from_value(json!({
            "graph_version": "0.1",
            "nodes": [{"id": "fetch", "kind": "http_get", "config": {"url": "https://httpbin.org/json"}}],
            "edges": []
        }))
        .unwrap();

        let hints = generate_hints(&graph, &reg);
        assert!(hints.iter().any(|h| h.severity == HintSeverity::Info
            && h.suggestion
                .as_ref()
                .is_some_and(|s| s.contains("json_parse") || s.contains("log_output"))));
    }
}
