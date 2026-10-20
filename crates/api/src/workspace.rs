use crate::{bounded_output, AppState, ErrorResponse};
use axum::{
    extract::{Path as AxumPath, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    fs,
    path::{Component, Path, PathBuf},
    process::Stdio,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::io::AsyncWriteExt;

const MAX_FILE_BYTES: u64 = 512 * 1024;
const MAX_TREE_ENTRIES: usize = 2_000;
const MAX_PROJECT_BYTES: u64 = 100 * 1024 * 1024;
const MAX_PROJECT_FILES: usize = 10_000;
static PROJECT_SEQUENCE: AtomicU64 = AtomicU64::new(1);

type ApiError = (StatusCode, Json<ErrorResponse>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabProject {
    pub id: String,
    pub name: String,
    pub source: String,
    pub created_at: u64,
}

#[derive(Debug, Deserialize)]
pub struct NewProjectRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubImportRequest {
    pub url: String,
    #[serde(default)]
    pub reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceEntry {
    pub path: String,
    pub kind: String,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceTree {
    pub project: LabProject,
    pub entries: Vec<WorkspaceEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LabFile {
    pub project_id: String,
    pub path: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct FileQuery {
    pub project_id: String,
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct SaveFileRequest {
    pub project_id: String,
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CargoAction {
    Fetch,
    Check,
    Test,
    Run,
    Clippy,
    Fmt,
}

impl CargoAction {
    fn args(self) -> &'static [&'static str] {
        match self {
            Self::Fetch => &[
                "fetch",
                "--config",
                "net.git-fetch-with-cli=false",
                "--config",
                "registry.global-credential-providers=[]",
            ],
            Self::Check => &["check", "--offline", "--message-format=short"],
            Self::Test => &["test", "--offline", "--", "--nocapture"],
            Self::Run => &["run", "--offline"],
            Self::Clippy => &["clippy", "--offline", "--message-format=short"],
            Self::Fmt => &["fmt", "--", "--check"],
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Fetch => "fetch",
            Self::Check => "check",
            Self::Test => "test",
            Self::Run => "run",
            Self::Clippy => "clippy",
            Self::Fmt => "fmt",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CargoCommandRequest {
    pub project_id: String,
    pub action: CargoAction,
    #[serde(default)]
    pub approved: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CargoCommandResponse {
    pub success: bool,
    pub action: CargoAction,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u128,
    pub sandboxed: bool,
    pub sandbox: String,
}

#[derive(Debug, Deserialize)]
pub struct CrateSearchQuery {
    pub q: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrateSummary {
    pub name: String,
    pub max_version: String,
    pub description: String,
    pub downloads: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrateSearchResponse {
    pub crates: Vec<CrateSummary>,
}

#[derive(Debug, Deserialize)]
pub struct DependencyRequest {
    pub project_id: String,
    pub crate_name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub default_features: bool,
    #[serde(default)]
    pub approved: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyResponse {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Deserialize)]
struct CratesIoResponse {
    crates: Vec<CratesIoCrate>,
}

#[derive(Debug, Deserialize)]
struct CratesIoCrate {
    name: String,
    max_version: String,
    description: Option<String>,
    downloads: u64,
}

pub fn routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/lab/projects", get(list_projects).post(new_project))
        .route("/api/lab/import/github", post(import_github))
        .route("/api/lab/projects/:id/tree", get(project_tree))
        .route("/api/lab/file", get(read_file).put(save_file))
        .route("/api/lab/cargo", post(run_cargo))
        .route("/api/lab/crates/search", get(search_crates))
        .route("/api/lab/dependencies", post(add_dependency))
}

fn api_error(status: StatusCode, message: impl Into<String>) -> ApiError {
    (
        status,
        Json(ErrorResponse {
            error: message.into(),
        }),
    )
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn valid_slug(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 80
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        && value != "."
        && value != ".."
}

fn slugify(value: &str) -> String {
    let slug = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty() {
        "rust-project".to_string()
    } else {
        slug.chars().take(48).collect()
    }
}

fn allocate_project(root: &Path, name: &str) -> Result<(LabProject, PathBuf), ApiError> {
    fs::create_dir_all(root)
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let sequence = PROJECT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let id = format!("{}-{:x}-{sequence:x}", slugify(name), now_secs());
    let slot = root.join(&id);
    fs::create_dir(&slot)
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let project = LabProject {
        id,
        name: name.trim().to_string(),
        source: "local".to_string(),
        created_at: now_secs(),
    };
    Ok((project, slot))
}

fn write_metadata(slot: &Path, project: &LabProject) -> Result<(), ApiError> {
    let bytes = serde_json::to_vec_pretty(project)
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    fs::write(slot.join("project.json"), bytes)
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

fn load_project(root: &Path, id: &str) -> Result<(LabProject, PathBuf, PathBuf), ApiError> {
    if !valid_slug(id) {
        return Err(api_error(StatusCode::BAD_REQUEST, "Invalid project id"));
    }
    let slot = root.join(id);
    let project_root = slot.join("project");
    if !project_root.is_dir() {
        return Err(api_error(StatusCode::NOT_FOUND, "Lab project not found"));
    }
    let metadata = fs::read(slot.join("project.json"))
        .map_err(|_| api_error(StatusCode::NOT_FOUND, "Lab project metadata not found"))?;
    let project = serde_json::from_slice(&metadata).map_err(|_| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Invalid project metadata",
        )
    })?;
    Ok((project, slot, project_root))
}

async fn list_projects(State(state): State<AppState>) -> Result<Json<Vec<LabProject>>, ApiError> {
    fs::create_dir_all(state.workspace_root.as_ref())
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let mut projects = Vec::new();
    for entry in fs::read_dir(state.workspace_root.as_ref())
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
    {
        let Ok(entry) = entry else { continue };
        let Ok(bytes) = fs::read(entry.path().join("project.json")) else {
            continue;
        };
        if let Ok(project) = serde_json::from_slice::<LabProject>(&bytes) {
            projects.push(project);
        }
    }
    projects.sort_by(|a, b| {
        b.created_at
            .cmp(&a.created_at)
            .then_with(|| a.name.cmp(&b.name))
    });
    Ok(Json(projects))
}

async fn new_project(
    State(state): State<AppState>,
    Json(request): Json<NewProjectRequest>,
) -> Result<Json<LabProject>, ApiError> {
    let name = request.name.trim();
    if name.is_empty() || name.len() > 80 {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "Project name must contain 1-80 characters",
        ));
    }
    let (project, slot) = allocate_project(state.workspace_root.as_ref(), name)?;
    let project_root = slot.join("project");
    fs::create_dir_all(project_root.join("src"))
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let package = slugify(name).replace('-', "_");
    fs::write(
        project_root.join("Cargo.toml"),
        format!("[package]\nname = \"{package}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n\n[workspace]\n"),
    )
    .and_then(|_| {
        fs::write(
            project_root.join("src/main.rs"),
            "fn main() {\n    println!(\"Hello from Rust Lab!\");\n}\n",
        )
    })
    .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    write_metadata(&slot, &project)?;
    Ok(Json(project))
}

fn parse_github_url(raw: &str) -> Option<(String, String, String)> {
    let value = raw.trim().trim_end_matches('/').trim_end_matches(".git");
    let rest = value.strip_prefix("https://github.com/")?;
    let mut parts = rest.split('/');
    let owner = parts.next()?;
    let repo = parts.next()?;
    if parts.next().is_some() || !valid_slug(owner) || !valid_slug(repo) {
        return None;
    }
    Some((
        owner.to_string(),
        repo.to_string(),
        format!("https://github.com/{owner}/{repo}.git"),
    ))
}

fn valid_reference(value: &str) -> bool {
    value.is_empty()
        || (value.len() <= 120
            && !value.starts_with('-')
            && !value.contains("..")
            && value.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'/')
            }))
}

async fn import_github(
    State(state): State<AppState>,
    Json(request): Json<GitHubImportRequest>,
) -> Result<Json<LabProject>, ApiError> {
    let (owner, repo, url) = parse_github_url(&request.url).ok_or_else(|| {
        api_error(
            StatusCode::BAD_REQUEST,
            "Only public https://github.com/OWNER/REPOSITORY URLs are allowed",
        )
    })?;
    if !valid_reference(&request.reference) {
        return Err(api_error(StatusCode::BAD_REQUEST, "Invalid Git reference"));
    }
    let (mut project, slot) = allocate_project(state.workspace_root.as_ref(), &repo)?;
    project.source = format!("github:{owner}/{repo}");
    let destination = slot.join("project");
    let mut command = tokio::process::Command::new("git");
    command
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_ASKPASS", "/bin/false");
    command.args([
        "clone",
        "--depth",
        "1",
        "--single-branch",
        "--filter=blob:limit=2m",
    ]);
    if !request.reference.is_empty() {
        command.args(["--branch", request.reference.as_str()]);
    }
    command
        .arg("--")
        .arg(&url)
        .arg(&destination)
        .kill_on_drop(true);
    let output = tokio::time::timeout(Duration::from_secs(90), command.output()).await;
    let output = match output {
        Ok(Ok(output)) if output.status.success() => output,
        Ok(Ok(output)) => {
            let _ = fs::remove_dir_all(&slot);
            return Err(api_error(
                StatusCode::BAD_GATEWAY,
                bounded_output(&output.stderr),
            ));
        }
        Ok(Err(err)) => {
            let _ = fs::remove_dir_all(&slot);
            return Err(api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Could not run git: {err}"),
            ));
        }
        Err(_) => {
            let _ = fs::remove_dir_all(&slot);
            return Err(api_error(
                StatusCode::REQUEST_TIMEOUT,
                "GitHub import exceeded 90 seconds",
            ));
        }
    };
    let _ = output;
    if !destination.join("Cargo.toml").exists() {
        let _ = fs::remove_dir_all(&slot);
        return Err(api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Repository root has no Cargo.toml",
        ));
    }
    ensure_workspace_boundary(&destination.join("Cargo.toml"))?;
    let (files, bytes) = project_size(&destination)?;
    if files > MAX_PROJECT_FILES || bytes > MAX_PROJECT_BYTES {
        let _ = fs::remove_dir_all(&slot);
        return Err(api_error(
            StatusCode::PAYLOAD_TOO_LARGE,
            "Repository exceeds the Rust Lab project limit",
        ));
    }
    write_metadata(&slot, &project)?;
    Ok(Json(project))
}

fn ensure_workspace_boundary(manifest: &Path) -> Result<(), ApiError> {
    let content = fs::read_to_string(manifest).map_err(|err| {
        api_error(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            format!("Could not read Cargo.toml: {err}"),
        )
    })?;
    if !content.lines().any(|line| line.trim() == "[workspace]") {
        let mut updated = content;
        if !updated.ends_with('\n') {
            updated.push('\n');
        }
        updated.push_str("\n[workspace]\n");
        fs::write(manifest, updated).map_err(|err| {
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Could not isolate Cargo workspace: {err}"),
            )
        })?;
    }
    Ok(())
}

fn project_size(root: &Path) -> Result<(usize, u64), ApiError> {
    fn visit(path: &Path, files: &mut usize, bytes: &mut u64) -> std::io::Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.file_type()?;
            if metadata.is_symlink() {
                continue;
            }
            if metadata.is_dir() {
                visit(&entry.path(), files, bytes)?;
            } else if metadata.is_file() {
                *files += 1;
                *bytes = bytes.saturating_add(entry.metadata()?.len());
            }
        }
        Ok(())
    }
    let mut files = 0;
    let mut bytes = 0;
    visit(root, &mut files, &mut bytes)
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    Ok((files, bytes))
}

async fn project_tree(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<String>,
) -> Result<Json<WorkspaceTree>, ApiError> {
    let (project, _, root) = load_project(state.workspace_root.as_ref(), &id)?;
    let mut entries = Vec::new();
    collect_tree(&root, &root, &mut entries)?;
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(Json(WorkspaceTree { project, entries }))
}

fn collect_tree(
    root: &Path,
    current: &Path,
    entries: &mut Vec<WorkspaceEntry>,
) -> Result<(), ApiError> {
    if entries.len() >= MAX_TREE_ENTRIES {
        return Ok(());
    }
    for entry in fs::read_dir(current)
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
    {
        if entries.len() >= MAX_TREE_ENTRIES {
            break;
        }
        let entry =
            entry.map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
        let kind = entry
            .file_type()
            .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
        if kind.is_symlink() {
            continue;
        }
        let name = entry.file_name();
        if kind.is_dir()
            && matches!(
                name.to_str(),
                Some(".git" | ".cargo" | "target" | ".rust-lab")
            )
        {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(root)
            .unwrap_or(entry.path().as_path())
            .to_string_lossy()
            .replace('\\', "/");
        if kind.is_dir() {
            entries.push(WorkspaceEntry {
                path: relative,
                kind: "directory".to_string(),
                size: 0,
            });
            collect_tree(root, &entry.path(), entries)?;
        } else if kind.is_file() {
            let size = entry.metadata().map(|meta| meta.len()).unwrap_or(0);
            entries.push(WorkspaceEntry {
                path: relative,
                kind: "file".to_string(),
                size,
            });
        }
    }
    Ok(())
}

fn safe_relative_path(raw: &str) -> Option<PathBuf> {
    let path = Path::new(raw);
    if path.is_absolute() || raw.is_empty() || raw.contains('\0') {
        return None;
    }
    let mut safe = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => safe.push(part),
            _ => return None,
        }
    }
    let first = safe.components().next()?.as_os_str();
    if matches!(
        first.to_str(),
        Some(".git" | ".cargo" | "target" | ".rust-lab")
    ) {
        return None;
    }
    Some(safe)
}

fn allowed_text_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(OsStr::to_str),
        Some("rs" | "toml" | "lock" | "md" | "txt" | "json" | "yaml" | "yml")
    ) || matches!(
        path.file_name().and_then(OsStr::to_str),
        Some(".gitignore" | "LICENSE")
    )
}

