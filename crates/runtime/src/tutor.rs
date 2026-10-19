//! Deterministic, offline learning hints for Rust Lab.
//!
//! The backend returns stable hint codes and interpolation parameters. User-facing
//! copy is localized by the UI, keeping English as the canonical language while
//! allowing complete language switches without mixed messages.

use crate::plan;
use crate::registry::NodeRegistry;
use lab_node_sdk::{GraphSpec, PinType};
use seed_core::KnowledgeStore;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HintSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AdviceLevel {
    Nudge,
    Explain,
    Solution,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HintCode {
    CycleDetected,
    UnknownNodeKind,
    MissingRequiredInput,
    OrphanNode,
    UnusedOutput,
    MissingOutputPort,
    MissingInputPort,
    TypeMismatch,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum HintConcept {
    GraphStructure,
    NodeInterfaces,
    RequiredInputs,
    DataFlow,
    ExplicitTypes,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HintSuggestion {
    BreakCycle,
    ChooseKnownNode,
    ProvideFsReadPath,
    ProvideFsWritePath,
    ConnectFsWriteContent,
    ProvideHttpUrl,
    ProvideJsonInput,
    ProvideRequiredInput,
    ConnectOrRemoveNode,
    ConnectHttpBody,
    ConnectFileContent,
    ConvertJsonOutput,
    ConnectCleanedText,
    ConnectFilePath,
    CheckAvailableOutputs,
    CheckAvailableInputs,
    InsertStringToPath,
    InsertJsonToString,
    InsertJsonToBool,
    UseConverter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hint {
    pub id: String,
    pub code: HintCode,
    pub severity: HintSeverity,
    pub concept: HintConcept,
    pub level: AdviceLevel,
    pub attempts: u32,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub params: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<HintSuggestion>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TutorProgress {
    #[serde(default)]
    pub attempts: BTreeMap<String, u32>,
    #[serde(default)]
    pub active: BTreeMap<String, HintConcept>,
    #[serde(default)]
    pub learned: BTreeSet<HintConcept>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorOutcome {
    pub hints: Vec<Hint>,
    pub progress: TutorProgress,
    pub learned_now: Vec<HintConcept>,
}

fn concept_key(concept: HintConcept) -> &'static str {
    match concept {
        HintConcept::GraphStructure => "graph_structure",
        HintConcept::NodeInterfaces => "node_interfaces",
        HintConcept::RequiredInputs => "required_inputs",
        HintConcept::DataFlow => "data_flow",
        HintConcept::ExplicitTypes => "explicit_types",
    }
}

fn concept_vector(concept: HintConcept) -> Vec<f32> {
    let index = match concept {
        HintConcept::GraphStructure => 0,
        HintConcept::NodeInterfaces => 1,
        HintConcept::RequiredInputs => 2,
        HintConcept::DataFlow => 3,
        HintConcept::ExplicitTypes => 4,
    };
    let mut vector = vec![0.0; 5];
    vector[index] = 1.0;
    vector
}

fn concept_memory() -> &'static KnowledgeStore {
    static MEMORY: OnceLock<KnowledgeStore> = OnceLock::new();
    MEMORY.get_or_init(|| {
        let store = KnowledgeStore::new();
        for concept in [
            HintConcept::GraphStructure,
            HintConcept::NodeInterfaces,
            HintConcept::RequiredInputs,
            HintConcept::DataFlow,
            HintConcept::ExplicitTypes,
        ] {
            store.learn(
                concept_key(concept).to_string(),
                "rust_learning".to_string(),
                concept_vector(concept),
            );
        }
        store
    })
}

/// Grounds a tutor signal in seed-core's deterministic local concept memory.
fn ground_concept(signal: HintConcept) -> HintConcept {
    let (matches, _) = concept_memory().query_top_k(&concept_vector(signal), 1);
    match matches.first().map(|item| item.concept.as_str()) {
        Some("graph_structure") => HintConcept::GraphStructure,
        Some("node_interfaces") => HintConcept::NodeInterfaces,
        Some("required_inputs") => HintConcept::RequiredInputs,
        Some("data_flow") => HintConcept::DataFlow,
        Some("explicit_types") => HintConcept::ExplicitTypes,
        _ => signal,
    }
}

fn hint_id(code: &HintCode, node_id: Option<&str>, params: &BTreeMap<String, String>) -> String {
    let code = serde_json::to_value(code)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| "hint".to_string());
    let details = params
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("&");
    format!("{code}:{}:{details}", node_id.unwrap_or("graph"))
}

fn hint(
    code: HintCode,
    severity: HintSeverity,
    concept: HintConcept,
    node_id: Option<String>,
    suggestion: Option<HintSuggestion>,
    params: impl IntoIterator<Item = (&'static str, String)>,
) -> Hint {
    let params: BTreeMap<String, String> = params
        .into_iter()
        .map(|(key, value)| (key.to_string(), value))
        .collect();
    let concept = ground_concept(concept);
    Hint {
        id: hint_id(&code, node_id.as_deref(), &params),
        code,
        severity,
        concept,
        level: AdviceLevel::Nudge,
        attempts: 1,
        params,
        node_id,
        suggestion,
    }
}

pub fn personalize_hints(
    mut hints: Vec<Hint>,
    mut progress: TutorProgress,
    revision: String,
) -> TutorOutcome {
    let changed = progress.revision.as_deref() != Some(revision.as_str());
    let current_concepts: BTreeSet<HintConcept> = hints.iter().map(|hint| hint.concept).collect();

    let mut learned_now = progress
        .active
        .values()
        .copied()
        .filter(|concept| !current_concepts.contains(concept))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    learned_now.retain(|concept| progress.learned.insert(*concept));

    if changed {
        for hint in &hints {
            let attempts = progress.attempts.entry(hint.id.clone()).or_default();
            *attempts = attempts.saturating_add(1);
        }
    }

    for hint in &mut hints {
        hint.attempts = *progress.attempts.entry(hint.id.clone()).or_insert(1);
        hint.level = match hint.attempts {
            0 | 1 => AdviceLevel::Nudge,
            2 => AdviceLevel::Explain,
            _ => AdviceLevel::Solution,
        };
    }

    progress.active = hints
        .iter()
        .map(|hint| (hint.id.clone(), hint.concept))
        .collect();
    progress.revision = Some(revision);

    TutorOutcome {
        hints,
        progress,
        learned_now,
    }
}

pub fn tutor_outcome(
    graph: &GraphSpec,
    registry: &NodeRegistry,
    progress: TutorProgress,
    revision: String,
) -> TutorOutcome {
    personalize_hints(generate_hints(graph, registry), progress, revision)
}

pub fn generate_hints(graph: &GraphSpec, registry: &NodeRegistry) -> Vec<Hint> {
    let mut hints = Vec::new();
    let node_map: HashMap<&str, &lab_node_sdk::NodeInstance> = graph
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect();

    if let Err(error) = plan::execution_order(graph) {
        let detail = error.to_string();
        if detail.contains("Cycle detected") {
            hints.push(hint(
                HintCode::CycleDetected,
                HintSeverity::Error,
                HintConcept::GraphStructure,
                None,
                Some(HintSuggestion::BreakCycle),
                [("detail", detail)],
            ));
        }
    }

    let mut incoming_edges: HashMap<&str, Vec<&lab_node_sdk::Edge>> = HashMap::new();
    let mut outgoing_edges: HashMap<&str, Vec<&lab_node_sdk::Edge>> = HashMap::new();
    for edge in &graph.edges {
        incoming_edges.entry(&edge.to.node).or_default().push(edge);
        outgoing_edges
            .entry(&edge.from.node)
            .or_default()
            .push(edge);
    }

    let mut condition_targets: HashSet<&str> = HashSet::new();
    for node in &graph.nodes {
        if node.kind == "condition" {
            if let Some(target) = node
                .config
                .get("true_branch")
                .and_then(|value| value.as_str())
            {
                condition_targets.insert(target);
            }
            if let Some(target) = node
                .config
                .get("false_branch")
                .and_then(|value| value.as_str())
            {
                condition_targets.insert(target);
            }
        }
    }

    for node in &graph.nodes {
        let handler = match registry.get(&node.kind) {
            Some(handler) => handler,
            None => {
                hints.push(hint(
                    HintCode::UnknownNodeKind,
                    HintSeverity::Error,
                    HintConcept::NodeInterfaces,
                    Some(node.id.clone()),
                    Some(HintSuggestion::ChooseKnownNode),
                    [("node", node.id.clone()), ("kind", node.kind.clone())],
                ));
                continue;
            }
        };
        let spec = handler.spec();
        let config_map = node.config.as_object();

        for pin in spec.inputs.iter().filter(|pin| pin.required) {
            let pin_lower = pin.name.to_lowercase();
            let has_edge = incoming_edges.get(node.id.as_str()).is_some_and(|edges| {
                edges.iter().any(|edge| {
                    edge.to.port.to_lowercase() == pin_lower
                        || (pin.name == "path"
                            && matches!(
                                edge.to.port.to_lowercase().as_str(),
                                "file_path" | "filepath"
                            ))
                })
            });

            let has_config = config_map.is_some_and(|map| {
                let value = map
                    .get(&pin.name)
                    .or_else(|| map.get(&pin_lower))
                    .or_else(|| {
                        (pin.name == "path")
                            .then(|| map.get("file_path").or_else(|| map.get("filepath")))
                            .flatten()
                    });
                match value {
                    Some(serde_json::Value::Null) | None => false,
                    Some(serde_json::Value::String(text)) => !text.trim().is_empty(),
                    Some(_) => true,
                }
            });

            if !has_edge && !has_config {
                let suggestion = match (node.kind.as_str(), pin.name.as_str()) {
                    ("fs_read", "path") => HintSuggestion::ProvideFsReadPath,
                    ("fs_write", "path") => HintSuggestion::ProvideFsWritePath,
                    ("fs_write", "content") => HintSuggestion::ConnectFsWriteContent,
                    ("http_get", "url") => HintSuggestion::ProvideHttpUrl,
                    ("json_parse", "json_str") => HintSuggestion::ProvideJsonInput,
                    _ => HintSuggestion::ProvideRequiredInput,
                };
                hints.push(hint(
                    HintCode::MissingRequiredInput,
                    HintSeverity::Warning,
                    HintConcept::RequiredInputs,
                    Some(node.id.clone()),
                    Some(suggestion),
                    [
                        ("node", node.id.clone()),
                        ("kind", node.kind.clone()),
                        ("pin", pin.name.clone()),
                    ],
                ));
            }
        }

        let incoming_count = incoming_edges.get(node.id.as_str()).map_or(0, Vec::len);
        let outgoing_count = outgoing_edges.get(node.id.as_str()).map_or(0, Vec::len);
        let is_condition_target = condition_targets.contains(node.id.as_str());
        let has_after = !node.after.is_empty();
        let is_after_target = graph.nodes.iter().any(|candidate| {
            candidate
                .after
                .iter()
                .any(|dependency| dependency == &node.id)
        });

        if graph.nodes.len() > 1
            && incoming_count == 0
            && outgoing_count == 0
            && !is_condition_target
            && !has_after
            && !is_after_target
        {
            hints.push(hint(
                HintCode::OrphanNode,
                HintSeverity::Warning,
                HintConcept::DataFlow,
                Some(node.id.clone()),
                Some(HintSuggestion::ConnectOrRemoveNode),
                [("node", node.id.clone()), ("kind", node.kind.clone())],
            ));
        }

        if outgoing_count == 0 && !spec.outputs.is_empty() {
            let suggestion = match node.kind.as_str() {
                "http_get" => Some(HintSuggestion::ConnectHttpBody),
                "fs_read" => Some(HintSuggestion::ConnectFileContent),
                "json_parse" => Some(HintSuggestion::ConvertJsonOutput),
                "text_clean" => Some(HintSuggestion::ConnectCleanedText),
                "string_to_path" => Some(HintSuggestion::ConnectFilePath),
                _ => None,
            };
            if let Some(suggestion) = suggestion {
                hints.push(hint(
                    HintCode::UnusedOutput,
                    HintSeverity::Info,
                    HintConcept::DataFlow,
                    Some(node.id.clone()),
                    Some(suggestion),
                    [("node", node.id.clone()), ("kind", node.kind.clone())],
                ));
            }
        }
    }

    for edge in &graph.edges {
        if let (Some(from_node), Some(to_node)) = (
            node_map.get(edge.from.node.as_str()),
            node_map.get(edge.to.node.as_str()),
        ) {
            if let (Some(from_handler), Some(to_handler)) =
                (registry.get(&from_node.kind), registry.get(&to_node.kind))
            {
                let from_spec = from_handler.spec();
                let to_spec = to_handler.spec();
                let from_pin = from_spec
                    .outputs
                    .iter()
                    .find(|pin| pin.name == edge.from.port);
                let to_pin = to_spec.inputs.iter().find(|pin| pin.name == edge.to.port);

                match (from_pin, to_pin) {
                    (None, _) => hints.push(hint(
                        HintCode::MissingOutputPort,
                        HintSeverity::Error,
                        HintConcept::NodeInterfaces,
                        Some(edge.from.node.clone()),
                        Some(HintSuggestion::CheckAvailableOutputs),
                        [
                            ("node", edge.from.node.clone()),
                            ("kind", from_node.kind.clone()),
                            ("port", edge.from.port.clone()),
                        ],
                    )),
                    (_, None) => hints.push(hint(
                        HintCode::MissingInputPort,
                        HintSeverity::Error,
                        HintConcept::NodeInterfaces,
                        Some(edge.to.node.clone()),
                        Some(HintSuggestion::CheckAvailableInputs),
                        [
                            ("node", edge.to.node.clone()),
                            ("kind", to_node.kind.clone()),
                            ("port", edge.to.port.clone()),
                        ],
                    )),
                    (Some(source), Some(target))
                        if !source.pin_type.is_compatible_with(&target.pin_type) =>
                    {
                        let suggestion = match (&source.pin_type, &target.pin_type) {
                            (PinType::String, PinType::FilePath) => {
                                HintSuggestion::InsertStringToPath
                            }
                            (PinType::Json, PinType::String) => HintSuggestion::InsertJsonToString,
                            (PinType::Json, PinType::Boolean) => HintSuggestion::InsertJsonToBool,
                            _ => HintSuggestion::UseConverter,
                        };
                        hints.push(hint(
                            HintCode::TypeMismatch,
                            HintSeverity::Error,
                            HintConcept::ExplicitTypes,
                            Some(edge.to.node.clone()),
                            Some(suggestion),
                            [
                                ("from_node", edge.from.node.clone()),
                                ("from_port", edge.from.port.clone()),
                                ("from_type", source.pin_type.to_string()),
                                ("to_node", edge.to.node.clone()),
                                ("to_port", edge.to.port.clone()),
                                ("to_type", target.pin_type.to_string()),
                            ],
                        ));
                    }
                    (Some(_), Some(_)) => {}
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
        let mut registry = NodeRegistry::new();
        nodes::register_all(&mut registry);
        registry
    }

    #[test]
    fn tutor_detects_missing_required_input() {
        let graph: GraphSpec = serde_json::from_value(json!({
            "graph_version": "0.1",
            "nodes": [{"id": "reader", "kind": "fs_read", "config": {}}],
            "edges": []
        }))
        .unwrap();

        let hints = generate_hints(&graph, &test_registry());
        assert!(hints.iter().any(|hint| {
            hint.code == HintCode::MissingRequiredInput
                && hint.params.get("pin").is_some_and(|pin| pin == "path")
        }));
    }

    #[test]
    fn tutor_detects_orphan_node() {
        let graph: GraphSpec = serde_json::from_value(json!({
            "graph_version": "0.1",
            "nodes": [
                {"id": "cleaner", "kind": "text_clean", "config": {"text": "hello"}},
                {"id": "orphan", "kind": "log_output", "config": {"text": "lonely"}}
            ],
            "edges": []
        }))
        .unwrap();

        let hints = generate_hints(&graph, &test_registry());
        assert!(hints.iter().any(|hint| {
            hint.code == HintCode::OrphanNode && hint.node_id.as_deref() == Some("orphan")
        }));
    }

    #[test]
    fn tutor_detects_type_mismatch() {
        let graph: GraphSpec = serde_json::from_value(json!({
            "graph_version": "0.1",
            "nodes": [
                {"id": "reader", "kind": "fs_read", "config": {"path": "data/a.txt"}},
                {"id": "writer", "kind": "fs_write", "config": {"content": "x"}}
            ],
            "edges": [{"from": {"node": "reader", "port": "content"}, "to": {"node": "writer", "port": "path"}}]
        }))
        .unwrap();

        let hints = generate_hints(&graph, &test_registry());
        assert!(hints.iter().any(|hint| {
            hint.code == HintCode::TypeMismatch
                && hint.suggestion == Some(HintSuggestion::InsertStringToPath)
        }));
    }

    #[test]
    fn tutor_suggests_next_node_for_http_get() {
        let graph: GraphSpec = serde_json::from_value(json!({
            "graph_version": "0.1",
            "nodes": [{"id": "fetch", "kind": "http_get", "config": {"url": "https://httpbin.org/json"}}],
            "edges": []
        }))
        .unwrap();

        let hints = generate_hints(&graph, &test_registry());
        assert!(hints.iter().any(|hint| {
            hint.code == HintCode::UnusedOutput
                && hint.suggestion == Some(HintSuggestion::ConnectHttpBody)
        }));
    }

    #[test]
    fn serialized_hints_contain_codes_not_localized_copy() {
        let graph: GraphSpec = serde_json::from_value(json!({
            "graph_version": "0.1",
            "nodes": [{"id": "reader", "kind": "fs_read", "config": {}}],
            "edges": []
        }))
        .unwrap();

        let serialized = serde_json::to_value(generate_hints(&graph, &test_registry())).unwrap();
        let first = serialized.as_array().unwrap().first().unwrap();
        assert!(first.get("code").is_some());
        assert!(first.get("message").is_none());
    }

    #[test]
    fn repeated_issue_progresses_from_nudge_to_solution() {
        let graph: GraphSpec = serde_json::from_value(json!({
            "graph_version": "0.1",
            "nodes": [{"id": "reader", "kind": "fs_read", "config": {}}],
            "edges": []
        }))
        .unwrap();
        let registry = test_registry();

        let first = tutor_outcome(
            &graph,
            &registry,
            TutorProgress::default(),
            "revision-1".into(),
        );
        assert!(first
            .hints
            .iter()
            .all(|hint| hint.level == AdviceLevel::Nudge && hint.attempts == 1));

        let second = tutor_outcome(&graph, &registry, first.progress, "revision-2".into());
        assert!(second
            .hints
            .iter()
            .all(|hint| hint.level == AdviceLevel::Explain && hint.attempts == 2));

        let repeated_refresh =
            tutor_outcome(&graph, &registry, second.progress, "revision-2".into());
        assert!(repeated_refresh
            .hints
            .iter()
            .all(|hint| hint.level == AdviceLevel::Explain && hint.attempts == 2));

        let third = tutor_outcome(
            &graph,
            &registry,
            repeated_refresh.progress,
            "revision-3".into(),
        );
        assert!(third
            .hints
            .iter()
            .all(|hint| hint.level == AdviceLevel::Solution && hint.attempts == 3));
    }

    #[test]
    fn resolving_an_issue_marks_its_concept_as_learned() {
        let broken: GraphSpec = serde_json::from_value(json!({
            "graph_version": "0.1",
            "nodes": [{"id": "reader", "kind": "fs_read", "config": {}}],
            "edges": []
        }))
        .unwrap();
        let fixed: GraphSpec = serde_json::from_value(json!({
            "graph_version": "0.1",
            "nodes": [{"id": "reader", "kind": "fs_read", "config": {"path": "data/input.txt"}}],
            "edges": []
        }))
        .unwrap();
        let registry = test_registry();

        let first = tutor_outcome(
            &broken,
            &registry,
            TutorProgress::default(),
            "broken".into(),
        );
        let resolved = tutor_outcome(&fixed, &registry, first.progress, "fixed".into());

        assert!(resolved.learned_now.contains(&HintConcept::RequiredInputs));
        assert!(resolved
            .progress
            .learned
            .contains(&HintConcept::RequiredInputs));
    }
}
