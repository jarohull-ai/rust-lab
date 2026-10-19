use api::{app, AppState, ExportResponse, HealthResponse, ValidateResponse};
use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
};
use http_body_util::BodyExt;
use lab_node_sdk::NodeSpec;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tower::ServiceExt;

fn build_router() -> axum::Router {
    app(AppState::default())
}

async fn get_json<T: DeserializeOwned>(router: axum::Router, path: &str) -> T {
    let req = Request::builder()
        .uri(path)
        .method(Method::GET)
        .body(Body::empty())
        .unwrap();

    let res = router.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK, "GET {path} should return 200 OK");

    let body = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body).unwrap()
}

async fn post_json<T: DeserializeOwned>(router: axum::Router, path: &str, payload: Value) -> T {
    let req = Request::builder()
        .uri(path)
        .method(Method::POST)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let res = router.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK, "POST {path} should return 200 OK");

    let body = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test]
async fn health_endpoint_reports_service_status() {
    let router = build_router();
    let health: HealthResponse = get_json(router, "/api/health").await;

    assert_eq!(health.status, "ok");
    assert_eq!(health.service, "rust-lab-api");
    assert_eq!(health.version, "0.1.0");
}

#[tokio::test]
async fn nodes_endpoint_lists_expected_registry() {
    let router = build_router();
    let specs: Vec<NodeSpec> = get_json(router, "/api/nodes").await;

    assert!(!specs.is_empty(), "node catalog should not be empty");
    let names: Vec<String> = specs.iter().map(|spec| spec.name.clone()).collect();

    assert!(names.contains(&"condition".to_string()));
    assert!(names.contains(&"delay".to_string()));
    assert!(names.contains(&"http_get".to_string()));
    assert!(names.contains(&"json_parse".to_string()));
    assert!(names.contains(&"log_output".to_string()));
    assert!(names.contains(&"string_to_path".to_string()));
    assert!(names.contains(&"json_to_string".to_string()));
    assert!(names.contains(&"json_to_bool".to_string()));
}

#[tokio::test]
async fn validate_endpoint_accepts_valid_graph_and_rejects_invalid_graph() {
    let valid_graph = json!({
        "graph_version": "0.1",
        "nodes": [
            { "id": "cleaner", "kind": "text_clean", "config": { "text": "   HELLO WORLD  " } },
            { "id": "printer", "kind": "log_output", "config": {} }
        ],
        "edges": [
            { "from": { "node": "cleaner", "port": "cleaned" }, "to": { "node": "printer", "port": "text" } }
        ]
    });

    let router = build_router();
    let valid: ValidateResponse = post_json(router.clone(), "/api/validate", valid_graph).await;
    assert!(valid.valid);
    assert!(valid.errors.is_empty());
    assert_eq!(valid.execution_order, vec!["cleaner", "printer"]);

    let invalid_graph = json!({
        "graph_version": "0.1",
        "nodes": [
            { "id": "cleaner", "kind": "text_clean", "config": { "text": "hello" } },
            { "id": "cond", "kind": "condition", "config": { "true_branch": "a", "false_branch": "b" } }
        ],
        "edges": [
            { "from": { "node": "cleaner", "port": "cleaned" }, "to": { "node": "cond", "port": "value" } }
        ]
    });

    let invalid: ValidateResponse = post_json(router, "/api/validate", invalid_graph).await;
    assert!(!invalid.valid);
    assert!(!invalid.errors.is_empty());
    assert!(invalid.errors[0].contains("Type mismatch"));
}

#[tokio::test]
async fn execute_endpoint_returns_effective_execution_report() {
    let router = build_router();
    let graph = json!({
        "graph_version": "0.1",
        "nodes": [
            { "id": "cleaner", "kind": "text_clean", "config": { "text": "   HELLO API   ", "to_lower": true } },
            { "id": "printer", "kind": "log_output", "config": {} }
        ],
        "edges": [
            { "from": { "node": "cleaner", "port": "cleaned" }, "to": { "node": "printer", "port": "text" } }
        ]
    });

    let report: runtime::ExecutionReport = post_json(router, "/api/execute", graph).await;
    assert_eq!(report.executed, vec!["cleaner", "printer"]);
    assert!(report.skipped.is_empty());
    assert_eq!(report.outputs["cleaner"]["cleaned"], serde_json::json!("hello api"));
}

#[tokio::test]
async fn export_endpoint_generates_project_source_files() {
    let router = build_router();
    let export_req = json!({
        "project_name": "weather_service",
        "graph": {
            "graph_version": "0.1",
            "nodes": [
                { "id": "cleaner", "kind": "text_clean", "config": { "text": "Sunny", "to_upper": true } },
                { "id": "logger", "kind": "log_output", "config": {} }
            ],
            "edges": [
                { "from": { "node": "cleaner", "port": "cleaned" }, "to": { "node": "logger", "port": "text" } }
            ]
        }
    });

    let export_res: ExportResponse = post_json(router, "/api/export", export_req).await;
    assert_eq!(export_res.project_name, "weather_service");
    assert!(export_res.cargo_toml.contains("name = \"weather_service\""));
    assert!(export_res.main_rs.contains("nodes::text_clean"));
    assert!(export_res.nodes_rs.contains("pub fn text_clean"));
}

#[tokio::test]
async fn export_disk_endpoint_writes_generated_project_to_temp_dir() {
    let router = build_router();
    let temp_out = std::env::temp_dir().join("test_rust_lab_api_export_professional");
    let _ = std::fs::remove_dir_all(&temp_out);

    let disk_req = json!({
        "project_name": "disk_app",
        "output_dir": temp_out.to_str().unwrap(),
        "graph": {
            "graph_version": "0.1",
            "nodes": [
                { "id": "cleaner", "kind": "text_clean", "config": { "text": "Disk App" } },
                { "id": "logger", "kind": "log_output", "config": {} }
            ],
            "edges": [
                { "from": { "node": "cleaner", "port": "cleaned" }, "to": { "node": "logger", "port": "text" } }
            ]
        }
    });

    let response: api::ExportDiskResponse = post_json(router, "/api/export/disk", disk_req).await;
    assert!(response.success);
    assert!(temp_out.join("Cargo.toml").exists());
    assert!(temp_out.join("src/main.rs").exists());
    assert!(temp_out.join("src/nodes.rs").exists());
    assert!(response.output_dir.starts_with(std::env::temp_dir().to_string_lossy().as_ref()));

    let _ = std::fs::remove_dir_all(&temp_out);
}
