use crate::nodes::condition::{NEXT_KEY, SKIP_KEY};
use crate::plan;
use crate::registry::NodeRegistry;
use anyhow::{anyhow, Context, Result};
use lab_node_sdk::{GraphSpec, NodeInput, NodeOutput};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct Executor {
    registry: Arc<NodeRegistry>,
}

/// What happened to each node during a run — handy for tests and future UI.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub executed: Vec<String>,
    pub skipped: Vec<String>,
    pub outputs: HashMap<String, NodeOutput>,
}

impl Executor {
    pub fn new(registry: NodeRegistry) -> Self {
        Self {
            registry: Arc::new(registry),
        }
    }

    pub async fn execute(&self, graph: GraphSpec) -> Result<ExecutionReport> {
        let mut policy = crate::seed::SeedPolicy::default_safe();
        if let Some(security) = &graph.security {
            if let Ok(extra) = serde_json::from_value::<crate::seed::SeedPolicy>(security.clone()) {
                policy.merge_with(&extra);
            }
        }
        policy.from_env();
        self.execute_with_policy(graph, &policy).await
    }

    pub async fn execute_with_policy(
        &self,
        graph: GraphSpec,
        policy: &crate::seed::SeedPolicy,
    ) -> Result<ExecutionReport> {
        // Topological order over data, condition and `after` edges: every node's
        // dependencies (including its condition, if it is a branch) come before it.
        let order = plan::execution_order(&graph)?;
        let adj = plan::successors(&graph);
        let nodes: HashMap<&str, _> = graph.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

        let mut report = ExecutionReport::default();
        let mut skipped: HashSet<String> = HashSet::new();
        let mut finished: HashSet<String> = HashSet::new();

        for id in order {
            if skipped.contains(&id) {
                println!("⏭️  Skipping Node: {id} (branch not taken)");
                finished.insert(id.clone());
                report.skipped.push(id);
                continue;
            }

            let node = nodes[id.as_str()];

            // `after` gate: every listed dependency must have finished (run or pruned).
            if let Some(dep) = node.after.iter().find(|dep| !finished.contains(*dep)) {
                return Err(anyhow!(
                    "Node '{id}' must run after '{dep}', which has not finished"
                ));
            }

            let impl_node = self
                .registry
                .get(&node.kind)
                .ok_or_else(|| anyhow!("Node implementation missing for {}", node.kind))?;
            let spec = impl_node.spec();
            println!("🚀 Executing Node: {} [{}]", node.id, node.kind);

            let inputs = self.collect_inputs(&graph, &id, &report.outputs)?;

            // SEED Policy Guard: evaluate before execution
            let verdict = crate::seed::evaluate_node_inputs(&node.kind, &inputs, policy);
            if let crate::seed::SeedVerdict::Denied { reason, rule } = verdict {
                return Err(anyhow!("SEED Veto [{rule}]: {reason}"));
            }

            for pin in spec.inputs.iter().filter(|p| p.required) {
                if inputs.get(&pin.name).is_none_or(Value::is_null) {
                    return Err(anyhow!(
                        "Node '{id}' [{}] is missing required input '{}'",
                        node.kind,
                        pin.name
                    ));
                }
            }

            let mut output = impl_node
                .execute(inputs)
                .await
                .with_context(|| format!("Node '{id}' [{}] failed", node.kind))?;

            // Control flow: `__next__` must be a pending node; `__skip__` gets pruned.
            let next = output.remove(NEXT_KEY);
            let skip = output.remove(SKIP_KEY);
            if let Some(next) = next {
                let next = next
                    .as_str()
                    .ok_or_else(|| anyhow!("'{NEXT_KEY}' from '{id}' must be a node id"))?
                    .to_string();
                if !nodes.contains_key(next.as_str()) || finished.contains(&next) {
                    return Err(anyhow!(
                        "'{id}' jumps to '{next}', which is unknown or already ran"
                    ));
                }
                if skipped.contains(&next) {
                    return Err(anyhow!(
                        "'{id}' jumps to '{next}', which was pruned earlier"
                    ));
                }
                if let Some(skip) = skip.as_ref().and_then(Value::as_str) {
                    skipped.extend(plan::pruned_branch(&adj, &next, skip));
                }
            }

            finished.insert(id.clone());
            report.executed.push(id.clone());
            report.outputs.insert(id, output);
        }

        Ok(report)
    }

