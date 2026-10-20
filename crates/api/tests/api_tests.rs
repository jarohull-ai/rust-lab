use api::{app, AppState, CompileResponse, ExportResponse, HealthResponse, ValidateResponse};
use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
};
use http_body_util::BodyExt;
use lab_node_sdk::NodeSpec;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tower::ServiceExt;

static LAB_TEST_SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn build_router() -> axum::Router {
    app(AppState::default())
}

fn build_lab_router() -> (axum::Router, std::path::PathBuf) {
    let sequence = LAB_TEST_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let root = std::path::PathBuf::from("target/rust-lab-api-tests")
        .join(format!("{}-{sequence}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let state = AppState {
        registry: std::sync::Arc::new(runtime::NodeRegistry::default()),
        workspace_root: std::sync::Arc::new(root.clone()),
    };
    (app(state), root)
}

async fn get_json<T: DeserializeOwned>(router: axum::Router, path: &str) -> T {
    let req = Request::builder()
        .uri(path)
        .method(Method::GET)
        .body(Body::empty())
        .unwrap();

    let res = router.oneshot(req).await.unwrap();
    assert_eq!(
        res.status(),
        StatusCode::OK,
        "GET {path} should return 200 OK"
    );

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
    assert_eq!(
        res.status(),
        StatusCode::OK,
        "POST {path} should return 200 OK"
    );

    let body = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test]
async fn health_endpoint_reports_service_status() {
    let router = build_router();
    let health: HealthResponse = get_json(router, "/api/health").await;

    assert_eq!(health.status, "ok");
    assert_eq!(health.service, "rust-lab-api");
    assert_eq!(health.version, env!("CARGO_PKG_VERSION"));
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
    assert_eq!(
        report.outputs["cleaner"]["cleaned"],
        serde_json::json!("hello api")
    );
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
async fn compile_endpoint_checks_generated_rust_offline() {
    let router = build_router();
    let request = json!({
        "project_name": "compile_check",
        "graph": {
            "graph_version": "0.1",
            "nodes": [
                { "id": "cleaner", "kind": "text_clean", "config": { "text": "Compile me" } },
                { "id": "logger", "kind": "log_output", "config": {} }
            ],
            "edges": [
                { "from": { "node": "cleaner", "port": "cleaned" }, "to": { "node": "logger", "port": "text" } }
            ]
        }
    });

    let response: CompileResponse = post_json(router, "/api/compile", request).await;
    assert!(response.success, "cargo check failed: {}", response.stderr);
}

#[tokio::test]
async fn export_disk_endpoint_writes_generated_project_to_temp_dir() {
    let router = build_router();
    let temp_out = std::path::PathBuf::from("target/exports/test_rust_lab_api_export");
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
    assert!(response.output_dir.starts_with("target/exports/"));

    let _ = std::fs::remove_dir_all(&temp_out);
}

#[tokio::test]
async fn export_disk_endpoint_rejects_paths_outside_export_workspace() {
    let router = build_router();
    let disk_req = json!({
        "project_name": "blocked_app",
        "output_dir": "/tmp/rust-lab-must-not-write-here",
        "graph": {
            "graph_version": "0.1",
            "nodes": [{ "id": "logger", "kind": "log_output", "config": { "text": "safe" } }],
            "edges": []
        }
    });
    let req = Request::builder()
        .uri("/api/export/disk")
        .method(Method::POST)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&disk_req).unwrap()))
        .unwrap();

    let response = router.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn tutor_hints_endpoint_returns_structured_contextual_hints() {
    let router = app(AppState::default());
    let graph = json!({
        "graph_version": "0.1",
        "nodes": [
            {"id": "fetch", "kind": "http_get", "config": {}}
        ],
        "edges": []
    });

    let req = Request::builder()
        .uri("/api/tutor/hints")
        .method(Method::POST)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&json!({
                "graph": graph,
                "progress": {},
                "revision": "first-edit"
            }))
            .unwrap(),
        ))
        .unwrap();

    let res = router.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let hints_resp: api::HintsResponse = serde_json::from_slice(&body).unwrap();
    assert!(!hints_resp.hints.is_empty());
    assert!(hints_resp.hints.iter().any(|hint| {
        hint.code == runtime::HintCode::MissingRequiredInput
            && hint.params.get("pin").is_some_and(|pin| pin == "url")
    }));
    assert!(hints_resp
        .hints
        .iter()
        .all(|hint| hint.level == runtime::AdviceLevel::Nudge));
    assert_eq!(hints_resp.progress.revision.as_deref(), Some("first-edit"));
}

