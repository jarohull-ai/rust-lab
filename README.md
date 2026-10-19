# 🚀 Rust Lab – The Arduino of Rust

**Visual Graph ↔ Code ↔ Execution** – hybrydowe środowisko do nauki i prototypowania w języku Rust.

> *"Move from Action $\rightarrow$ Modification $\rightarrow$ Implementation. Instead of fighting the compiler for hours, build working systems with atomic blocks, then dive into the code to understand why it works."*

---

## 📌 Project Status

- ✅ **Phase 1: Foundation (MVP)** – 8 atomic nodes, graph execution engine, DAG topological planner.
- ✅ **Phase 2 & 3: Control Flow & Dependencies** – data edges, branching (`condition` with `__next__`/`__skip__`), ordering (`after`), branch pruning.
- ✅ **Phase 4: Extended Validation & Typed Pins** – strict type checking (`PinType::is_compatible_with`), isolation of `FilePath` from `String`, static `config` validation.
- ✅ **Phase 4.5: Converter Nodes** – explicit type casting nodes (`string_to_path`, `json_to_string`, `json_to_bool`).
- ✅ **Phase 5: Construction Mode** – Code generator converting `graph.json` to a standalone Cargo project (`main.rs` with `tokio`, native `if/else`, and `nodes.rs`).
- 🔵 **Phase 6: Visual Canvas (UI)** – Interactive node graph editor.

---

## 🧱 Architecture

- **[`crates/lab-node-sdk`](crates/lab-node-sdk)**: Core SDK defining the `ViperNode` trait, `NodeSpec`, `TypedPin`, and `PinType` compatibility matrix.
- **[`crates/runtime`](crates/runtime)**: Execution runtime, CLI runner, `NodeRegistry`, `Executor`, `Validator`, `codegen`, and atomic node implementations.

### Available Nodes (11)
- **I/O & Network**: `http_get`, `fs_read`, `fs_write`, `log_output`
- **Data & Text**: `json_parse`, `text_clean`
- **Control**: `condition`, `delay`
- **Type Converters**: `string_to_path`, `json_to_string`, `json_to_bool`

---

## ⚡ Quick Start

### 1. Run Graph Interactively
```bash
cargo build
cargo run -p runtime -- graph.json
```

### 2. Export to Standalone Rust Project (Construction Mode)
```bash
# Export graph to a standalone Cargo project
cargo run -p runtime -- --export ./my-exported-app examples/condition_delay.json

# Build and run the exported project natively
cd my-exported-app
cargo run
```

### Run Examples
```bash
# Pipeline HTTP GET -> JSON Parse -> Clean -> Print
cargo run -p runtime -- examples/http_pipeline.json

# Branching and Delay with After control
cargo run -p runtime -- examples/condition_delay.json
```

---

## 🧪 Tests & Quality

```bash
cargo test        # 39/39 passing unit and integration tests
cargo clippy      # 0 warnings
```

---

## 📄 License

Dual-licensed under either of:
- MIT License ([LICENSE-MIT](LICENSE) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
