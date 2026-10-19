use anyhow::{Context, Result};
use lab_node_sdk::GraphSpec;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// Sanitizes a string into a valid Rust identifier (e.g. "welcome-user" -> "welcome_user").
pub fn sanitize_ident(s: &str) -> String {
    let mut ident = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_alphanumeric() || c == '_' {
            ident.push(c);
        } else {
            ident.push('_');
        }
        if i == 0 && c.is_ascii_digit() {
            ident.insert(0, '_');
        }
    }
    if ident.is_empty() {
        ident.push('_');
    }
    ident
}

/// Computes reachable nodes from a starting node ID following data edges, condition branches, and `after` lists.
pub fn reachable_from(start_id: &str, graph: &GraphSpec) -> HashSet<String> {
    let mut visited = HashSet::new();
    let mut queue = vec![start_id.to_string()];
    visited.insert(start_id.to_string());

    // Build adjacency list
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for edge in &graph.edges {
        adj.entry(&edge.from.node).or_default().push(&edge.to.node);
    }
    for node in &graph.nodes {
        for dep in &node.after {
            adj.entry(dep.as_str()).or_default().push(&node.id);
        }
        if node.kind == "condition" {
            if let Some(tb) = node.config.get("true_branch").and_then(|v| v.as_str()) {
                adj.entry(&node.id).or_default().push(tb);
            }
            if let Some(fb) = node.config.get("false_branch").and_then(|v| v.as_str()) {
                adj.entry(&node.id).or_default().push(fb);
            }
        }
    }

    while let Some(curr) = queue.pop() {
        if let Some(neighbors) = adj.get(curr.as_str()) {
            for &next in neighbors {
                if visited.insert(next.to_string()) {
                    queue.push(next.to_string());
                }
            }
        }
    }

    visited
}

/// Generates Cargo.toml content.
pub fn generate_cargo_toml(project_name: &str, has_network: bool) -> String {
    let mut toml = format!(
        r#"[package]
name = "{project_name}"
version = "0.1.0"
edition = "2021"

[workspace]

[dependencies]
tokio = {{ version = "1", features = ["full"] }}
serde_json = "1"
anyhow = "1"
"#
    );

    if has_network {
        toml.push_str(r#"reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
"#);
    }

    toml
}

/// Generates `src/nodes.rs` with self-contained implementations of the required nodes.
pub fn generate_nodes_rs(used_kinds: &HashSet<&str>) -> String {
    let mut code = String::from(
        r#"//! Auto-generated atomic node functions for Rust Lab exported project.
#![allow(unused_imports, dead_code)]
use anyhow::{anyhow, Result};
use serde_json::Value;

"#,
    );

    if used_kinds.contains("http_get") {
        code.push_str(
            r#"pub async fn http_get(url: &str) -> Result<String> {
    println!("   Fetching: {}", url);
    let client = reqwest::Client::builder()
        .user_agent("RustLab-Export/0.1.0")
        .build()?;
    let resp = client.get(url).send().await?.text().await?;
    Ok(resp)
}

"#,
        );
    }

    if used_kinds.contains("fs_read") {
        code.push_str(
            r#"pub fn fs_read(path: &str) -> Result<String> {
    println!("   Reading file: {}", path);
    let content = std::fs::read_to_string(path)?;
    Ok(content)
}

"#,
        );
    }

    if used_kinds.contains("fs_write") {
        code.push_str(
            r#"pub fn fs_write(path: &str, content: &str) -> Result<()> {
    println!("   Writing to file: {}", path);
    std::fs::write(path, content)?;
    Ok(())
}

"#,
        );
    }

    if used_kinds.contains("log_output") {
        code.push_str(
            r#"pub fn log_output(text: impl std::fmt::Display) -> Result<()> {
    println!("   Log Output: {}", text);
    Ok(())
}

"#,
        );
    }

    if used_kinds.contains("json_parse") {
        code.push_str(
            r#"pub fn select_path(value: &Value, path: &str) -> Result<Value> {
    let mut current = value;
    for segment in path.split('.').filter(|s| !s.is_empty()) {
        current = match current {
            Value::Object(map) => map.get(segment),
            Value::Array(items) => segment.parse::<usize>().ok().and_then(|i| items.get(i)),
            _ => None,
        }
        .ok_or_else(|| anyhow!("Path segment '{segment}' not found in '{path}'"))?;
    }
    Ok(current.clone())
}

pub fn json_parse(raw: &str, path: Option<&str>) -> Result<Value> {
    let parsed: Value = serde_json::from_str(raw)
        .map_err(|e| anyhow!("Invalid JSON: {e}"))?;
    match path.filter(|p| !p.is_empty()) {
        Some(p) => select_path(&parsed, p),
        None => Ok(parsed),
    }
}

"#,
        );
    }

    if used_kinds.contains("text_clean") {
        code.push_str(
            r#"pub fn text_clean(
    text: &str,
    trim: bool,
    collapse: bool,
    to_lower: bool,
    to_upper: bool,
) -> Result<String> {
    if to_lower && to_upper {
        return Err(anyhow!("'to_lower' and 'to_upper' cannot both be true"));
    }
    let mut result = text.to_string();
    if trim {
        result = result.trim().to_string();
    }
    if collapse {
        let lead = !trim && result.starts_with(char::is_whitespace);
        let trail = !trim && result.ends_with(char::is_whitespace);
        let core = result.split_whitespace().collect::<Vec<_>>().join(" ");
        result = format!(
            "{}{}{}",
            if lead { " " } else { "" },
            core,
            if trail && !core.is_empty() { " " } else { "" }
        );
    }
    if to_lower {
        result = result.to_lowercase();
    }
    if to_upper {
        result = result.to_uppercase();
    }
    Ok(result)
}

"#,
        );
    }

    if used_kinds.contains("delay") {
        code.push_str(
            r#"pub async fn delay(duration_str: &str) -> Result<()> {
    let text = duration_str.trim();
    let split = text
        .find(|c: char| !c.is_ascii_digit())
        .ok_or_else(|| anyhow!("Duration '{text}' needs a unit: ms, s or m"))?;
    let (number, unit) = text.split_at(split);
    let amount: u64 = number.parse()?;
    let duration = match unit {
        "ms" => std::time::Duration::from_millis(amount),
        "s" => std::time::Duration::from_secs(amount),
        "m" => std::time::Duration::from_secs(amount * 60),
        other => return Err(anyhow!("Unknown duration unit '{other}'")),
    };
    println!("   Sleeping for {duration:?}");
    tokio::time::sleep(duration).await;
    Ok(())
}

"#,
        );
    }

    if used_kinds.contains("string_to_path") {
        code.push_str(
            r#"pub fn string_to_path(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("Path cannot be empty"));
    }
    Ok(trimmed.to_string())
}

