use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PinType {
    String,
    Number,
    Boolean,
    Json,
    HttpResponse,
    FilePath,
    Any,
}

impl std::fmt::Display for PinType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PinType::String => write!(f, "string"),
            PinType::Number => write!(f, "number"),
            PinType::Boolean => write!(f, "boolean"),
            PinType::Json => write!(f, "json"),
            PinType::HttpResponse => write!(f, "http_response"),
            PinType::FilePath => write!(f, "file_path"),
            PinType::Any => write!(f, "any"),
        }
    }
}

impl PinType {
    /// Checks type compatibility from source (self) to destination (target).
    /// Strict typing:
    /// - Same type is always compatible.
    /// - `Any` is compatible with all types in either direction.
    /// - `FilePath` is strictly NOT compatible with `String` (explicit conversion required).
    /// - All other type pairs are incompatible.
    pub fn is_compatible_with(&self, target: &PinType) -> bool {
        if self == target {
            return true;
        }
        if *self == PinType::Any || *target == PinType::Any {
            return true;
        }
        false
    }

    /// Checks if a static JSON value in a node's `config` conforms to this pin type.
    pub fn matches_value(&self, value: &Value) -> bool {
        match self {
            PinType::String => value.is_string(),
            PinType::Number => value.is_number(),
            PinType::Boolean => value.is_boolean(),
            PinType::Json => true,
            PinType::HttpResponse => value.is_object() || value.is_string(),
            PinType::FilePath => value.is_string(),
            PinType::Any => true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypedPin {
    pub name: String,
    pub pin_type: PinType,
    pub required: bool,
    pub description: String,
}

/// The blueprint of a node (how it's defined in the library).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSpec {
    pub name: String,
    pub version: String,
    pub description: String,
    pub inputs: Vec<TypedPin>,
    pub outputs: Vec<TypedPin>,
    #[serde(default)]
    pub requires_network: bool,
    #[serde(default)]
    pub requires_fs_write: bool,
}

/// The instance of a node in a specific graph.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeInstance {
    pub id: String,
    pub kind: String,
    #[serde(default, alias = "params")]
    pub config: Value,
    /// Control dependencies: ids of nodes that must finish before this one runs.
    /// Used for ordering after nodes without outputs (e.g. `delay`, `log_output`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub after: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Port {
    pub node: String,
    pub port: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Edge {
    pub from: Port,
    pub to: Port,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GraphSpec {
    pub graph_version: String,
    pub nodes: Vec<NodeInstance>,
    pub edges: Vec<Edge>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security: Option<Value>,
}

pub type NodeInput = HashMap<String, Value>;
pub type NodeOutput = HashMap<String, Value>;

#[async_trait]
pub trait ViperNode: Send + Sync {
    fn spec(&self) -> NodeSpec;
    async fn execute(&self, input: NodeInput) -> Result<NodeOutput, anyhow::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn string_is_compatible_with_string() {
        assert!(PinType::String.is_compatible_with(&PinType::String));
    }

    #[test]
    fn any_is_compatible_with_string() {
        assert!(PinType::Any.is_compatible_with(&PinType::String));
    }

    #[test]
    fn string_is_compatible_with_any() {
        assert!(PinType::String.is_compatible_with(&PinType::Any));
    }

    #[test]
    fn file_path_is_not_compatible_with_string() {
        assert!(!PinType::FilePath.is_compatible_with(&PinType::String));
        assert!(!PinType::String.is_compatible_with(&PinType::FilePath));
    }

    #[test]
    fn json_is_not_compatible_with_bool() {
        assert!(!PinType::Json.is_compatible_with(&PinType::Boolean));
    }

    #[test]
    fn matches_value_validates_json_types() {
        assert!(PinType::String.matches_value(&json!("hello")));
        assert!(!PinType::String.matches_value(&json!(123)));

        assert!(PinType::Boolean.matches_value(&json!(true)));
        assert!(!PinType::Boolean.matches_value(&json!("true")));

        assert!(PinType::Number.matches_value(&json!(42)));
        assert!(!PinType::Number.matches_value(&json!("42")));

        assert!(PinType::FilePath.matches_value(&json!("/path/to/file")));
        assert!(!PinType::FilePath.matches_value(&json!(123)));

        assert!(PinType::Any.matches_value(&json!("anything")));
        assert!(PinType::Any.matches_value(&json!(999)));
        assert!(PinType::Any.matches_value(&json!(null)));
    }
}