fn resolve_existing_file(root: &Path, raw: &str) -> Result<(PathBuf, PathBuf), ApiError> {
    let relative = safe_relative_path(raw)
        .filter(|path| allowed_text_file(path))
        .ok_or_else(|| api_error(StatusCode::BAD_REQUEST, "Unsupported or unsafe file path"))?;
    let root = root
        .canonicalize()
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let candidate = root
        .join(&relative)
        .canonicalize()
        .map_err(|_| api_error(StatusCode::NOT_FOUND, "File not found"))?;
    if !candidate.starts_with(&root) || !candidate.is_file() {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "File escapes the project workspace",
        ));
    }
    Ok((relative, candidate))
}

async fn read_file(
    State(state): State<AppState>,
    Query(query): Query<FileQuery>,
) -> Result<Json<LabFile>, ApiError> {
    let (_, _, root) = load_project(state.workspace_root.as_ref(), &query.project_id)?;
    let (relative, candidate) = resolve_existing_file(&root, &query.path)?;
    let metadata = fs::metadata(&candidate)
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    if metadata.len() > MAX_FILE_BYTES {
        return Err(api_error(
            StatusCode::PAYLOAD_TOO_LARGE,
            "File is too large for the Lab editor",
        ));
    }
    let content = fs::read_to_string(candidate).map_err(|_| {
        api_error(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "File is not valid UTF-8 text",
        )
    })?;
    Ok(Json(LabFile {
        project_id: query.project_id,
        path: relative.to_string_lossy().replace('\\', "/"),
        content,
    }))
}

