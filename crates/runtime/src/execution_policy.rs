//! Deterministic safety rules evaluated before node execution.
//!
//! It prevents `http_get` from reaching unknown domains, restricts `fs_write`
//! to configured directories, and denies sensitive `fs_read` paths.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Result of an execution policy evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyVerdict {
    Allowed,
    Denied { reason: String, rule: String },
}

impl PolicyVerdict {
    pub fn is_denied(&self) -> bool {
        matches!(self, PolicyVerdict::Denied { .. })
    }

    pub fn reason(&self) -> Option<&str> {
        match self {
            PolicyVerdict::Allowed => None,
            PolicyVerdict::Denied { reason, .. } => Some(reason.as_str()),
        }
    }

    pub fn rule(&self) -> Option<&str> {
        match self {
            PolicyVerdict::Allowed => None,
            PolicyVerdict::Denied { rule, .. } => Some(rule.as_str()),
        }
    }
}

/// Safety policy configurable through `graph.json` or the environment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionPolicy {
    /// Domain allowlist for `http_get`, extended by `RUST_LAB_ALLOWED_DOMAINS`.
    #[serde(default)]
    pub allowed_domains: Vec<String>,
    /// Directory allowlist for `fs_write` (defaults to `data/` and `output/`).
    #[serde(default)]
    pub allowed_write_dirs: Vec<String>,
    /// Directory allowlist for `fs_read` (defaults to local learning content).
    #[serde(default)]
    pub allowed_read_dirs: Vec<String>,
    /// Path denylist for `fs_read`.
    #[serde(default)]
    pub denied_read_paths: Vec<String>,
}

impl Default for ExecutionPolicy {
    fn default() -> Self {
        Self::default_safe()
    }
}

impl ExecutionPolicy {
    /// Default policy intended for a local learning environment.
    pub fn default_safe() -> Self {
        Self {
            allowed_domains: vec![
                "httpbin.org".into(),
                "api.github.com".into(),
                "hnrss.org".into(),
                "jsonplaceholder.typicode.com".into(),
                "raw.githubusercontent.com".into(),
            ],
            allowed_write_dirs: vec!["data/".into(), "output/".into()],
            allowed_read_dirs: vec!["data/".into(), "examples/".into()],
            denied_read_paths: vec![
                "/etc".into(),
                "/root".into(),
                ".ssh".into(),
                ".env".into(),
                ".git".into(),
            ],
        }
    }

    /// Extends the policy with values from the graph's `security` object.
    pub fn merge_with(&mut self, extra: &ExecutionPolicy) {
        for domain in &extra.allowed_domains {
            if !self.allowed_domains.contains(domain) {
                self.allowed_domains.push(domain.clone());
            }
        }
        for dir in &extra.allowed_write_dirs {
            if !self.allowed_write_dirs.contains(dir) {
                self.allowed_write_dirs.push(dir.clone());
            }
        }
        for dir in &extra.allowed_read_dirs {
            if !self.allowed_read_dirs.contains(dir) {
                self.allowed_read_dirs.push(dir.clone());
            }
        }
        for path in &extra.denied_read_paths {
            if !self.denied_read_paths.contains(path) {
                self.denied_read_paths.push(path.clone());
            }
        }
    }

    /// Extends the domain allowlist from `RUST_LAB_ALLOWED_DOMAINS`.
    pub fn from_env(&mut self) {
        if let Ok(extra) = std::env::var("RUST_LAB_ALLOWED_DOMAINS") {
            for domain in extra.split(',') {
                let d = domain.trim();
                if !d.is_empty() && !self.allowed_domains.contains(&d.to_string()) {
                    self.allowed_domains.push(d.into());
                }
            }
        }
        if let Ok(extra) = std::env::var("RUST_LAB_ALLOWED_READ_DIRS") {
            for dir in extra
                .split(',')
                .map(str::trim)
                .filter(|dir| !dir.is_empty())
            {
                if !self.allowed_read_dirs.iter().any(|allowed| allowed == dir) {
                    self.allowed_read_dirs.push(dir.into());
                }
            }
        }
        if let Ok(extra) = std::env::var("RUST_LAB_ALLOWED_WRITE_DIRS") {
            for dir in extra
                .split(',')
                .map(str::trim)
                .filter(|dir| !dir.is_empty())
            {
                if !self.allowed_write_dirs.iter().any(|allowed| allowed == dir) {
                    self.allowed_write_dirs.push(dir.into());
                }
            }
        }
    }
}

