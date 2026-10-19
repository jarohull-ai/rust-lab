# API Specification: Graph Description Format v0.2

## Overview
This document defines the JSON structure used to describe a Rust Lab graph,
as accepted by `crates/runtime` (`lab_node_sdk::GraphSpec`).

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
- `condition` config has `true_branch` and `false_branch` pointing at existing,
  distinct nodes other than itself.
- `after` entries reference existing nodes, are not the node itself and are not duplicated.
- No cycles across data edges, condition branches and `after` lists.

## Node Catalogue (runtime v0.0.2)

| kind | inputs (required*) | outputs | notes |
|------|--------------------|---------|-------|
| `http_get` | `url`* | `body` | network |
| `fs_read` | `path`* | `content` | |
| `fs_write` | `path`*, `content`* | – | writes to disk |
| `log_output` | `text`* | – | prints to console |
| `json_parse` | `json_str`*, `path` | `value` | dot path, numeric segments index arrays |
| `text_clean` | `text`*, `trim` (default true), `remove_extra_whitespace`, `to_lower`, `to_upper` | `cleaned` | `to_lower` + `to_upper` is an error |
| `condition` | `value`* (bool), `true_branch`*, `false_branch`* | – | control flow |
| `delay` | `duration`* (`150ms`, `3s`, `2m`; max 10m) | – | |

## Example JSON
```json
{
  "graph_version": "0.1",
  "nodes": [
    { "id": "fetch", "kind": "http_get", "config": { "url": "https://httpbin.org/json" } },
    { "id": "title", "kind": "json_parse", "config": { "path": "slideshow.title" } },
    { "id": "wait", "kind": "delay", "config": { "duration": "1s" } },
    { "id": "print", "kind": "log_output", "config": {}, "after": ["wait"] }
  ],
  "edges": [
    { "from": { "node": "fetch", "port": "body" }, "to": { "node": "title", "port": "json_str" } },
    { "from": { "node": "title", "port": "value" }, "to": { "node": "print", "port": "text" } }
  ]
}
```

See `examples/` for runnable graphs: `cargo run -p runtime -- examples/condition_delay.json`.
