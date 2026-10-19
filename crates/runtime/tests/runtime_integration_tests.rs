use std::fs;
use std::path::PathBuf;

use runtime::{nodes::register_all, Executor, NodeRegistry, Validator};
use serde_json::json;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    register_all(&mut reg);
    reg
}

fn graph(value: serde_json::Value) -> lab_node_sdk::GraphSpec {
    serde_json::from_value(value).unwrap()
}

#[test]
fn validator_accepts_valid_pipeline_graph() {
    let g = graph(json!({
        "graph_version": "0.1",
        "nodes": [
            {"id": "fetch", "kind": "http_get", "config": {"url": "https://example.com"}},
            {"id": "parse", "kind": "json_parse", "config": {"json_str": "{\"message\":\"hello\"}", "path": "message"}},
            {"id": "clean", "kind": "text_clean", "config": {"text": " sample ", "trim": true, "to_upper": true}},
            {"id": "print", "kind": "log_output", "config": {}}
        ],
        "edges": [
            {"from": {"node": "fetch", "port": "body"}, "to": {"node": "parse", "port": "json_str"}},
            {"from": {"node": "parse", "port": "value"}, "to": {"node": "clean", "port": "text"}},
            {"from": {"node": "clean", "port": "cleaned"}, "to": {"node": "print", "port": "text"}}
        ]
    }));

    let validator = Validator::new(&registry());
    assert!(validator.validate(&g).is_ok());
}

#[test]
fn validator_rejects_duplicate_node_ids() {
    let g = graph(json!({
        "graph_version": "0.1",
        "nodes": [
            {"id": "dup", "kind": "delay", "config": {"duration": "1ms"}},
            {"id": "dup", "kind": "log_output", "config": {"text": "same id"}}
        ],
        "edges": []
    }));

    let err = Validator::new(&registry()).validate(&g).unwrap_err().to_string();
    assert!(err.contains("Duplicate node ID"), "{err}");
}

#[tokio::test]
async fn executor_runs_expected_branch_and_prunes_other_path() {
    let g = graph(json!({
        "graph_version": "0.1",
        "nodes": [
            {"id": "if", "kind": "condition", "config": {"value": true, "true_branch": "yes", "false_branch": "no"}},
            {"id": "yes", "kind": "log_output", "config": {"text": "allowed"}},
            {"id": "no", "kind": "delay", "config": {"duration": "1ms"}},
            {"id": "merge", "kind": "log_output", "config": {"text": "done"}}
        ],
        "edges": [
            {"from": {"node": "if", "port": "__next"}, "to": {"node": "yes", "port": "text"}}
        ]
    }));

    let report = Executor::new(registry()).execute(g).await.unwrap();
    assert!(report.executed.contains(&"if".to_string()));
    assert!(report.executed.contains(&"yes".to_string()));
    assert!(report.executed.contains(&"merge".to_string()));
    assert!(report.skipped.contains(&"no".to_string()));
}

#[test]
fn export_project_creates_complete_working_structure() {
    let g = graph(json!({
        "graph_version": "0.1",
        "nodes": [
            {"id": "parse", "kind": "json_parse", "config": {"json_str": "{\"msg\":\"hello world\"}", "path": "msg"}},
            {"id": "to_text", "kind": "json_to_string", "config": {}},
            {"id": "clean", "kind": "text_clean", "config": {"to_upper": true}},
            {"id": "print", "kind": "log_output", "config": {}}
        ],
        "edges": [
            {"from": {"node": "parse", "port": "value"}, "to": {"node": "to_text", "port": "value"}},
            {"from": {"node": "to_text", "port": "text"}, "to": {"node": "clean", "port": "text"}},
            {"from": {"node": "clean", "port": "cleaned"}, "to": {"node": "print", "port": "text"}}
        ]
    }));

    let temp_dir = std::env::temp_dir().join("rust_lab_runtime_integration_test");
    let _ = fs::remove_dir_all(&temp_dir);

    runtime::export_project(&g, &temp_dir, "rust_lab_runtime_integration_test").unwrap();

    assert!(temp_dir.join("Cargo.toml").exists());
    assert!(temp_dir.join("src").is_dir());
    assert!(temp_dir.join("src/main.rs").exists());
    assert!(temp_dir.join("src/nodes.rs").exists());

    let main_source = std::fs::read_to_string(temp_dir.join("src/main.rs")).unwrap();
    assert!(main_source.contains("nodes::json_parse"));
    assert!(main_source.contains("nodes::json_to_string"));
    assert!(main_source.contains("nodes::text_clean"));
    assert!(main_source.contains("nodes::log_output"));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn runtime_supports_branch_validation_and_execution_order() {
    let g = graph(json!({
        "graph_version": "0.1",
        "nodes": [
            {"id": "if", "kind": "condition", "config": {"value": true, "true_branch": "a", "false_branch": "b"}},
            {"id": "a", "kind": "log_output", "config": {"text": "A"}},
            {"id": "b", "kind": "log_output", "config": {"text": "B"}},
            {"id": "merge", "kind": "log_output", "config": {"text": "merge"}}
        ],
        "edges": []
    }));

    let validator = Validator::new(&registry());
    assert!(validator.validate(&g).is_ok());

    let order = runtime::plan::execution_order(&g).unwrap();
    assert!(order[0] == "if");
    assert!(order.contains(&"a".to_string()));
    assert!(order.contains(&"b".to_string()));
    assert!(order.contains(&"merge".to_string()));
}