/// Evaluates a node configuration before execution.
pub fn evaluate_node(node_kind: &str, params: &Value, policy: &ExecutionPolicy) -> PolicyVerdict {
    let mut map = HashMap::new();
    if let Some(obj) = params.as_object() {
        for (k, v) in obj {
            map.insert(k.clone(), v.clone());
            map.insert(k.to_lowercase(), v.clone());
        }
    }
    evaluate_node_inputs(node_kind, &map, policy)
}

/// Evaluates the node inputs collected by the executor.
pub fn evaluate_node_inputs(
    node_kind: &str,
    inputs: &HashMap<String, Value>,
    policy: &ExecutionPolicy,
) -> PolicyVerdict {
    match node_kind {
        "http_get" => evaluate_http_get(inputs, policy),
        "fs_write" => evaluate_fs_write(inputs, policy),
        "fs_read" => evaluate_fs_read(inputs, policy),
        _ => PolicyVerdict::Allowed,
    }
}

fn evaluate_http_get(params: &HashMap<String, Value>, policy: &ExecutionPolicy) -> PolicyVerdict {
    let url = match params.get("url").and_then(|v| v.as_str()) {
        Some(u) => u,
        None => return PolicyVerdict::Allowed, // Input validation reports the missing URL.
    };

    // Extract the host without adding another dependency.
    let host = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("");

    if host.is_empty() {
        return PolicyVerdict::Allowed;
    }

    if policy
        .allowed_domains
        .iter()
        .any(|d| host == d || host.ends_with(&format!(".{d}")))
    {
        PolicyVerdict::Allowed
    } else {
        PolicyVerdict::Denied {
            reason: format!("Domain '{host}' is not in allowlist"),
            rule: "http_get.domain_allowlist".into(),
        }
    }
}

fn evaluate_fs_write(params: &HashMap<String, Value>, policy: &ExecutionPolicy) -> PolicyVerdict {
    let path = match params
        .get("path")
        .or_else(|| params.get("file_path"))
        .and_then(|v| v.as_str())
    {
        Some(p) => p,
        None => return PolicyVerdict::Allowed,
    };

    // Always reject absolute paths and parent-directory traversal.
    if path.starts_with('/') || path.starts_with('~') || path.contains("..") {
        return PolicyVerdict::Denied {
            reason: format!("Absolute path or traversal '{path}' is not allowed"),
            rule: "fs_write.absolute_path".into(),
        };
    }

    // Require the path to start with an allowed directory.
    let norm_path = path.trim_start_matches("./");
    if policy.allowed_write_dirs.iter().any(|d| {
        let norm_d = d.trim_start_matches("./");
        norm_path.starts_with(norm_d)
    }) {
        PolicyVerdict::Allowed
    } else {
        PolicyVerdict::Denied {
            reason: format!(
                "Path '{path}' is not in allowed write dirs: {:?}",
                policy.allowed_write_dirs
            ),
            rule: "fs_write.dir_allowlist".into(),
        }
    }
}