#[tokio::test]
async fn runtime_rejects_http_get_to_evil_domain() {
    let graph = json!({
        "graph_version": "0.1",
        "nodes": [
            {"id": "fetch", "kind": "http_get", "params": {"url": "https://evil.example.com/exfil"}}
        ],
        "edges": []
    });
    let mut registry = runtime::NodeRegistry::new();
    runtime::nodes::register_all(&mut registry);
    let policy = runtime::ExecutionPolicy::default_safe();
    let result = runtime::executor::execute_graph(&graph, &registry, &policy).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Execution policy denied"), "{err}");
    assert!(err.contains("http_get.domain_allowlist"), "{err}");
}

#[tokio::test]
async fn runtime_allows_http_get_to_httpbin() {
    let graph = json!({
        "graph_version": "0.1",
        "nodes": [
            {"id": "fetch", "kind": "http_get", "params": {"url": "https://httpbin.org/get"}}
        ],
        "edges": []
    });
    let mut registry = runtime::NodeRegistry::new();
    runtime::nodes::register_all(&mut registry);
    let policy = runtime::ExecutionPolicy::default_safe();
    let spec: lab_node_sdk::GraphSpec = serde_json::from_value(graph).unwrap();
    let executor = runtime::Executor::new(registry);
    let res = executor.execute_with_policy(spec, &policy).await;
    if let Err(e) = res {
        let err_str = e.to_string();
        assert!(
            !err_str.contains("Execution policy denied"),
            "The execution policy should allow the configured domain: {err_str}"
        );
    }
}

#[tokio::test]
async fn lab_workspace_creates_reads_and_updates_a_cargo_project() {
    let (router, root) = build_lab_router();
    let project: api::LabProject = post_json(
        router.clone(),
        "/api/lab/projects",
        json!({ "name": "First Prototype" }),
    )
    .await;
    assert!(project.id.starts_with("first-prototype-"));

    let tree: api::WorkspaceTree = get_json(
        router.clone(),
        &format!("/api/lab/projects/{}/tree", project.id),
    )
    .await;
    assert!(tree.entries.iter().any(|entry| entry.path == "Cargo.toml"));
    assert!(tree.entries.iter().any(|entry| entry.path == "src/main.rs"));

    let file: api::LabFile = get_json(
        router.clone(),
        &format!("/api/lab/file?project_id={}&path=src%2Fmain.rs", project.id),
    )
    .await;
    assert!(file.content.contains("Hello from Rust Lab"));

    let saved: api::LabFile = post_json_with_method(
        router.clone(),
        "/api/lab/file",
        Method::PUT,
        json!({
            "project_id": project.id,
            "path": "src/main.rs",
            "content": "fn main() { println!(\"edited\"); }\n"
        }),
    )
    .await;
    assert!(saved.content.contains("edited"));
    let _ = std::fs::remove_dir_all(root);
}

async fn post_json_with_method<T: DeserializeOwned>(
    router: axum::Router,
    path: &str,
    method: Method,
    payload: Value,
) -> T {
    let req = Request::builder()
        .uri(path)
        .method(method)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();
    let response = router.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test]
async fn lab_workspace_rejects_unsafe_imports_paths_and_unapproved_execution() {
    let (router, root) = build_lab_router();
    let request = Request::builder()
        .uri("/api/lab/import/github")
        .method(Method::POST)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&json!({ "url": "https://evil.example/project" })).unwrap(),
        ))
        .unwrap();
    assert_eq!(
        router.clone().oneshot(request).await.unwrap().status(),
        StatusCode::BAD_REQUEST
    );

    let project: api::LabProject = post_json(
        router.clone(),
        "/api/lab/projects",
        json!({ "name": "Guarded" }),
    )
    .await;
    let request = Request::builder()
        .uri(format!(
            "/api/lab/file?project_id={}&path=..%2Fproject.json",
            project.id
        ))
        .method(Method::GET)
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        router.clone().oneshot(request).await.unwrap().status(),
        StatusCode::BAD_REQUEST
    );

    let request = Request::builder()
        .uri("/api/lab/cargo")
        .method(Method::POST)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&json!({
                "project_id": project.id,
                "action": "run",
                "approved": false
            }))
            .unwrap(),
        ))
        .unwrap();
    assert_eq!(
        router.oneshot(request).await.unwrap().status(),
        StatusCode::FORBIDDEN
    );
    let _ = std::fs::remove_dir_all(root);
}
