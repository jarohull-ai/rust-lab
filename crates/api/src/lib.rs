use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use lab_node_sdk::{GraphSpec, NodeSpec};
use runtime::{
    codegen, plan, ExecutionReport, Executor, NodeRegistry, Validator,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<NodeRegistry>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            registry: Arc::new(NodeRegistry::default()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub service: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateResponse {
    pub valid: bool,
    pub errors: Vec<String>,
    pub execution_order: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportRequest {
    pub graph: GraphSpec,
    #[serde(default = "default_project_name")]
    pub project_name: String,
}

fn default_project_name() -> String {
    "rust_lab_app".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportResponse {
    pub project_name: String,
    pub cargo_toml: String,
    pub main_rs: String,
    pub nodes_rs: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportDiskRequest {
    pub graph: GraphSpec,
    pub output_dir: String,
    #[serde(default = "default_project_name")]
    pub project_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportDiskResponse {
    pub success: bool,
    pub output_dir: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HintsResponse {
    pub hints: Vec<runtime::copilot::Hint>,
}

pub fn app(state: AppState) -> Router {
    let mut router = Router::new()
        .route("/api/health", get(health_handler))
        .route("/api/nodes", get(nodes_handler))
        .route("/api/validate", post(validate_handler))
        .route("/api/execute", post(execute_handler))
        .route("/api/export", post(export_handler))
        .route("/api/export/disk", post(export_disk_handler))
        .route("/api/copilot/hints", post(copilot_hints_handler));

    if Path::new("ui/dist").exists() {
        router = router.fallback_service(ServeDir::new("ui/dist"));
    }

    router
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: "0.1.0".to_string(),
        service: "rust-lab-api".to_string(),
    })
}

async fn nodes_handler(State(state): State<AppState>) -> Json<Vec<NodeSpec>> {
    let specs = state.registry.all_specs();
    Json(specs)
}

async fn validate_handler(
    State(state): State<AppState>,
    Json(graph): Json<GraphSpec>,
) -> Json<ValidateResponse> {
    let validator = Validator::new(&state.registry);
    match validator.validate(&graph) {
        Ok(()) => {
            let order = plan::execution_order(&graph).unwrap_or_default();
            Json(ValidateResponse {
                valid: true,
                errors: Vec::new(),
                execution_order: order,
            })
        }
        Err(err) => Json(ValidateResponse {
            valid: false,
            errors: vec![err.to_string()],
            execution_order: Vec::new(),
        }),
    }
}

async fn execute_handler(
    State(state): State<AppState>,
    Json(graph): Json<GraphSpec>,
) -> Result<Json<ExecutionReport>, (StatusCode, Json<ErrorResponse>)> {
    // 1. Validate first
    let validator = Validator::new(&state.registry);
    if let Err(err) = validator.validate(&graph) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Validation failed: {err}"),
            }),
        ));
    }

    // 2. Execute
    let executor = Executor::new((*state.registry).clone());
    match executor.execute(graph).await {
        Ok(report) => Ok(Json(report)),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Execution failed: {err}"),
            }),
        )),
    }
}

async fn export_handler(
    State(state): State<AppState>,
    Json(payload): Json<ExportRequest>,
) -> Result<Json<ExportResponse>, (StatusCode, Json<ErrorResponse>)> {
    // 1. Validate
    let validator = Validator::new(&state.registry);
    if let Err(err) = validator.validate(&payload.graph) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Validation failed: {err}"),
            }),
        ));
    }

    let safe_name = codegen::sanitize_ident(&payload.project_name);
    let used_kinds: HashSet<&str> = payload.graph.nodes.iter().map(|n| n.kind.as_str()).collect();
    let has_network = used_kinds.contains("http_get");

    let cargo_toml = codegen::generate_cargo_toml(&safe_name, has_network);
    let nodes_rs = codegen::generate_nodes_rs(&used_kinds);
    let order = match plan::execution_order(&payload.graph) {
        Ok(o) => o,
        Err(e) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Planning error: {e}"),
                }),
            ))
        }
    };
    let main_rs = match codegen::generate_main_rs(&payload.graph, &order) {
        Ok(m) => m,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Codegen error: {e}"),
                }),
            ))
        }
    };

    Ok(Json(ExportResponse {
        project_name: safe_name,
        cargo_toml,
        main_rs,
        nodes_rs,
    }))
}

async fn export_disk_handler(
    State(state): State<AppState>,
    Json(payload): Json<ExportDiskRequest>,
) -> Result<Json<ExportDiskResponse>, (StatusCode, Json<ErrorResponse>)> {
    let validator = Validator::new(&state.registry);
    if let Err(err) = validator.validate(&payload.graph) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Validation failed: {err}"),
            }),
        ));
    }

    let safe_name = codegen::sanitize_ident(&payload.project_name);
    let out_path = Path::new(&payload.output_dir);

    match codegen::export_project(&payload.graph, out_path, &safe_name) {
        Ok(()) => Ok(Json(ExportDiskResponse {
            success: true,
            output_dir: payload.output_dir.clone(),
            message: format!("Project exported to {}", payload.output_dir),
        })),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Export to disk failed: {err}"),
            }),
        )),
    }
}

async fn copilot_hints_handler(
    State(state): State<AppState>,
    Json(graph): Json<GraphSpec>,
) -> Json<HintsResponse> {
    let hints = runtime::copilot::generate_hints(&graph, &state.registry);
    Json(HintsResponse { hints })
}