"#,
        );
    }

    if used_kinds.contains("json_to_string") {
        code.push_str(
            r#"pub fn json_to_string(val: &Value) -> Result<String> {
    match val {
        Value::String(s) => Ok(s.clone()),
        _ => Ok(serde_json::to_string(val)?),
    }
}

"#,
        );
    }

    if used_kinds.contains("json_to_bool") {
        code.push_str(
            r#"pub fn json_to_bool(val: &Value) -> Result<bool> {
    match val {
        Value::Bool(b) => Ok(*b),
        Value::String(s) => match s.trim().to_lowercase().as_str() {
            "true" | "1" | "yes" => Ok(true),
            "false" | "0" | "no" => Ok(false),
            other => Err(anyhow!("Cannot convert string '{other}' to boolean")),
        },
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                match i {
                    1 => Ok(true),
                    0 => Ok(false),
                    other => Err(anyhow!("Cannot convert number {other} to boolean")),
                }
            } else {
                Err(anyhow!("Cannot convert floating number to boolean"))
            }
        }
        other => Err(anyhow!("Cannot convert JSON value to boolean")),
    }
}

"#,
        );
    }

    code
}

/// Generates code for a single node invocation.
fn generate_node_call(node: &lab_node_sdk::NodeInstance, graph: &GraphSpec, indent: &str) -> String {
    let id_ident = sanitize_ident(&node.id);
    // Find incoming edges
    let in_edges: HashMap<&str, (&str, &str)> = graph
        .edges
        .iter()
        .filter(|e| e.to.node == node.id)
        .map(|e| (e.to.port.as_str(), (e.from.node.as_str(), e.from.port.as_str())))
        .collect();

    let mut out = String::new();
    out.push_str(&format!("{indent}// 🚀 Node: '{}' [{}]\n", node.id, node.kind));

    match node.kind.as_str() {
        "http_get" => {
            let url_expr = if let Some(&(src, sport)) = in_edges.get("url") {
                format!("&{}_{}", sanitize_ident(src), sanitize_ident(sport))
            } else {
                let u = node.config.get("url").and_then(|v| v.as_str()).unwrap_or("");
                format!("\"{u}\"")
            };
            out.push_str(&format!(
                "{indent}let {id_ident}_body = nodes::http_get({url_expr}).await?;\n"
            ));
        }
        "fs_read" => {
            let path_expr = if let Some(&(src, sport)) = in_edges.get("path") {
                format!("&{}_{}", sanitize_ident(src), sanitize_ident(sport))
            } else {
                let p = node.config.get("path").and_then(|v| v.as_str()).unwrap_or("");
                format!("\"{p}\"")
            };
            out.push_str(&format!(
                "{indent}let {id_ident}_content = nodes::fs_read({path_expr})?;\n"
            ));
        }
        "fs_write" => {
            let path_expr = if let Some(&(src, sport)) = in_edges.get("path") {
                format!("&{}_{}", sanitize_ident(src), sanitize_ident(sport))
            } else {
                let p = node.config.get("path").and_then(|v| v.as_str()).unwrap_or("");
                format!("\"{p}\"")
            };
            let content_expr = if let Some(&(src, sport)) = in_edges.get("content") {
                format!("&{}_{}", sanitize_ident(src), sanitize_ident(sport))
            } else {
                let c = node.config.get("content").and_then(|v| v.as_str()).unwrap_or("");
                format!("\"{c}\"")
            };
            out.push_str(&format!(
                "{indent}nodes::fs_write({path_expr}, {content_expr})?;\n"
            ));
        }
        "log_output" => {
            let text_expr = if let Some(&(src, sport)) = in_edges.get("text") {
                format!("&{}_{}", sanitize_ident(src), sanitize_ident(sport))
            } else {
                let t = node.config.get("text").cloned().unwrap_or(serde_json::json!(""));
                match t {
                    serde_json::Value::String(s) => format!("\"{s}\""),
                    other => format!("serde_json::json!({other})"),
                }
            };
            out.push_str(&format!("{indent}nodes::log_output({text_expr})?;\n"));
        }
        "json_parse" => {
            let json_expr = if let Some(&(src, sport)) = in_edges.get("json_str") {
                format!("&{}_{}", sanitize_ident(src), sanitize_ident(sport))
            } else {
                let j = node.config.get("json_str").and_then(|v| v.as_str()).unwrap_or("{}");
                format!("{j:?}")
            };
            let path_expr = if let Some(&(src, sport)) = in_edges.get("path") {
                format!("Some(&{}_{}[..])", sanitize_ident(src), sanitize_ident(sport))
            } else if let Some(p) = node.config.get("path").and_then(|v| v.as_str()) {
                format!("Some(\"{p}\")")
            } else {
                "None".to_string()
            };
            out.push_str(&format!(
                "{indent}let {id_ident}_value = nodes::json_parse({json_expr}, {path_expr})?;\n"
            ));
        }
        "text_clean" => {
            let text_expr = if let Some(&(src, sport)) = in_edges.get("text") {
                format!("&{}_{}", sanitize_ident(src), sanitize_ident(sport))
            } else {
                let t = node.config.get("text").and_then(|v| v.as_str()).unwrap_or("");
                format!("\"{t}\"")
            };
            let trim = node.config.get("trim").and_then(|v| v.as_bool()).unwrap_or(true);
            let collapse = node.config.get("remove_extra_whitespace").and_then(|v| v.as_bool()).unwrap_or(false);
            let to_lower = node.config.get("to_lower").and_then(|v| v.as_bool()).unwrap_or(false);
            let to_upper = node.config.get("to_upper").and_then(|v| v.as_bool()).unwrap_or(false);
            out.push_str(&format!(
                "{indent}let {id_ident}_cleaned = nodes::text_clean({text_expr}, {trim}, {collapse}, {to_lower}, {to_upper})?;\n"
            ));
        }
        "delay" => {
            let dur_expr = if let Some(&(src, sport)) = in_edges.get("duration") {
                format!("&{}_{}", sanitize_ident(src), sanitize_ident(sport))
            } else {
                let d = node.config.get("duration").and_then(|v| v.as_str()).unwrap_or("0ms");
                format!("\"{d}\"")
            };
            out.push_str(&format!("{indent}nodes::delay({dur_expr}).await?;\n"));
        }
        "string_to_path" => {
            let path_expr = if let Some(&(src, sport)) = in_edges.get("path") {
                format!("&{}_{}", sanitize_ident(src), sanitize_ident(sport))
            } else {
                let p = node.config.get("path").and_then(|v| v.as_str()).unwrap_or("");
                format!("\"{p}\"")
            };
            out.push_str(&format!(
                "{indent}let {id_ident}_path = nodes::string_to_path({path_expr})?;\n"
            ));
        }
        "json_to_string" => {
            let val_expr = if let Some(&(src, sport)) = in_edges.get("value") {
                format!("&{}_{}", sanitize_ident(src), sanitize_ident(sport))
            } else {
                let v = node.config.get("value").cloned().unwrap_or(serde_json::json!(null));
                format!("&serde_json::json!({v})")
            };
            out.push_str(&format!(
                "{indent}let {id_ident}_text = nodes::json_to_string({val_expr})?;\n"
            ));
        }
        "json_to_bool" => {
            let val_expr = if let Some(&(src, sport)) = in_edges.get("value") {
                format!("&{}_{}", sanitize_ident(src), sanitize_ident(sport))
            } else {
                let v = node.config.get("value").cloned().unwrap_or(serde_json::json!(false));
                format!("&serde_json::json!({v})")
            };
            out.push_str(&format!(
                "{indent}let {id_ident}_value = nodes::json_to_bool({val_expr})?;\n"
            ));
        }
        _ => {}
    }

    out
}

