use axum::{
    extract::{DefaultBodyLimit, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use lab_node_sdk::{GraphSpec, NodeSpec};
use runtime::{codegen, plan, ExecutionReport, Executor, NodeRegistry, Validator};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tower_http::services::ServeDir;

mod workspace;
pub use workspace::{
    CargoAction, CargoCommandRequest, CargoCommandResponse, CrateSearchResponse, DependencyRequest,
    GitHubImportRequest, LabFile, LabProject, NewProjectRequest, SandboxCapabilities,
    SaveFileRequest, WorkspaceTree,
};

static COMPILE_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<NodeRegistry>,
    pub workspace_root: Arc<PathBuf>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            registry: Arc::new(NodeRegistry::default()),
            workspace_root: Arc::new(PathBuf::from("target/rust-lab-workspaces")),
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
    #[serde(default)]
    pub target: Option<lab_node_sdk::LanguageTarget>,
}

fn default_project_name() -> String {
    "polyglot_lab_app".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportResponse {
    pub target: lab_node_sdk::LanguageTarget,
    pub project_name: String,
    pub primary_file: String,
    pub files: Vec<lab_node_sdk::GeneratedFile>,
    #[serde(default)]
    pub cargo_toml: String,
    #[serde(default)]
    pub main_rs: String,
    #[serde(default)]
    pub nodes_rs: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompileResponse {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
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
    pub hints: Vec<runtime::tutor::Hint>,
    pub progress: runtime::TutorProgress,
    pub learned_now: Vec<runtime::HintConcept>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdaptiveHintsRequest {
    pub graph: GraphSpec,
    #[serde(default)]
    pub progress: runtime::TutorProgress,
    #[serde(default)]
    pub revision: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HintsRequest {
    Adaptive(AdaptiveHintsRequest),
    Graph(GraphSpec),
}

pub fn app(state: AppState) -> Router {
    let mut router = Router::new()
        .route("/api/health", get(health_handler))
        .route("/api/nodes", get(nodes_handler))
        .route("/api/validate", post(validate_handler))
        .route("/api/execute", post(execute_handler))
        .route("/api/export", post(export_handler))
        .route("/api/compile", post(compile_handler))
        .route("/api/export/disk", post(export_disk_handler))
        .route("/api/tutor/hints", post(tutor_hints_handler));

    router = workspace::routes(router);

    if Path::new("ui/dist").exists() {
        router = router.fallback_service(ServeDir::new("ui/dist"));
    }

    router
        .layer(DefaultBodyLimit::max(1_048_576))
        .with_state(state)
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
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
    match tokio::time::timeout(Duration::from_secs(30), executor.execute(graph)).await {
        Ok(Ok(report)) => Ok(Json(report)),
        Ok(Err(err)) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Execution failed: {err}"),
            }),
        )),
        Err(_) => Err((
            StatusCode::REQUEST_TIMEOUT,
            Json(ErrorResponse {
                error: "Execution exceeded the 30 second classroom limit".to_string(),
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

    let target = payload.target.unwrap_or(lab_node_sdk::LanguageTarget::Rust);
    let polyglot_proj = match codegen::generate_polyglot_project(&payload.graph, &payload.project_name, target) {
        Ok(p) => p,
        Err(e) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Codegen error: {e}"),
                }),
            ));
        }
    };

    let cargo_toml = polyglot_proj
        .files
        .iter()
        .find(|f| f.path == "Cargo.toml")
        .map(|f| f.content.clone())
        .unwrap_or_default();
    let main_rs = polyglot_proj
        .files
        .iter()
        .find(|f| f.path == "src/main.rs")
        .map(|f| f.content.clone())
        .unwrap_or_default();
    let nodes_rs = polyglot_proj
        .files
        .iter()
        .find(|f| f.path == "src/nodes.rs")
        .map(|f| f.content.clone())
        .unwrap_or_default();

    Ok(Json(ExportResponse {
        target: polyglot_proj.target,
        project_name: polyglot_proj.project_name,
        primary_file: polyglot_proj.primary_file,
        files: polyglot_proj.files,
        cargo_toml,
        main_rs,
        nodes_rs,
    }))
}

async fn compile_handler(
    State(state): State<AppState>,
    Json(payload): Json<ExportRequest>,
) -> Result<Json<CompileResponse>, (StatusCode, Json<ErrorResponse>)> {
    let validator = Validator::new(&state.registry);
    if let Err(err) = validator.validate(&payload.graph) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Validation failed: {err}"),
            }),
        ));
    }

    let project_name = codegen::sanitize_ident(&payload.project_name);
    let sequence = COMPILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let workspace = PathBuf::from("target/rust-lab-compile-checks")
        .join(format!("{}-{sequence}", std::process::id()));
    let _ = std::fs::remove_dir_all(&workspace);
    if let Err(err) = codegen::export_project(&payload.graph, &workspace, &project_name) {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Code generation failed: {err}"),
            }),
        ));
    }

    let target_dir = match std::env::current_dir() {
        Ok(dir) => dir.join("target/rust-lab-compile-cache"),
        Err(err) => {
            let _ = std::fs::remove_dir_all(&workspace);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Could not locate compile workspace: {err}"),
                }),
            ));
        }
    };
    let mut command = tokio::process::Command::new("cargo");
    command
        .args(["check", "--offline", "--message-format=short"])
        .current_dir(&workspace)
        .env("CARGO_TARGET_DIR", target_dir)
        .kill_on_drop(true);

    let result = tokio::time::timeout(Duration::from_secs(30), command.output()).await;
    let response = match result {
        Ok(Ok(output)) => CompileResponse {
            success: output.status.success(),
            stdout: bounded_output(&output.stdout),
            stderr: bounded_output(&output.stderr),
        },
        Ok(Err(err)) => CompileResponse {
            success: false,
            stdout: String::new(),
            stderr: format!("Could not run cargo check: {err}"),
        },
        Err(_) => CompileResponse {
            success: false,
            stdout: String::new(),
            stderr: "cargo check exceeded the 30 second classroom limit".to_string(),
        },
    };
    let _ = std::fs::remove_dir_all(&workspace);
    Ok(Json(response))
}

fn bounded_output(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .chars()
        .take(16_000)
        .collect()
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
    let out_path = safe_export_path(&payload.output_dir).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "output_dir must be a relative path inside target/exports".to_string(),
            }),
        )
    })?;

    match codegen::export_project(&payload.graph, &out_path, &safe_name) {
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

fn safe_export_path(raw: &str) -> Option<PathBuf> {
    let path = Path::new(raw);
    if path.is_absolute() || raw.contains("..") || raw.starts_with('~') {
        return None;
    }
    let base = Path::new("target/exports");
    path.starts_with(base).then(|| path.to_path_buf())
}

async fn tutor_hints_handler(
    State(state): State<AppState>,
    Json(request): Json<HintsRequest>,
) -> Json<HintsResponse> {
    let (graph, progress, revision) = match request {
        HintsRequest::Adaptive(payload) => {
            let revision = if payload.revision.is_empty() {
                serde_json::to_string(&payload.graph).unwrap_or_default()
            } else {
                payload.revision
            };
            (payload.graph, payload.progress, revision)
        }
        HintsRequest::Graph(graph) => {
            let revision = serde_json::to_string(&graph).unwrap_or_default();
            (graph, runtime::TutorProgress::default(), revision)
        }
    };
    let outcome = runtime::tutor::tutor_outcome(&graph, &state.registry, progress, revision);
    Json(HintsResponse {
        hints: outcome.hints,
        progress: outcome.progress,
        learned_now: outcome.learned_now,
    })
}
