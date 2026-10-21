# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## [0.4.2-alpha.3] - 2026-10-21

### Security
- Added per-launch local API session authentication and enforced loopback-only binding.
- Made Rust, Go, Python, and C++ execution fail closed when no real sandbox is available.
- Pinned polyglot container images and GitHub Actions to immutable digests or commits.
- Backported the `glib` iterator fix for RUSTSEC-2024-0429 and updated affected TLS roots.

### Fixed
- Replaced shell-based C++ HTTP and JSON helpers with libcurl and nlohmann/json.
- Kept installer metadata numeric so Windows MSI and Linux RPM bundles accept prerelease builds;
  the Git tag and GitHub release remain explicitly marked as prerelease.

## [0.4.0-alpha.1] - 2026-10-20

Polyglot Engine evolution: multi-language visual-to-code architecture.

### Added
- Polyglot Architecture supporting multi-language code generation (Rust, Go, Python, C++).
- Go code generator emitting clean `main.go` and `go.mod` with native error handling and typed flows.
- Python code generator emitting modern `main.py` with type annotations and `asyncio` pipelines.
- C++ code generator emitting clean C++20 `main.cpp` and `CMakeLists.txt`.
- Dynamic UI language switcher supporting Rust, Go, Python, and C++ with official logos.
- Dynamic syntax highlighter adapting to the active target language in real-time.

## [0.3.0] - 2026-10-20

Public alpha candidate; not published to crates.io.

### Added
- Official Rust logo on the welcome screen, application toolbar, and browser tab.
- Portable `.rustlab.json` project files with browser autosave, Open/Save, and project naming.
- Debounced 100-step graph Undo/Redo history with standard keyboard shortcuts.
- Browser-generated Cargo ZIP downloads without server-side path access.
- Real offline `cargo check` feedback in the generated Rust code panel.
- Guided lessons for safe file reading and writing, bringing the course to five lessons.
- Keyboard-accessible node palette, visible focus styling, and reduced-motion support.
- Hardened Docker and Compose deployment for a contained local instance.
- Real Cargo Workspace Lab with local project creation and public GitHub repository import.
- Source tree and CodeMirror editor for supported Rust project files.
- Public crates.io search and explicit dependency additions.
- Confirmed Cargo Fetch followed by offline Check, Test, Run, Clippy, and Fmt commands.
- Per-project Cargo cache and `trajectory.jsonl` execution records.
- Runtime sandbox capability endpoint and localized UI readiness indicator.
- Public-project documentation: contribution guide, support policy, code of conduct, security model,
  roadmap, release process, and GitHub issue/PR templates.
- GitHub Actions release gate for Rust, UI, and container builds plus monthly Dependabot updates.

### Changed
- Product positioning now presents Rust Lab as one Learn, Build, and Experiment environment.
- Internal workspace crates are explicitly excluded from crates.io publishing until their public
  API and package identities receive a separate review.
- License files now consistently provide the declared `MIT OR Apache-2.0` choice.
- Docker deployment limitations for Cargo Workspace execution are documented explicitly.
- Local and CI verification now enforce Rust formatting and locked dependencies.

### Security
- The API now binds to `127.0.0.1` unless `RUST_LAB_HOST` is explicitly configured.
- Untrusted graph content can no longer extend execution allowlists.
- File reads now use an allowlist in addition to sensitive-path denial.
- Execution has a 30-second timeout and requests have a 1 MiB body limit.
- Server-side exports are confined to `target/exports/`.
- Imported code executes only in Bubblewrap or a hardened Docker fallback; Cargo actions are
  disabled when neither backend is ready.
- Networked dependency Fetch is separated from offline execution and receives no host Cargo
  credentials.
- Public GitHub import is shallow, non-interactive, size-limited, and restricted to HTTPS GitHub
  repository URLs.

## [0.2.0] - 2026-10-19

Local release candidate; not published to crates.io.

### Added
- SEED Tutor with deterministic `seed-core` concept matching and adaptive three-level hints.
- Complete English/Polish UI switch with English as the canonical default language.
- Three guided lessons with live criteria, successful-run checks, and browser-local progress.
- First-run welcome screen with an immediate language choice and lesson entry point.
- One-command local startup (`scripts/run-local.sh`) and release verification (`scripts/verify.sh`).
- **Phase 5: Construction Mode (Cargo Project Generator)**:
  - Code generator module `crates/runtime/src/codegen.rs` converting `GraphSpec` into a standalone Cargo workspace.
  - Generates self-contained `src/nodes.rs` with clean public functions for used nodes.
  - Generates idiomatic `src/main.rs` with variable data flow, native `if / else` branching via reachability analysis, and `tokio::main`.
  - Added CLI flag `--export <output_dir> <graph.json>` to runtime.
  - Full E2E tests verifying export and native compilation (`cargo check --offline` / `cargo run --offline`).
- **Phase 4.5: Converter Nodes**:
  - `string_to_path`: explicitly converts and validates a `String` to a `FilePath`.
  - `json_to_string`: converts parsed `Json` values (strings, numbers, objects) to `String`.
  - `json_to_bool`: converts parsed `Json` values (booleans, strings, numbers) to `Boolean`.
  - Configured `fs_read` and `fs_write` to use `PinType::FilePath` for strict path safety.
  - End-to-end converter pipeline unit tests and updated runnable examples.
- **Phase 4: Extended Validation & Typed Pins**:
  - `PinType::is_compatible_with` for strict type checking on edges (`String`, `Number`, `Boolean`, `Json`, `HttpResponse`, `FilePath`, `Any`).
  - Strict isolation of `FilePath` from `String` (fail-fast, requiring explicit conversion).
  - `PinType::matches_value` and static `config` validation for node inputs.
  - Detailed type mismatch error reporting (`Type mismatch on edge from 'nodeA.pinA' [TypeA] to 'nodeB.pinB' [TypeB]`).
  - Unit tests for PinType compatibility (SDK) and edge/config validation (Runtime).
- Edge-based data mapping between node outputs and inputs, with required-input checks.
- Nodes: `json_parse` (dot-path), `text_clean` (4 flags), `condition` (`__next__`/`__skip__`), `delay` (`ms`/`s`/`m`).
- `after` control dependencies on nodes for ordering nodes without outputs.
- Shared execution planner: topological order over data, branch and `after` edges; branch pruning.
- Validator: port-name checks, condition branch checks, `after` checks, cycle detection over all edge kinds.
- Runnable graphs in `examples/`; unit tests for nodes, planner, validator and executor.

### Changed
- API spec rewritten to match the format the runtime actually reads (v0.2).
- Product, API, runtime, and UI versions aligned at `0.2.0`.
- Execution safety rules renamed from SEED Policy to Execution Policy so SEED refers only to the tutor.
- Example and sample content normalized to English; Polish remains an explicit UI option.

### Fixed
- Corrupted `json_parse`, `text_clean` and `nodes/mod.rs` sources; compiler warnings in `fs_read`/`fs_write`.
- `json_parse` and `text_clean` configuration values are now visible and editable in the node UI.

## [0.1.0] - 2026-10-19
### Added
- Initial project structure.
- Base Cargo configuration.
- Architecture Decision Records (ADR) system.
- API Specification for Graph JSON.
