# API Specification: Rust Lab v0.3

## Overview
This document defines the JSON structure used to describe a Rust Lab graph,
as accepted by `crates/runtime` (`lab_node_sdk::GraphSpec`).

It also records the local HTTP surface used by the bundled UI. The API is an alpha interface and
is not a remotely hosted or multi-tenant service contract.

> v0.1.0 of this document described `type`/`params`/`source`/`target` fields that the
> runtime never implemented. v0.2 documents the format the runtime actually reads.

## Data Structure

### Graph Object
- `graph_version`: string (e.g. `"0.1"`)
- `nodes`: array of Node Objects
- `edges`: array of Edge Objects (data flow)

### Node Object
- `id`: string, unique within the graph
- `kind`: string, a registered node name (e.g. `"http_get"`)
- `config`: object, static input values keyed by input pin name
- `after` *(optional)*: array of node ids that must finish before this node runs
  (control dependency, no data). Use it to order nodes that have no outputs,
  e.g. a `log_output` after a `delay`.
- Unknown fields (e.g. a future UI `position`) are ignored by the runtime.

### Edge Object (data)
- `from`: `{ "node": <id>, "port": <output pin name> }`
- `to`: `{ "node": <id>, "port": <input pin name> }`

A value arriving over an edge overrides the same key in the target's `config`.

## Execution Semantics
1. Nodes run in a topological order built from three dependency kinds:
   data edges, `condition` branches (`condition → true_branch/false_branch`)
   and `after` lists. Ties keep the order in which nodes are declared.
2. Before a node runs, every required input pin must be present and non-null.
3. A `condition` node emits the reserved keys `__next__` (branch taken) and
   `__skip__` (branch not taken). Nodes reachable **only** from the skipped branch
   (via edges, branches or `after`) are pruned. Nodes reachable from both
   branches (merge points) still run.
4. Reserved output keys (`__next__`, `__skip__`) are never exposed as data.

## Validation Rules
- Node ids are unique; every `kind` is registered.
- Edge endpoints exist; `from.port` is a declared output, `to.port` a declared input.
- **Edge Type Compatibility**: The source pin type (`from_pin.pin_type`) must be compatible with the destination pin type (`to_pin.pin_type`).
- **Static Config Type Checking**: Values provided in a node's `config` must strictly match the declared `pin_type` of the target input pin.
- `condition` config has `true_branch` and `false_branch` pointing at existing,
  distinct nodes other than itself.
- `after` entries reference existing nodes, are not the node itself and are not duplicated.
- No cycles across data edges, condition branches and `after` lists.

## Type System & Compatibility Matrix

Rust Lab enforces strict type checking at validation time to catch wiring errors before graph execution.

### Pin Types (`PinType`)
- `string`: UTF-8 textual data
- `number`: Numeric values (integers and floats)
- `boolean`: `true` or `false`
- `json`: Structured or parsed JSON value (`serde_json::Value`)
- `http_response`: HTTP response payload/metadata
- `file_path`: Filesystem path (strictly distinct from generic `string`)
- `any`: Universal escape hatch (accepts and emits any data type; used by `log_output`)

### Compatibility Matrix (`from.is_compatible_with(to)`)

| From \ To | `string` | `number` | `boolean` | `json` | `http_response` | `file_path` | `any` |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **`string`** | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| **`number`** | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ |
| **`boolean`** | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ |
| **`json`** | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ |
| **`http_response`** | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ |
| **`file_path`** | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ |
| **`any`** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

> **Fail-Fast Rule**: `file_path` is strictly distinct from `string`. Connecting a generic `string` (e.g. `http_get.body`) to a `file_path` is a validation error. Explicit conversion nodes must be used.

## Node Catalogue (runtime v0.3.0)

| kind | inputs (required*) | outputs | notes |
|------|--------------------|---------|-------|
| `http_get` | `url`* (string) | `body` (string) | network |
| `fs_read` | `path`* (file_path) | `content` (string) | |
| `fs_write` | `path`* (file_path), `content`* (string) | – | writes to disk |
| `log_output` | `text`* (any) | – | prints to console |
| `json_parse` | `json_str`* (string), `path` (string) | `value` (json) | dot path, numeric segments index arrays |
| `text_clean` | `text`* (string), `trim` (bool), `remove_extra_whitespace` (bool), `to_lower` (bool), `to_upper` (bool) | `cleaned` (string) | `to_lower` + `to_upper` is an error |
| `condition` | `value`* (boolean), `true_branch`* (string), `false_branch`* (string) | – | control flow (`__next__`/`__skip__`) |
| `delay` | `duration`* (string) | – | `150ms`, `3s`, `2m`; max 10m |
| `string_to_path` | `path`* (string) | `path` (file_path) | explicit string to filesystem path converter |
| `json_to_string` | `value`* (json) | `text` (string) | JSON string/number/object to string |
| `json_to_bool` | `value`* (json) | `value` (boolean) | JSON boolean/string/number to boolean |

