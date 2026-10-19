# ADR 001: Interchange Format for Graph Description

## Status
Accepted

## Context
The Rust Lab PoC needs a way to describe the visual graph (nodes and connections) created in the UI and pass it to the Rust execution engine. We need a format that is:
1. Easy to generate from a JS/TS frontend (React Flow).
2. Easy to deserialize in Rust.
3. Human-readable for debugging.

## Decision
We will use **JSON** as the primary interchange format. 

## Consequences
- **Pros:** Native support in almost every language, excellent tooling in Rust (`serde_json`), lightweight and standard.
- **Cons:** More verbose than binary formats, but for PoC scale, this is negligible.
