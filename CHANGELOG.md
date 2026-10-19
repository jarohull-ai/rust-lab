# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]
### Added
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

### Fixed
- Corrupted `json_parse`, `text_clean` and `nodes/mod.rs` sources; compiler warnings in `fs_read`/`fs_write`.

## [0.1.0] - 2026-10-19
### Added
- Initial project structure.
- Base Cargo configuration.
- Architecture Decision Records (ADR) system.
- API Specification for Graph JSON.