## Example JSON
```json
{
  "graph_version": "0.1",
  "nodes": [
    { "id": "fetch", "kind": "http_get", "config": { "url": "https://httpbin.org/json" } },
    { "id": "title", "kind": "json_parse", "config": { "path": "slideshow.title" } },
    { "id": "as_str", "kind": "json_to_string", "config": {} },
    { "id": "wait", "kind": "delay", "config": { "duration": "1s" } },
    { "id": "print", "kind": "log_output", "config": {}, "after": ["wait"] }
  ],
  "edges": [
    { "from": { "node": "fetch", "port": "body" }, "to": { "node": "title", "port": "json_str" } },
    { "from": { "node": "title", "port": "value" }, "to": { "node": "as_str", "port": "value" } },
    { "from": { "node": "as_str", "port": "text" }, "to": { "node": "print", "port": "text" } }
  ]
}
```

See `examples/` for runnable graphs: `cargo run -p runtime -- examples/condition_delay.json`.

## Dual Execution Modes

### 1. Fast Runtime (Interpretive)
Directly executes the graph JSON using the modular runtime engine:
```bash
cargo run -p runtime -- <graph.json>
```

### 2. Construction Mode (Compiled Standalone Export)
Generates an independent Cargo project with native async Rust code, typed variable bindings, and native `if/else` control flow:
```bash
cargo run -p runtime -- --export <output_dir> <graph.json>
cd <output_dir> && cargo run
```

## Host-controlled execution policy

The runtime treats graph JSON as untrusted input. A graph may still contain the legacy `security`
field for interchange compatibility, but it cannot extend host allowlists. Administrators configure
additional domains and directories only through `RUST_LAB_ALLOWED_DOMAINS`,
`RUST_LAB_ALLOWED_READ_DIRS`, and `RUST_LAB_ALLOWED_WRITE_DIRS`.

Default file reads are restricted to `data/` and `examples/`. Default writes are restricted to
`data/` and `output/`. Absolute paths, home-relative paths, and parent traversal are rejected.

## Local HTTP API

All endpoints are rooted at the Rust Lab origin and accept/return JSON unless noted otherwise.

| Method | Path | Purpose |
|---|---|---|
| `GET` | `/api/health` | API service and version health |
| `GET` | `/api/nodes` | Registered visual node specifications |
| `POST` | `/api/validate` | Validate a graph and return its execution order |
| `POST` | `/api/execute` | Validate and execute a graph under host policy |
| `POST` | `/api/export` | Generate standalone Cargo project source |
| `POST` | `/api/compile` | Check generated Rust with offline Cargo |
| `POST` | `/api/tutor/hints` | Return deterministic educational hints |
| `GET/POST` | `/api/lab/projects` | List or create local Cargo workspaces |
| `POST` | `/api/lab/import/github` | Import a public GitHub HTTPS repository |
| `GET` | `/api/lab/projects/:id/tree` | List supported project files |
| `GET/PUT` | `/api/lab/file` | Read or update a supported text file |
| `GET` | `/api/lab/capabilities` | Probe safe Cargo execution backends |
| `POST` | `/api/lab/cargo` | Run an explicitly approved Cargo action |
| `GET` | `/api/lab/crates/search` | Search the public crates.io index API |
| `POST` | `/api/lab/dependencies` | Add an explicitly approved dependency |

### Sandbox capability response

```json
{
  "offline_execution_available": true,
  "dependency_fetch_available": true,
  "cargo_detected": true,
  "bubblewrap_ready": true,
  "docker_ready": true,
  "preferred_backend": "bubblewrap"
}
```

Readiness is actively probed. `dependency_fetch_available` currently requires Docker because Fetch
is the only Cargo phase allowed network access. The UI must not offer an unavailable action as if it
were executable.
