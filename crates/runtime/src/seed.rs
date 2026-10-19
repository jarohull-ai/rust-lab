//! SEED Policy Guard – deterministyczne reguły bezpieczeństwa
//! uruchamiane PRZED wykonaniem node'a.
//!
//! Zapewnia, że `http_get` nie wyjdzie na nieznaną domenę,
//! `fs_write` nie zapisze poza workspace, a `fs_read` nie
//! odczyta plików wrażliwych.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Wynik werdyktu SEED.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeedVerdict {
    Allowed,
    Denied { reason: String, rule: String },
}

impl SeedVerdict {
    pub fn is_denied(&self) -> bool {
        matches!(self, SeedVerdict::Denied { .. })
    }

    pub fn reason(&self) -> Option<&str> {
        match self {
            SeedVerdict::Allowed => None,
            SeedVerdict::Denied { reason, .. } => Some(reason.as_str()),
        }
    }

    pub fn rule(&self) -> Option<&str> {
        match self {
            SeedVerdict::Allowed => None,
            SeedVerdict::Denied { rule, .. } => Some(rule.as_str()),
        }
    }
}

/// Polityka bezpieczeństwa – konfigurowalna przez `graph.json` lub env.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeedPolicy {
    /// Allowlista domen dla `http_get` (uzupełniana z `RUST_LAB_ALLOWED_DOMAINS`).
    #[serde(default)]
    pub allowed_domains: Vec<String>,
    /// Allowlista katalogów dla `fs_write` (domyślnie: `data/`, `output/`).
    #[serde(default)]
    pub allowed_write_dirs: Vec<String>,
    /// Denylist ścieżek dla `fs_read` (domyślnie: `/etc`, `/root`, `.ssh`, `.env`, `.git`).
    #[serde(default)]
    pub denied_read_paths: Vec<String>,
}

impl Default for SeedPolicy {
    fn default() -> Self {
        Self::default_safe()
    }
}

impl SeedPolicy {
    /// Domyślna polityka – bezpieczna dla uczniów.
    pub fn default_safe() -> Self {
        Self {
            allowed_domains: vec![
                "httpbin.org".into(),
                "api.github.com".into(),
                "hnrss.org".into(),
                "jsonplaceholder.typicode.com".into(),
                "raw.githubusercontent.com".into(),
                "localhost".into(),
                "127.0.0.1".into(),
            ],
            allowed_write_dirs: vec!["data/".into(), "output/".into()],
            denied_read_paths: vec![
                "/etc".into(),
                "/root".into(),
                ".ssh".into(),
                ".env".into(),
                ".git".into(),
            ],
        }
    }

    /// Rozszerza politykę o wartości z `graph.json` (`security`).
    pub fn merge_with(&mut self, extra: &SeedPolicy) {
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
        for path in &extra.denied_read_paths {
            if !self.denied_read_paths.contains(path) {
                self.denied_read_paths.push(path.clone());
            }
        }
    }

    /// Rozszerza politykę o zmienną środowiskową `RUST_LAB_ALLOWED_DOMAINS`.
    pub fn from_env(&mut self) {
        if let Ok(extra) = std::env::var("RUST_LAB_ALLOWED_DOMAINS") {
            for domain in extra.split(',') {
                let d = domain.trim();
                if !d.is_empty() && !self.allowed_domains.contains(&d.to_string()) {
                    self.allowed_domains.push(d.into());
                }
            }
        }
    }
}

/// Ewaluacja node'a przed wykonaniem (przyjmuje `serde_json::Value`).
pub fn evaluate_node(
    node_kind: &str,
    params: &Value,
    policy: &SeedPolicy,
) -> SeedVerdict {
    let mut map = HashMap::new();
    if let Some(obj) = params.as_object() {
        for (k, v) in obj {
            map.insert(k.clone(), v.clone());
            map.insert(k.to_lowercase(), v.clone());
        }
    }
    evaluate_node_inputs(node_kind, &map, policy)
}

/// Ewaluacja node'a z wejściami `HashMap<String, Value>` zebranymi przez executor.
pub fn evaluate_node_inputs(
    node_kind: &str,
    inputs: &HashMap<String, Value>,
    policy: &SeedPolicy,
) -> SeedVerdict {
    match node_kind {
        "http_get" => evaluate_http_get(inputs, policy),
        "fs_write" => evaluate_fs_write(inputs, policy),
        "fs_read" => evaluate_fs_read(inputs, policy),
        _ => SeedVerdict::Allowed,
    }
}