async fn save_file(
    State(state): State<AppState>,
    Json(request): Json<SaveFileRequest>,
) -> Result<Json<LabFile>, ApiError> {
    if request.content.len() as u64 > MAX_FILE_BYTES {
        return Err(api_error(
            StatusCode::PAYLOAD_TOO_LARGE,
            "File is too large for the Lab editor",
        ));
    }
    let (_, _, root) = load_project(state.workspace_root.as_ref(), &request.project_id)?;
    let (relative, candidate) = resolve_existing_file(&root, &request.path)?;
    fs::write(&candidate, request.content.as_bytes())
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    Ok(Json(LabFile {
        project_id: request.project_id,
        path: relative.to_string_lossy().replace('\\', "/"),
        content: request.content,
    }))
}

async fn run_cargo(
    State(state): State<AppState>,
    Json(request): Json<CargoCommandRequest>,
) -> Result<Json<CargoCommandResponse>, ApiError> {
    if !request.approved {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            "Explicit approval is required before executing project code",
        ));
    }
    let (_, slot, root) = load_project(state.workspace_root.as_ref(), &request.project_id)?;
    let root = root
        .canonicalize()
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let cache = slot.join("cargo-cache");
    fs::create_dir_all(cache.join("registry"))
        .and_then(|_| fs::create_dir_all(cache.join("git")))
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let cache = cache
        .canonicalize()
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let started = Instant::now();
    let (output, sandbox) = if matches!(request.action, CargoAction::Fetch) {
        docker_cargo(&root, &cache, request.action.args(), true, true).await?
    } else {
        sandboxed_cargo(&root, &cache, request.action).await?
    };
    let response = CargoCommandResponse {
        success: output.status.success(),
        action: request.action,
        stdout: bounded_output(&output.stdout),
        stderr: bounded_output(&output.stderr),
        duration_ms: started.elapsed().as_millis(),
        sandboxed: true,
        sandbox: sandbox.to_string(),
    };
    append_trajectory(
        &slot,
        request.action.label(),
        response.success,
        response.duration_ms,
    )
    .await;
    Ok(Json(response))
}

