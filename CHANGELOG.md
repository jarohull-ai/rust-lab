# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]
### Added
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
