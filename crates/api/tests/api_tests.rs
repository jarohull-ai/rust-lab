use api::{app, AppState, HealthResponse, ValidateResponse};
use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
};
use http_body_util::BodyExt;
use lab_node_sdk::NodeSpec;
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn test_health_endpoint() {
    let router = app(AppState::default());

    let req = Request::builder()
        .uri("/api/health")
        .method(Method::GET)
        .body(Body::empty())
        .unwrap();

    let res = router.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let health: HealthResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(health.status, "ok");
    assert_eq!(health.service, "rust-lab-api");
}

#[tokio::test]
async fn test_nodes_endpoint_returns_all_specs() {
    let router = app(AppState::default());

    let req = Request::builder()
        .uri("/api/nodes")
        .method(Method::GET)
        .body(Body::empty())
        .unwrap();

    let res = router.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let specs: Vec<NodeSpec> = serde_json::from_slice(&body).unwrap();

    // Check that we have 11 registered nodes
    assert_eq!(specs.len(), 11);
    let names: Vec<String> = specs.into_iter().map(|s| s.name).collect();
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
async fn test_validate_endpoint_valid_and_invalid() {
    let router = app(AppState::default());

    // Valid graph
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

    let req = Request::builder()
        .uri("/api/validate")
        .method(Method::POST)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&valid_graph).unwrap()))
        .unwrap();

    let res = router.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let val_res: ValidateResponse = serde_json::from_slice(&body).unwrap();
    assert!(val_res.valid);
    assert!(val_res.errors.is_empty());
    assert_eq!(val_res.execution_order, vec!["cleaner", "printer"]);

    // Invalid graph: type mismatch (String -> Boolean)
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

    let req_bad = Request::builder()
        .uri("/api/validate")
        .method(Method::POST)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&invalid_graph).unwrap()))
        .unwrap();

    let res_bad = router.oneshot(req_bad).await.unwrap();
    assert_eq!(res_bad.status(), StatusCode::OK);
    let body_bad = res_bad.into_body().collect().await.unwrap().to_bytes();
    let val_res_bad: ValidateResponse = serde_json::from_slice(&body_bad).unwrap();
    assert!(!val_res_bad.valid);
    assert!(!val_res_bad.errors.is_empty());
    assert!(val_res_bad.errors[0].contains("Type mismatch"));
}

#[tokio::test]
async fn test_execute_endpoint() {
    let router = app(AppState::default());

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

    let req = Request::builder()
        .uri("/api/execute")
        .method(Method::POST)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&graph).unwrap()))
        .unwrap();

    let res = router.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let report: runtime::ExecutionReport = serde_json::from_slice(&body).unwrap();
    assert_eq!(report.executed, vec!["cleaner", "printer"]);
    assert_eq!(report.skipped.len(), 0);
    assert_eq!(
        report.outputs["cleaner"]["cleaned"],
        serde_json::json!("hello api")
    );
}

#[tokio::test]
async fn test_export_endpoint() {
    let router = app(AppState::default());

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

    let req = Request::builder()
        .uri("/api/export")
        .method(Method::POST)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&export_req).unwrap()))
        .unwrap();

    let res = router.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let export_res: api::ExportResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(export_res.project_name, "weather_service");
    assert!(export_res.cargo_toml.contains("name = \"weather_service\""));
    assert!(export_res.main_rs.contains("nodes::text_clean"));
    assert!(export_res.nodes_rs.contains("pub fn text_clean"));
}

#[tokio::test]
async fn test_export_disk_endpoint() {
    let router = app(AppState::default());
    let temp_out = std::env::temp_dir().join("test_rust_lab_api_export");
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

    let req = Request::builder()
        .uri("/api/export/disk")
        .method(Method::POST)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&disk_req).unwrap()))
        .unwrap();

    let res = router.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(temp_out.join("Cargo.toml").exists());
    assert!(temp_out.join("src/main.rs").exists());
    assert!(temp_out.join("src/nodes.rs").exists());

    let _ = std::fs::remove_dir_all(&temp_out);
}