fn find_executable(name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|path| path.join(name))
            .find(|path| path.is_file())
    })
}

async fn sandboxed_cargo(
    root: &Path,
    cache: &Path,
    action: CargoAction,
) -> Result<(std::process::Output, &'static str), ApiError> {
    let bwrap = Path::new("/usr/bin/bwrap");
    if !bwrap.is_file() {
        return Err(api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "Safe Cargo sandbox is unavailable on this system",
        ));
    }
    let cargo = find_executable("cargo")
        .and_then(|path| path.canonicalize().ok())
        .ok_or_else(|| api_error(StatusCode::SERVICE_UNAVAILABLE, "Cargo is not installed"))?;
    let cargo_home = cargo.parent().and_then(Path::parent).ok_or_else(|| {
        api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "Could not locate Cargo home",
        )
    })?;
    let user_root = cargo_home.parent().ok_or_else(|| {
        api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "Could not locate Rust toolchain",
        )
    })?;
    let rustup_home = user_root.join(".rustup");
    if !rustup_home.is_dir() {
        return Err(api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "Rustup toolchain is unavailable",
        ));
    }
    let project_root = root
        .canonicalize()
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let prlimit = Path::new("/usr/bin/prlimit");
    let mut command = if prlimit.is_file() {
        let mut command = tokio::process::Command::new(prlimit);
        command.args(["--as=3221225472", "--nproc=128", "--"]);
        command.arg(bwrap);
        command
    } else {
        tokio::process::Command::new(bwrap)
    };
    command.args([
        "--unshare-all",
        "--die-with-parent",
        "--new-session",
        "--clearenv",
        "--proc",
        "/proc",
        "--dev",
        "/dev",
        "--tmpfs",
        "/tmp",
        "--ro-bind",
        "/usr",
        "/usr",
        "--ro-bind",
        "/etc",
        "/etc",
        "--dir",
        "/workspace",
        "--dir",
        "/cargo",
        "--dir",
        "/rustup",
        "--setenv",
        "HOME",
        "/tmp",
        "--setenv",
        "CARGO_HOME",
        "/cargo",
        "--setenv",
        "RUSTUP_HOME",
        "/rustup",
        "--setenv",
        "PATH",
        "/cargo/bin:/usr/bin:/bin",
        "--setenv",
        "CARGO_NET_OFFLINE",
        "true",
        "--setenv",
        "CARGO_TARGET_DIR",
        "/workspace/target",
    ]);
    for system_path in ["/bin", "/lib", "/lib64"] {
        if Path::new(system_path).exists() {
            command.args(["--ro-bind", system_path, system_path]);
        }
    }
    command.arg("--bind").arg(&project_root).arg("/workspace");
    command
        .arg("--ro-bind")
        .arg(cargo_home.join("bin"))
        .arg("/cargo/bin");
    for cache_name in ["registry", "git"] {
        let source = cache.join(cache_name);
        command
            .arg("--ro-bind")
            .arg(source)
            .arg(format!("/cargo/{cache_name}"));
    }
    command.arg("--ro-bind").arg(&rustup_home).arg("/rustup");
    command.args(["--chdir", "/workspace", "--", "/cargo/bin/cargo"]);
    command
        .args(action.args())
        .stdin(Stdio::null())
        .kill_on_drop(true);
    match tokio::time::timeout(Duration::from_secs(45), command.output()).await {
        Ok(Ok(output)) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !output.status.success() && stderr.starts_with("bwrap:") {
                docker_cargo(&project_root, cache, action.args(), false, false).await
            } else {
                Ok((output, "bubblewrap"))
            }
        }
        Ok(Err(_)) => docker_cargo(&project_root, cache, action.args(), false, false).await,
        Err(_) => Err(api_error(
            StatusCode::REQUEST_TIMEOUT,
            "Cargo command exceeded the 45 second Lab limit",
        )),
    }
}

