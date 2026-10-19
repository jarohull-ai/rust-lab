//! Execution planning shared by the validator and the executor.
//!
//! A graph has three kinds of dependencies:
//! * data edges declared in `graph.edges`;
//! * control edges implied by `condition` nodes (`condition → true_branch/false_branch`);
//! * explicit ordering edges from a node's `after` list (`dependency → node`).
//!
//! All of them are respected when ordering nodes, take part in cycle detection,
//! and propagate branch pruning (a node `after` a pruned node is pruned too).

use anyhow::{anyhow, Result};
use lab_node_sdk::GraphSpec;
use std::collections::{HashMap, HashSet, VecDeque};

pub const CONDITION_KIND: &str = "condition";
pub const BRANCH_KEYS: [&str; 2] = ["true_branch", "false_branch"];

/// Control edges `(condition_id, branch_id)` taken from condition node configs.
pub fn control_edges(graph: &GraphSpec) -> Vec<(String, String)> {
    graph
        .nodes
        .iter()
        .filter(|n| n.kind == CONDITION_KIND)
        .flat_map(|n| {
            BRANCH_KEYS.iter().filter_map(move |key| {
                n.config
                    .get(*key)
                    .and_then(|v| v.as_str())
                    .map(|target| (n.id.clone(), target.to_string()))
            })
        })
        .collect()
}

/// Ordering edges `(dependency_id, node_id)` taken from each node's `after` list.
pub fn after_edges(graph: &GraphSpec) -> Vec<(String, String)> {
    graph
        .nodes
        .iter()
        .flat_map(|n| n.after.iter().map(move |dep| (dep.clone(), n.id.clone())))
        .collect()
}

/// Adjacency list over data, condition and `after` edges.
pub fn successors(graph: &GraphSpec) -> HashMap<String, Vec<String>> {
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    let data = graph
        .edges
        .iter()
        .map(|e| (e.from.node.clone(), e.to.node.clone()));
    for (from, to) in data.chain(control_edges(graph)).chain(after_edges(graph)) {
        let list = adj.entry(from).or_default();
        if !list.contains(&to) {
            list.push(to);
        }
    }
    adj
}

/// Topological order that stays as close as possible to the order the author
/// wrote the nodes in (Kahn's algorithm, ties broken by declaration order).
pub fn execution_order(graph: &GraphSpec) -> Result<Vec<String>> {
    let position: HashMap<&str, usize> = graph
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.as_str(), i))
        .collect();
    let adj = successors(graph);

    let mut indegree: HashMap<&str, usize> = position.keys().map(|id| (*id, 0)).collect();
    for targets in adj.values() {
        for t in targets {
            *indegree
                .get_mut(t.as_str())
                .ok_or_else(|| anyhow!("Edge points to unknown node '{t}'"))? += 1;
        }
    }

    let mut ready: Vec<&str> = graph
        .nodes
        .iter()
        .map(|n| n.id.as_str())
        .filter(|id| indegree[id] == 0)
        .collect();
    let mut order = Vec::with_capacity(graph.nodes.len());

    while !ready.is_empty() {
        ready.sort_by_key(|id| std::cmp::Reverse(position[id]));
        let id = ready.pop().expect("non-empty");
        order.push(id.to_string());
        for t in adj.get(id).into_iter().flatten() {
            let d = indegree.get_mut(t.as_str()).expect("validated above");
            *d -= 1;
            if *d == 0 {
                ready.push(t.as_str());
            }
        }
    }

    if order.len() != graph.nodes.len() {
        let stuck: Vec<&str> = graph
            .nodes
            .iter()
            .map(|n| n.id.as_str())
            .filter(|id| !order.iter().any(|o| o == id))
            .collect();
        return Err(anyhow!(
            "Cycle detected involving nodes: {}",
            stuck.join(", ")
        ));
    }
    Ok(order)
}

/// All nodes reachable from `start` (including `start`).
pub fn reachable(adj: &HashMap<String, Vec<String>>, start: &str) -> HashSet<String> {
    let mut seen = HashSet::from([start.to_string()]);
    let mut queue = VecDeque::from([start.to_string()]);
    while let Some(id) = queue.pop_front() {
        for next in adj.get(&id).into_iter().flatten() {
            if seen.insert(next.clone()) {
                queue.push_back(next.clone());
            }
        }
    }
    seen
}

/// Nodes that belong exclusively to the branch that was not taken.
/// Nodes reachable from both branches (merge points) stay runnable.
pub fn pruned_branch(
    adj: &HashMap<String, Vec<String>>,
    taken: &str,
    skipped: &str,
) -> HashSet<String> {
    let keep = reachable(adj, taken);
    reachable(adj, skipped)
        .into_iter()
        .filter(|id| !keep.contains(id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn graph(value: serde_json::Value) -> GraphSpec {
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn data_edges_reorder_nodes() {
        let g = graph(json!({
            "graph_version": "0.1",
            "nodes": [
                {"id": "log", "kind": "log_output", "config": {}},
                {"id": "read", "kind": "fs_read", "config": {}}
            ],
            "edges": [{"from": {"node": "read", "port": "content"}, "to": {"node": "log", "port": "text"}}]
        }));
        assert_eq!(execution_order(&g).unwrap(), vec!["read", "log"]);
    }

    #[test]
    fn condition_branches_run_after_condition_and_prune_correctly() {
        let g = graph(json!({
            "graph_version": "0.1",
            "nodes": [
                {"id": "yes", "kind": "log_output", "config": {}},
                {"id": "no", "kind": "log_output", "config": {}},
                {"id": "merge", "kind": "log_output", "config": {}},
                {"id": "if", "kind": "condition", "config": {"value": true, "true_branch": "yes", "false_branch": "no"}}
            ],
            "edges": [
                {"from": {"node": "yes", "port": "x"}, "to": {"node": "merge", "port": "text"}},
                {"from": {"node": "no", "port": "x"}, "to": {"node": "merge", "port": "text"}}
            ]
        }));
        let order = execution_order(&g).unwrap();
        assert_eq!(order[0], "if");
        let pruned = pruned_branch(&successors(&g), "yes", "no");
        assert_eq!(pruned, HashSet::from(["no".to_string()]));
    }

    #[test]
    fn control_edges_participate_in_cycle_detection() {
        let g = graph(json!({
            "graph_version": "0.1",
            "nodes": [
                {"id": "if", "kind": "condition", "config": {"true_branch": "a", "false_branch": "a"}},
                {"id": "a", "kind": "log_output", "config": {}}
            ],
            "edges": [{"from": {"node": "a", "port": "x"}, "to": {"node": "if", "port": "value"}}]
        }));
        assert!(execution_order(&g).is_err());
    }
}