    /// Config values first, then values arriving over edges (edges win).
    fn collect_inputs(
        &self,
        graph: &GraphSpec,
        id: &str,
        results: &HashMap<String, NodeOutput>,
    ) -> Result<NodeInput> {
        let node = graph
            .nodes
            .iter()
            .find(|n| n.id == id)
            .ok_or_else(|| anyhow!("Unknown node '{id}'"))?;

        let mut inputs: NodeInput = node
            .config
            .as_object()
            .map(|obj| {
                let mut map = HashMap::new();
                for (k, v) in obj {
                    let k_lower = k.to_lowercase();
                    map.insert(k.clone(), v.clone());
                    map.insert(k_lower.clone(), v.clone());
                    if k_lower == "file_path" || k_lower == "filepath" {
                        map.insert("path".to_string(), v.clone());
                    }
                }
                map
            })
            .unwrap_or_default();

        for edge in graph.edges.iter().filter(|e| e.to.node == id) {
            // A source that was skipped (or produced no such port) simply contributes nothing;
            // the required-input check reports it if the pin was mandatory.
            if let Some(val) = results.get(&edge.from.node).and_then(|o| {
                o.get(&edge.from.port)
                    .or_else(|| o.get(&edge.from.port.to_lowercase()))
            }) {
                inputs.insert(edge.to.port.clone(), val.clone());
                inputs.insert(edge.to.port.to_lowercase(), val.clone());
                if edge.to.port.to_lowercase() == "file_path"
                    || edge.to.port.to_lowercase() == "filepath"
                {
                    inputs.insert("path".to_string(), val.clone());
                }
            }
        }
        Ok(inputs)
    }
}