/// Generates `src/main.rs` with topological execution, branching if/else, and variable flow.
pub fn generate_main_rs(graph: &GraphSpec, topo_order: &[String]) -> Result<String> {
    let node_map: HashMap<&str, &lab_node_sdk::NodeInstance> =
        graph.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    // Check for condition nodes
    let conditions: Vec<&lab_node_sdk::NodeInstance> =
        graph.nodes.iter().filter(|n| n.kind == "condition").collect();

    let mut body = String::new();

    if conditions.is_empty() {
        // Pure linear DAG
        for id in topo_order {
            if let Some(node) = node_map.get(id.as_str()) {
                body.push_str(&generate_node_call(node, graph, "    "));
            }
        }
    } else {
        // Handle branching with reachability analysis for each condition
        // For simplicity with single condition:
        let cond = conditions[0];
        let true_target = cond.config.get("true_branch").and_then(|v| v.as_str()).unwrap_or("");
        let false_target = cond.config.get("false_branch").and_then(|v| v.as_str()).unwrap_or("");

        let reach_true = reachable_from(true_target, graph);
        let reach_false = reachable_from(false_target, graph);

        let true_exclusive: HashSet<_> = reach_true.difference(&reach_false).cloned().collect();
        let false_exclusive: HashSet<_> = reach_false.difference(&reach_true).cloned().collect();

        // 1. Nodes before condition
        let cond_idx = topo_order.iter().position(|id| id == &cond.id).unwrap_or(0);
        for id in &topo_order[..cond_idx] {
            if let Some(node) = node_map.get(id.as_str()) {
                body.push_str(&generate_node_call(node, graph, "    "));
            }
        }

        // 2. Condition evaluation
        let cond_val_expr = if let Some(edge) = graph.edges.iter().find(|e| e.to.node == cond.id && e.to.port == "value") {
            format!("{}_{}", sanitize_ident(&edge.from.node), sanitize_ident(&edge.from.port))
        } else {
            let b = cond.config.get("value").and_then(|v| v.as_bool()).unwrap_or(false);
            format!("{b}")
        };

        body.push_str(&format!(
            "    // 🌿 Branching Condition: '{}'\n    if {cond_val_expr} {{\n",
            cond.id
        ));

        // True branch nodes in topological order
        for id in topo_order {
            if true_exclusive.contains(id) {
                if let Some(node) = node_map.get(id.as_str()) {
                    body.push_str(&generate_node_call(node, graph, "        "));
                }
            }
        }

        body.push_str("    } else {\n");

        // False branch nodes in topological order
        for id in topo_order {
            if false_exclusive.contains(id) {
                if let Some(node) = node_map.get(id.as_str()) {
                    body.push_str(&generate_node_call(node, graph, "        "));
                }
            }
        }

        body.push_str("    }\n\n");

        // 3. Common / merge nodes after condition
        for id in &topo_order[cond_idx + 1..] {
            if !true_exclusive.contains(id) && !false_exclusive.contains(id) {
                if let Some(node) = node_map.get(id.as_str()) {
                    body.push_str(&generate_node_call(node, graph, "    "));
                }
            }
        }
    }

    let main_code = format!(
        r#"//! Auto-generated by Rust Lab Construction Mode.
#![allow(unused_variables, unused_mut)]
mod nodes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {{
    println!("🚀 Starting exported Rust Lab application...");

{body}
    println!("✅ Application finished successfully!");
    Ok(())
}}
"#
    );

    Ok(main_code)
}

