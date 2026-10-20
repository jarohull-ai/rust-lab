# Rust Lab — The Arduino of Rust

**Visual Graph → Real Rust Code → Execution**

Rust Lab is a standalone learning environment for building typed data-flow graphs, seeing the Rust code they generate, and running the result. The default product language is English; Polish is available through the `EN / PL` switch in the UI.

Current local release candidate: **v0.3.0**. It is not published to crates.io.

## Project status

- 11 built-in nodes for I/O, parsing, conversion, control flow, and output.
- DAG planner with data dependencies, control dependencies, branching, and branch pruning.
- Strict typed pins with explicit converter nodes.
- Construction Mode exports a standalone Cargo project.
- Project files, browser autosave, Undo/Redo, and downloadable Cargo ZIP archives.
- React Flow canvas with generated Rust code, execution diagnostics, and real `cargo check` feedback.
- SEED Tutor provides adaptive, deterministic learning hints without an LLM.
- Five guided lessons with live completion checks and browser-local progress.
- Real Cargo Workspace mode for prototypes, public GitHub repositories, crates.io dependencies,
  source editing, and isolated Cargo commands.

## Real Cargo Workspace Lab

Switch from **Graph** to **Lab** in the main toolbar. Lab mode works with real Cargo projects:

1. Create an independent binary package or import a public
   `https://github.com/OWNER/REPOSITORY` URL.
2. Browse supported source and configuration files without exposing arbitrary host paths.
3. Edit and save Rust, TOML, Markdown, JSON, YAML, and text files.
4. Search public crates.io packages and add a selected version to `Cargo.toml`.
5. Use **Fetch** to download dependencies into a cache dedicated to that project.
6. Run **Check**, **Test**, **Run**, **Clippy**, or **Fmt** offline in an isolated sandbox.

Every dependency change and Cargo command requires an explicit confirmation in the UI. Public
crates and repositories do not require an account. GitHub authentication for private repositories
and crates.io credentials for publishing are deliberately outside L1.

### Workspace sandbox

Rust Lab never executes imported code directly on the host. It prefers Bubblewrap and falls back
to a local `rust:bookworm` Docker image when the host cannot create an unprivileged namespace.
Execution has no network, drops Linux capabilities, uses CPU/memory/process/time limits, and sees
only the selected project plus its isolated dependency cache. **Fetch** and **Add crate** use a
separate contained phase with network access but no host Cargo credentials. Their Cargo credential
providers and external Git transport are disabled.

The local Workspace Lab therefore requires either a working Bubblewrap installation or Docker
with the `rust:bookworm` image. The standard graph, lessons, code generation, and export continue
to work without either sandbox backend.

## Workspace

```text
rust-lab/
├── crates/
│   ├── lab-node-sdk/   # Graph contracts, node trait, typed pins
│   ├── runtime/        # Validator, planner, executor, codegen, tutor
│   └── api/            # Axum API and static UI hosting
├── ui/                 # React, React Flow, CodeMirror, EN/PL localization
├── scripts/            # One-command local start and release verification
├── examples/           # Runnable graph JSON files
└── data/               # Local sample input and output
```

## Built-in nodes

| Node | Category | Inputs | Outputs |
|---|---|---|---|
| `http_get` | Network | `url: String` | `body: String` |
| `fs_read` | I/O | `path: FilePath` | `content: String` |
| `fs_write` | I/O | `path: FilePath`, `content: String` | — |
| `log_output` | Output | `text: Any` | — |
| `json_parse` | Data | `json_str: String`, `path: String` | `value: Json` |
| `text_clean` | Text | `text: String` | `cleaned: String` |
| `condition` | Control | `value: Boolean` | control branch |
| `delay` | Control | `duration: String` | — |
| `string_to_path` | Converter | `path: String` | `path: FilePath` |
| `json_to_string` | Converter | `value: Json` | `text: String` |
| `json_to_bool` | Converter | `value: Json` | `value: Boolean` |

## SEED Tutor

The tutor analyzes the graph in real time and returns structured hint codes rather than localized backend sentences. The UI renders the complete hint in the selected language. The published `seed-core` crate supplies a small local concept memory; Rust Lab adds the educational rules around it.