fn numeric_identity(flag: &str) -> Option<String> {
    std::process::Command::new("id")
        .arg(flag)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
}

async fn docker_cargo(
    project_root: &Path,
    cache: &Path,
    cargo_args: &[&str],
    network: bool,
    writable_cache: bool,
) -> Result<(std::process::Output, &'static str), ApiError> {
    let docker = find_executable("docker").ok_or_else(|| {
        api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "No supported Cargo sandbox is available (Bubblewrap failed and Docker is missing)",
        )
    })?;
    let uid = numeric_identity("-u").unwrap_or_else(|| "1000".to_string());
    let gid = numeric_identity("-g").unwrap_or_else(|| "1000".to_string());
    let mut command = tokio::process::Command::new(docker);
    command.args(["run", "--rm"]);
    if !network {
        command.args(["--network", "none"]);
    }
    command.args([
        "--memory",
        "3g",
        "--cpus",
        "2",
        "--pids-limit",
        "128",
        "--cap-drop",
        "ALL",
        "--security-opt",
        "no-new-privileges",
        "--read-only",
        "--tmpfs",
        "/tmp:rw,noexec,nosuid,size=64m",
        "--user",
    ]);
    command.arg(format!("{uid}:{gid}"));
    command.arg("--mount").arg(format!(
        "type=bind,src={},dst=/workspace",
        project_root.display()
    ));
    command.arg("--mount").arg(format!(
        "type=bind,src={},dst=/cargo-cache{}",
        cache.display(),
        if writable_cache { "" } else { ",readonly=true" }
    ));
    command.args(["-e", "HOME=/tmp", "-e", "CARGO_HOME=/cargo-cache"]);
    command.arg("-e").arg(if network {
        "CARGO_NET_OFFLINE=false"
    } else {
        "CARGO_NET_OFFLINE=true"
    });
    command.args([
        "-e",
        "CARGO_TARGET_DIR=/workspace/target",
        "-w",
        "/workspace",
        "rust:bookworm",
        "/usr/local/cargo/bin/cargo",
    ]);
    command
        .args(cargo_args)
        .stdin(Stdio::null())
        .kill_on_drop(true);
    match tokio::time::timeout(Duration::from_secs(45), command.output()).await {
        Ok(Ok(output)) => Ok((output, "docker")),
        Ok(Err(err)) => Err(api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Could not start Docker Cargo sandbox: {err}"),
        )),
        Err(_) => Err(api_error(
            StatusCode::REQUEST_TIMEOUT,
            "Docker Cargo command exceeded the 45 second Lab limit",
        )),
    }
}