fn evaluate_fs_read(params: &HashMap<String, Value>, policy: &ExecutionPolicy) -> PolicyVerdict {
    let path = match params
        .get("path")
        .or_else(|| params.get("file_path"))
        .and_then(|v| v.as_str())
    {
        Some(p) => p,
        None => return PolicyVerdict::Allowed,
    };

    if path.starts_with('/') || path.starts_with('~') || path.contains("..") {
        return PolicyVerdict::Denied {
            reason: format!("Absolute path or traversal '{path}' is not allowed"),
            rule: "fs_read.absolute_path".into(),
        };
    }

    if policy.denied_read_paths.iter().any(|d| path.contains(d)) {
        return PolicyVerdict::Denied {
            reason: format!("Path '{path}' contains denied pattern"),
            rule: "fs_read.path_denylist".into(),
        };
    }

    let normalized = path.trim_start_matches("./");
    if policy.allowed_read_dirs.iter().any(|directory| {
        let allowed = directory.trim_start_matches("./");
        normalized.starts_with(allowed)
    }) {
        PolicyVerdict::Allowed
    } else {
        PolicyVerdict::Denied {
            reason: format!(
                "Path '{path}' is not in allowed read dirs: {:?}",
                policy.allowed_read_dirs
            ),
            rule: "fs_read.dir_allowlist".into(),
        }
    }
}

// ============================================================================
// Unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn policy() -> ExecutionPolicy {
        let mut p = ExecutionPolicy::default_safe();
        p.from_env();
        p
    }

    #[test]
    fn allows_known_domain() {
        let p = policy();
        let v = evaluate_node(
            "http_get",
            &json!({"url": "https://api.github.com/zen"}),
            &p,
        );
        assert_eq!(v, PolicyVerdict::Allowed);
    }

    #[test]
    fn rejects_unknown_domain() {
        let p = policy();
        let v = evaluate_node(
            "http_get",
            &json!({"url": "https://evil.example.com/exfil"}),
            &p,
        );
        assert!(v.is_denied(), "expected denied, got {:?}", v);
        if let PolicyVerdict::Denied { rule, reason } = v {
            assert_eq!(rule, "http_get.domain_allowlist");
            assert!(reason.contains("evil.example.com"));
        }
    }

    #[test]
    fn allows_fs_write_in_data_dir() {
        let p = policy();
        let v = evaluate_node("fs_write", &json!({"path": "data/output.txt"}), &p);
        assert_eq!(v, PolicyVerdict::Allowed);
    }

    #[test]
    fn rejects_fs_write_to_etc() {
        let p = policy();
        let v = evaluate_node("fs_write", &json!({"path": "/etc/hosts"}), &p);
        assert!(v.is_denied(), "expected denied, got {:?}", v);
        if let PolicyVerdict::Denied { rule, .. } = v {
            assert_eq!(rule, "fs_write.absolute_path");
        }
    }

    #[test]
    fn rejects_fs_write_traversal() {
        let p = policy();
        let v = evaluate_node("fs_write", &json!({"path": "data/../../secret.txt"}), &p);
        assert!(v.is_denied(), "expected denied, got {:?}", v);
    }

    #[test]
    fn rejects_fs_read_passwd() {
        let p = policy();
        let v = evaluate_node("fs_read", &json!({"path": "/etc/passwd"}), &p);
        assert!(v.is_denied(), "expected denied, got {:?}", v);
        if let PolicyVerdict::Denied { rule, .. } = v {
            assert_eq!(rule, "fs_read.absolute_path");
        }
    }

    #[test]
    fn allows_fs_read_from_data_dir() {
        let p = policy();
        let v = evaluate_node("fs_read", &json!({"path": "data/input.txt"}), &p);
        assert_eq!(v, PolicyVerdict::Allowed);
    }

    #[test]
    fn rejects_fs_read_outside_allowed_dirs() {
        let p = policy();
        let v = evaluate_node("fs_read", &json!({"path": "notes/private.txt"}), &p);
        assert!(v.is_denied(), "expected denied, got {:?}", v);
        if let PolicyVerdict::Denied { rule, .. } = v {
            assert_eq!(rule, "fs_read.dir_allowlist");
        }
    }

    #[test]
    fn allows_unknown_node_kind() {
        let p = policy();
        let v = evaluate_node("log_output", &json!({}), &p);
        assert_eq!(v, PolicyVerdict::Allowed);
    }
}