fn evaluate_http_get(params: &HashMap<String, Value>, policy: &SeedPolicy) -> SeedVerdict {
    let url = match params.get("url").and_then(|v| v.as_str()) {
        Some(u) => u,
        None => return SeedVerdict::Allowed, // brak URL – walidacja złapie to wcześniej
    };

    // Parsowanie hosta z URL (bez konieczności zewnętrznego crate'a)
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
        return SeedVerdict::Allowed;
    }

    if policy
        .allowed_domains
        .iter()
        .any(|d| host == d || host.ends_with(&format!(".{d}")))
    {
        SeedVerdict::Allowed
    } else {
        SeedVerdict::Denied {
            reason: format!("Domain '{host}' is not in allowlist"),
            rule: "http_get.domain_allowlist".into(),
        }
    }
}

fn evaluate_fs_write(params: &HashMap<String, Value>, policy: &SeedPolicy) -> SeedVerdict {
    let path = match params
        .get("path")
        .or_else(|| params.get("file_path"))
        .and_then(|v| v.as_str())
    {
        Some(p) => p,
        None => return SeedVerdict::Allowed,
    };

    // Zawsze blokuj ścieżki absolutne lub próby ucieczki (..) poza workspace
    if path.starts_with('/') || path.starts_with('~') || path.contains("..") {
        return SeedVerdict::Denied {
            reason: format!("Absolute path or traversal '{path}' is not allowed"),
            rule: "fs_write.absolute_path".into(),
        };
    }

    // Sprawdź, czy ścieżka zaczyna się od dozwolonego katalogu
    let norm_path = path.trim_start_matches("./");
    if policy.allowed_write_dirs.iter().any(|d| {
        let norm_d = d.trim_start_matches("./");
        norm_path.starts_with(norm_d)
    }) {
        SeedVerdict::Allowed
    } else {
        SeedVerdict::Denied {
            reason: format!(
                "Path '{path}' is not in allowed write dirs: {:?}",
                policy.allowed_write_dirs
            ),
            rule: "fs_write.dir_allowlist".into(),
        }
    }
}

fn evaluate_fs_read(params: &HashMap<String, Value>, policy: &SeedPolicy) -> SeedVerdict {
    let path = match params
        .get("path")
        .or_else(|| params.get("file_path"))
        .and_then(|v| v.as_str())
    {
        Some(p) => p,
        None => return SeedVerdict::Allowed,
    };

    if policy.denied_read_paths.iter().any(|d| path.contains(d)) {
        SeedVerdict::Denied {
            reason: format!("Path '{path}' contains denied pattern"),
            rule: "fs_read.path_denylist".into(),
        }
    } else {
        SeedVerdict::Allowed
    }
}

// ============================================================================
// Testy jednostkowe
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn policy() -> SeedPolicy {
        let mut p = SeedPolicy::default_safe();
        p.from_env();
        p
    }

    #[test]
    fn allows_known_domain() {
        let p = policy();
        let v = evaluate_node("http_get", &json!({"url": "https://api.github.com/zen"}), &p);
        assert_eq!(v, SeedVerdict::Allowed);
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
        if let SeedVerdict::Denied { rule, reason } = v {
            assert_eq!(rule, "http_get.domain_allowlist");
            assert!(reason.contains("evil.example.com"));
        }
    }

    #[test]
    fn allows_fs_write_in_data_dir() {
        let p = policy();
        let v = evaluate_node("fs_write", &json!({"path": "data/output.txt"}), &p);
        assert_eq!(v, SeedVerdict::Allowed);
    }

    #[test]
    fn rejects_fs_write_to_etc() {
        let p = policy();
        let v = evaluate_node("fs_write", &json!({"path": "/etc/hosts"}), &p);
        assert!(v.is_denied(), "expected denied, got {:?}", v);
        if let SeedVerdict::Denied { rule, .. } = v {
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
        if let SeedVerdict::Denied { rule, .. } = v {
            assert_eq!(rule, "fs_read.path_denylist");
        }
    }

    #[test]
    fn allows_unknown_node_kind() {
        let p = policy();
        let v = evaluate_node("log_output", &json!({}), &p);
        assert_eq!(v, SeedVerdict::Allowed);
    }
}