async fn append_trajectory(slot: &Path, action: &str, success: bool, duration_ms: u128) {
    let line = serde_json::json!({
        "ts": now_secs(),
        "event": "cargo_command",
        "action": action,
        "success": success,
        "duration_ms": duration_ms,
    });
    if let Ok(mut file) = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(slot.join("trajectory.jsonl"))
        .await
    {
        let _ = file.write_all(format!("{line}\n").as_bytes()).await;
    }
}

async fn search_crates(
    Query(query): Query<CrateSearchQuery>,
) -> Result<Json<CrateSearchResponse>, ApiError> {
    let term = query.q.trim();
    if term.len() < 2
        || term.len() > 80
        || !term
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "Crate search must contain 2-80 letters, digits, '-' or '_'",
        ));
    }
    let response = reqwest::Client::new()
        .get("https://crates.io/api/v1/crates")
        .query(&[("q", term), ("per_page", "12"), ("page", "1")])
        .header(
            reqwest::header::USER_AGENT,
            concat!(env!("CARGO_PKG_VERSION"), " rust-lab educational workspace"),
        )
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|err| {
            api_error(
                StatusCode::BAD_GATEWAY,
                format!("crates.io search failed: {err}"),
            )
        })?;
    if !response.status().is_success() {
        return Err(api_error(
            StatusCode::BAD_GATEWAY,
            format!("crates.io returned {}", response.status()),
        ));
    }
    let payload: CratesIoResponse = response.json().await.map_err(|err| {
        api_error(
            StatusCode::BAD_GATEWAY,
            format!("Invalid crates.io response: {err}"),
        )
    })?;
    Ok(Json(CrateSearchResponse {
        crates: payload
            .crates
            .into_iter()
            .map(|item| CrateSummary {
                name: item.name,
                max_version: item.max_version,
                description: item.description.unwrap_or_default(),
                downloads: item.downloads,
            })
            .collect(),
    }))
}

