# Rust Lab — The Arduino of Rust

**Visual Graph → Real Rust Code → Execution**

Rust Lab is a standalone learning environment for building typed data-flow graphs, seeing the Rust code they generate, and running the result. The default product language is English; Polish is available through the `EN / PL` switch in the UI.

Current local release candidate: **v0.2.0**. It is not published to crates.io.

## Project status

- 11 built-in nodes for I/O, parsing, conversion, control flow, and output.
- DAG planner with data dependencies, control dependencies, branching, and branch pruning.
- Strict typed pins with explicit converter nodes.
- Construction Mode exports a standalone Cargo project.
- React Flow canvas with a generated Rust code view and execution diagnostics.
- SEED Tutor provides adaptive, deterministic learning hints without an LLM.
- Three guided lessons with live completion checks and browser-local progress.

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

Open **Lessons** in the toolbar to start one of three exercises:

1. **Text Pipeline** — connect compatible String ports and run a two-node graph.
2. **HTTP and JSON** — build an HTTP → JSON → String → Log pipeline.
3. **Conditions** — perform an explicit Json-to-Boolean conversion and configure control-flow branches.

Each lesson starts with an intentionally incomplete graph. Its checklist updates as the learner edits the graph, and the final criterion is awarded only after the current graph executes successfully. Completed lessons are stored locally in the browser.

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