/// Generates a complete standalone Cargo project from a graph.
pub fn export_project(graph: &GraphSpec, output_dir: &Path, project_name: &str) -> Result<()> {
    fs::create_dir_all(output_dir.join("src"))
        .with_context(|| format!("Failed to create output directory: {:?}", output_dir))?;

    let used_kinds: HashSet<&str> = graph.nodes.iter().map(|n| n.kind.as_str()).collect();
    let has_network = used_kinds.contains("http_get");

    // 1. Cargo.toml
    let cargo_toml = generate_cargo_toml(project_name, has_network);
    fs::write(output_dir.join("Cargo.toml"), cargo_toml)?;

    // 2. src/nodes.rs
    let nodes_rs = generate_nodes_rs(&used_kinds);
    fs::write(output_dir.join("src/nodes.rs"), nodes_rs)?;

    // 3. src/main.rs
    let order = crate::plan::execution_order(graph)?;
    let main_rs = generate_main_rs(graph, &order)?;
    fs::write(output_dir.join("src/main.rs"), main_rs)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn sanitizes_identifiers() {
        assert_eq!(sanitize_ident("welcome-user"), "welcome_user");
        assert_eq!(sanitize_ident("123node"), "_123node");
        assert_eq!(sanitize_ident("node.out"), "node_out");
    }

    #[test]
    fn generates_cargo_toml_with_conditional_reqwest() {
        let without_net = generate_cargo_toml("app1", false);
        assert!(!without_net.contains("reqwest"));
        assert!(without_net.contains("tokio"));

        let with_net = generate_cargo_toml("app2", true);
        assert!(with_net.contains("reqwest"));
    }

    #[test]
    fn reachability_analysis_separates_branches() {
        let graph: GraphSpec = serde_json::from_value(json!({
            "graph_version": "0.1",
            "nodes": [
                {"id": "if", "kind": "condition", "config": {"value": true, "true_branch": "yes", "false_branch": "no"}},
                {"id": "yes", "kind": "log_output", "config": {"text": "Y"}},
                {"id": "no", "kind": "log_output", "config": {"text": "N"}},
                {"id": "common", "kind": "log_output", "config": {"text": "C"}}
            ],
            "edges": []
        })).unwrap();

        let reach_yes = reachable_from("yes", &graph);
        assert!(reach_yes.contains("yes"));
        assert!(!reach_yes.contains("no"));

        let reach_no = reachable_from("no", &graph);
        assert!(reach_no.contains("no"));
        assert!(!reach_no.contains("yes"));
    }

    #[test]
    fn exports_full_working_cargo_project() {
        let graph: GraphSpec = serde_json::from_value(json!({
            "graph_version": "0.1",
            "nodes": [
                {"id": "parse", "kind": "json_parse", "config": {"json_str": "{\"msg\": \"hello world\"}", "path": "msg"}},
                {"id": "to_str", "kind": "json_to_string", "config": {}},
                {"id": "clean", "kind": "text_clean", "config": {"to_upper": true}},
                {"id": "print", "kind": "log_output", "config": {}}
            ],
            "edges": [
                {"from": {"node": "parse", "port": "value"}, "to": {"node": "to_str", "port": "value"}},
                {"from": {"node": "to_str", "port": "text"}, "to": {"node": "clean", "port": "text"}},
                {"from": {"node": "clean", "port": "cleaned"}, "to": {"node": "print", "port": "text"}}
            ]
        })).unwrap();

        let tmp_dir = std::env::temp_dir().join("rust_lab_e2e_test");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        export_project(&graph, &tmp_dir, "rust_lab_e2e_test").unwrap();

        assert!(tmp_dir.join("Cargo.toml").exists());
        assert!(tmp_dir.join("src/main.rs").exists());
        assert!(tmp_dir.join("src/nodes.rs").exists());

        let main_src = std::fs::read_to_string(tmp_dir.join("src/main.rs")).unwrap();
        assert!(main_src.contains("nodes::json_parse"));
        assert!(main_src.contains("nodes::json_to_string"));
        assert!(main_src.contains("nodes::text_clean"));
        assert!(main_src.contains("nodes::log_output"));

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }
}