fn valid_crate_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn valid_version(value: &str) -> bool {
    value.is_empty()
        || (value.len() <= 40
            && value.bytes().all(|byte| {
                byte.is_ascii_alphanumeric()
                    || matches!(
                        byte,
                        b'.' | b'-' | b'+' | b'*' | b'^' | b'~' | b'<' | b'>' | b'=' | b','
                    )
            }))
}

async fn add_dependency(
    State(state): State<AppState>,
    Json(request): Json<DependencyRequest>,
) -> Result<Json<DependencyResponse>, ApiError> {
    if !request.approved {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            "Explicit approval is required before changing Cargo.toml",
        ));
    }
    if !valid_crate_name(&request.crate_name) || !valid_version(&request.version) {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "Invalid crate name or version",
        ));
    }
    if request.features.len() > 32
        || request
            .features
            .iter()
            .any(|feature| !valid_crate_name(feature))
    {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "Invalid crate feature list",
        ));
    }
    let (_, slot, root) = load_project(state.workspace_root.as_ref(), &request.project_id)?;
    let root = root
        .canonicalize()
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let package = if request.version.is_empty() {
        request.crate_name.clone()
    } else {
        format!("{}@{}", request.crate_name, request.version)
    };
    let cache = slot.join("cargo-cache");
    fs::create_dir_all(cache.join("registry"))
        .and_then(|_| fs::create_dir_all(cache.join("git")))
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let cache = cache
        .canonicalize()
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let mut args = vec![
        "add",
        "--registry",
        "crates-io",
        "--color",
        "never",
        "--config",
        "net.git-fetch-with-cli=false",
        "--config",
        "registry.global-credential-providers=[]",
    ];
    if !request.default_features {
        args.push("--no-default-features");
    }
    let feature_list = request.features.join(",");
    if !request.features.is_empty() {
        args.push("--features");
        args.push(&feature_list);
    }
    args.push(&package);
    let started = Instant::now();
    let (output, _) = docker_cargo(&root, &cache, &args, true, true).await?;
    append_trajectory(
        &slot,
        "add_dependency",
        output.status.success(),
        started.elapsed().as_millis(),
    )
    .await;
    Ok(Json(DependencyResponse {
        success: output.status.success(),
        stdout: bounded_output(&output.stdout),
        stderr: bounded_output(&output.stderr),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn github_urls_are_strictly_limited() {
        assert!(parse_github_url("https://github.com/rust-lang/rustlings").is_some());
        assert!(parse_github_url("https://github.com/rust-lang/rustlings.git").is_some());
        assert!(parse_github_url("http://github.com/rust-lang/rustlings").is_none());
        assert!(parse_github_url("https://evil.example/rust-lang/rustlings").is_none());
        assert!(parse_github_url("https://github.com/rust-lang/rustlings/tree/main").is_none());
    }

    #[test]
    fn workspace_paths_reject_escape_and_cargo_configuration() {
        assert_eq!(
            safe_relative_path("src/main.rs"),
            Some(PathBuf::from("src/main.rs"))
        );
        assert!(safe_relative_path("../secret").is_none());
        assert!(safe_relative_path("/etc/passwd").is_none());
        assert!(safe_relative_path(".cargo/config.toml").is_none());
        assert!(safe_relative_path("target/debug/app").is_none());
    }
}
