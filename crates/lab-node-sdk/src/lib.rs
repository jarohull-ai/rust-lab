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
}

pub type NodeInput = HashMap<String, Value>;
pub type NodeOutput = HashMap<String, Value>;

#[async_trait]
pub trait ViperNode: Send + Sync {
    fn spec(&self) -> NodeSpec;
    async fn execute(&self, input: NodeInput) -> Result<NodeOutput, anyhow::Error>;
}