/// Convenience helper to execute a JSON graph value with a specified policy.
pub async fn execute_graph(
    graph: &Value,
    registry: &NodeRegistry,
    policy: &crate::seed::SeedPolicy,
) -> Result<ExecutionReport> {
    let spec: GraphSpec = serde_json::from_value(graph.clone())
        .map_err(|e| anyhow!("Failed to deserialize graph: {e}"))?;
    let executor = Executor::new(registry.clone());
    executor.execute_with_policy(spec, policy).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes;
    use serde_json::json;

    fn executor() -> Executor {
        let mut registry = NodeRegistry::new();
        nodes::register_all(&mut registry);
        Executor::new(registry)
    }

    fn graph(value: Value) -> GraphSpec {
        serde_json::from_value(value).unwrap()
    }

    #[tokio::test]
    async fn edges_carry_values_through_pipeline() {
        let g = graph(json!({
            "graph_version": "0.1",
            "nodes": [
                {"id": "clean", "kind": "text_clean", "config": {"to_upper": true}},
                {"id": "parse", "kind": "json_parse", "config": {"json_str": "{\"msg\":\"  hi there \"}", "path": "msg"}}
            ],
            "edges": [{"from": {"node": "parse", "port": "value"}, "to": {"node": "clean", "port": "text"}}]
        }));
        let report = executor().execute(g).await.unwrap();
        assert_eq!(report.executed, vec!["parse", "clean"]);
        assert_eq!(report.outputs["clean"]["cleaned"], json!("HI THERE"));
    }

    #[tokio::test]
    async fn condition_runs_one_branch_and_keeps_merge_node() {
        for (value, taken, pruned) in [(true, "yes", "no"), (false, "no", "yes")] {
            let g = graph(json!({
                "graph_version": "0.1",
                "nodes": [
                    {"id": "if", "kind": "condition", "config": {"value": value, "true_branch": "yes", "false_branch": "no"}},
                    {"id": "no", "kind": "delay", "config": {"duration": "1ms"}},
                    {"id": "yes", "kind": "delay", "config": {"duration": "1ms"}},
                    {"id": "after", "kind": "log_output", "config": {"text": "done"}}
                ],
                "edges": []
            }));
            let report = executor().execute(g).await.unwrap();
            assert_eq!(report.executed, vec!["if", taken, "after"]);
            assert_eq!(report.skipped, vec![pruned]);
        }
    }

    #[tokio::test]
    async fn log_output_with_after_runs_after_delay_finishes() {
        // `log-1` is declared first, but `after` forces it behind `delay-1`.
        let g = graph(json!({
            "graph_version": "0.1",
            "nodes": [
                {"id": "log-1", "kind": "log_output", "config": {"text": "po delay"}, "after": ["delay-1"]},
                {"id": "delay-1", "kind": "delay", "config": {"duration": "30ms"}}
            ],
            "edges": []
        }));
        let started = std::time::Instant::now();
        let report = executor().execute(g).await.unwrap();
        assert_eq!(report.executed, vec!["delay-1", "log-1"]);
        assert!(started.elapsed() >= std::time::Duration::from_millis(30));
    }

    #[tokio::test]
    async fn node_after_untaken_branch_is_pruned() {
        let g = graph(json!({
            "graph_version": "0.1",
            "nodes": [
                {"id": "if", "kind": "condition", "config": {"value": true, "true_branch": "yes", "false_branch": "wait"}},
                {"id": "yes", "kind": "log_output", "config": {"text": "admin"}},
                {"id": "wait", "kind": "delay", "config": {"duration": "1ms"}},
                {"id": "after-wait", "kind": "log_output", "config": {"text": "user"}, "after": ["wait"]}
            ],
            "edges": []
        }));
        let report = executor().execute(g).await.unwrap();
        assert_eq!(report.executed, vec!["if", "yes"]);
        assert_eq!(report.skipped, vec!["wait", "after-wait"]);
    }

    #[tokio::test]
    async fn missing_required_input_is_reported() {
        let g = graph(json!({
            "graph_version": "0.1",
            "nodes": [{"id": "clean", "kind": "text_clean", "config": {}}],
            "edges": []
        }));
        let err = executor().execute(g).await.unwrap_err().to_string();
        assert!(err.contains("missing required input 'text'"), "{err}");
    }

    #[tokio::test]
    async fn converter_nodes_pipe_json_to_bool_and_string_to_path() {
        let g = graph(json!({
            "graph_version": "0.1",
            "nodes": [
                {"id": "parse", "kind": "json_parse", "config": {"json_str": "{\"flag\": true, \"filename\": \"output.txt\"}", "path": "flag"}},
                {"id": "to_bool", "kind": "json_to_bool", "config": {}},
                {"id": "cond", "kind": "condition", "config": {"true_branch": "str_path", "false_branch": "skip_node"}},
                {"id": "str_path", "kind": "string_to_path", "config": {"path": "data/converted.txt"}},
                {"id": "skip_node", "kind": "log_output", "config": {"text": "skipped"}},
                {"id": "write", "kind": "fs_write", "config": {"content": "hello from converter"}, "after": ["str_path"]}
            ],
            "edges": [
                {"from": {"node": "parse", "port": "value"}, "to": {"node": "to_bool", "port": "value"}},
                {"from": {"node": "to_bool", "port": "value"}, "to": {"node": "cond", "port": "value"}},
                {"from": {"node": "str_path", "port": "path"}, "to": {"node": "write", "port": "path"}}
            ]
        }));
        let mut reg = NodeRegistry::new();
        crate::nodes::register_all(&mut reg);
        crate::validator::Validator::new(&reg).validate(&g).unwrap();

        let report = executor().execute(g).await.unwrap();
        assert!(report.executed.contains(&"str_path".to_string()));
        assert!(report.executed.contains(&"write".to_string()));
        assert!(report.skipped.contains(&"skip_node".to_string()));
        let _ = std::fs::remove_file("data/converted.txt");
    }

    #[tokio::test]
    async fn seed_veto_rejects_evil_domain_http_get() {
        let g = graph(json!({
            "graph_version": "0.1",
            "nodes": [
                {"id": "fetch", "kind": "http_get", "config": {"url": "https://evil.example.com/exfiltrate"}}
            ],
            "edges": []
        }));
        let err = executor().execute(g).await.unwrap_err().to_string();
        assert!(err.contains("SEED Veto"), "{err}");
        assert!(err.contains("http_get.domain_allowlist"), "{err}");
    }

    #[tokio::test]
    async fn seed_veto_rejects_absolute_path_fs_write() {
        let g = graph(json!({
            "graph_version": "0.1",
            "nodes": [
                {"id": "write", "kind": "fs_write", "config": {"path": "/etc/shadow", "content": "bad"}}
            ],
            "edges": []
        }));
        let err = executor().execute(g).await.unwrap_err().to_string();
        assert!(err.contains("SEED Veto"), "{err}");
        assert!(err.contains("fs_write.absolute_path"), "{err}");
    }

    #[tokio::test]
    async fn seed_veto_rejects_denied_read_path_fs_read() {
        let g = graph(json!({
            "graph_version": "0.1",
            "nodes": [
                {"id": "read", "kind": "fs_read", "config": {"path": "/etc/passwd"}}
            ],
            "edges": []
        }));
        let err = executor().execute(g).await.unwrap_err().to_string();
        assert!(err.contains("SEED Veto"), "{err}");
        assert!(err.contains("fs_read.path_denylist"), "{err}");
    }

    #[tokio::test]
    async fn seed_policy_allows_extended_domain_from_graph_security() {
        let g = graph(json!({
            "graph_version": "0.1",
            "security": {
                "allowed_domains": ["custom-partner-api.org"]
            },
            "nodes": [
                {"id": "fetch", "kind": "http_get", "config": {"url": "https://custom-partner-api.org/data"}}
            ],
            "edges": []
        }));
        // Since custom-partner-api.org won't resolve in sandbox, we just verify SEED didn't veto it!
        let err = executor().execute(g).await.unwrap_err().to_string();
        assert!(
            !err.contains("SEED Veto"),
            "SEED should not have vetoed allowed domain: {err}"
        );
    }
}