Current checks include:

1. Missing required inputs.
2. Orphan nodes.
3. Graph cycles.
4. Type mismatches with converter suggestions.
5. Invalid input and output ports.
6. Useful next-node suggestions for unconsumed outputs.

Hints adapt to repeated attempts:

1. First occurrence: a short nudge.
2. Second occurrence after a graph edit: an explanation of the Rust concept.
3. Third occurrence: a concrete solution.

Resolving the issue marks its concept as learned. Progress stays in the browser's local storage and can be reset from the tutor panel. Refreshing the same graph does not count as another attempt.

## Guided lessons

Open **Lessons** in the toolbar to start one of five exercises:

1. **Text Pipeline** — connect compatible String ports and run a two-node graph.
2. **HTTP and JSON** — build an HTTP → JSON → String → Log pipeline.
3. **Conditions** — perform an explicit Json-to-Boolean conversion and configure control-flow branches.
4. **Read a File Safely** — convert a String into a typed path and process local file content.
5. **Write a Result** — connect a safe output path and transformed text to a file writer.

Each lesson starts with an intentionally incomplete graph. Its checklist updates as the learner edits the graph, and the final criterion is awarded only after the current graph executes successfully. Completed lessons are stored locally in the browser.

## Projects and compiler feedback

The project menu supports New, Open, and Save for portable `.rustlab.json` files. Every edit is
also autosaved in the current browser. Use `Ctrl+Z`/`Ctrl+Y` for graph history and `Ctrl+S` to
download the current project file.

**Export Cargo** downloads a ZIP containing a complete standalone Rust project. **Check Rust**
runs `cargo check --offline` against the generated source and shows real compiler diagnostics.

## Run the application

From a fresh checkout, use the single local start command:

```bash
./scripts/run-local.sh
```

The script installs UI dependencies when needed, builds the interface, and starts the Rust API. Open `http://localhost:3030`. To use another port:

```bash
PORT=3031 ./scripts/run-local.sh
```

The equivalent manual command after the UI has been built is:

```bash
cargo run -p api
```

For frontend development:

```bash
# Terminal 1
cargo run -p api

# Terminal 2
cd ui
npm ci
npm run dev
```

Rust Lab listens only on `127.0.0.1` by default. To make a deliberately managed instance
available on a trusted LAN, set `RUST_LAB_HOST` explicitly.

### Docker

For a contained local deployment:

```bash
docker compose up --build
```

The supplied Compose configuration publishes Rust Lab only on `127.0.0.1:3030`, runs as an
unprivileged user without Linux capabilities, and provides temporary compile/output workspaces.

## Execution safety

- HTTP nodes use a host-controlled domain allowlist.
- File reads are limited to `data/` and `examples/`; writes are limited to `data/` and `output/`.
- Graph files cannot relax the host execution policy.
- API requests are limited to 1 MiB and graph execution is limited to 30 seconds.
- Server-side project export is restricted to `target/exports/`; browser ZIP export does not write
  an arbitrary server path.

Administrators can extend the defaults with `RUST_LAB_ALLOWED_DOMAINS`,
`RUST_LAB_ALLOWED_READ_DIRS`, and `RUST_LAB_ALLOWED_WRITE_DIRS`.

## Run a graph from the CLI

```bash
cargo run -p runtime -- examples/file_pipeline.json
cargo run -p runtime -- examples/web_scraper.json
```

## Export a standalone Cargo project

```bash
cargo run -p runtime -- --export ./my-app examples/web_scraper.json
cd my-app
cargo run
```

## Verification

Run the complete local release gate:

```bash
./scripts/verify.sh
```

Or run each command manually:

```bash
cargo test --workspace
cargo clippy --all-targets --all-features -- -D warnings

cd ui
npm ci
npm test
npm run build
```

## License

Rust Lab is available under the MIT or Apache-2.0 license.

The official Rust logo is used unmodified under CC BY 4.0. Rust Lab is an independent
educational project and is not affiliated with or endorsed by the Rust Project or Rust
Foundation. See [NOTICE.md](NOTICE.md) for attribution and trademark information.
