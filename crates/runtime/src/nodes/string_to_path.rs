use anyhow::{anyhow, Result};
use async_trait::async_trait;
use lab_node_sdk::{NodeInput, NodeOutput, NodeSpec, PinType, TypedPin, ViperNode};
use serde_json::json;
use std::collections::HashMap;

pub struct StringToPathNode;

#[async_trait]
impl ViperNode for StringToPathNode {
    fn spec(&self) -> NodeSpec {
        NodeSpec {
            name: "string_to_path".into(),
            version: "0.1.0".into(),
            description: "Explicitly converts a String to a FilePath".into(),
            inputs: vec![TypedPin {
                name: "path".into(),
                pin_type: PinType::String,
                required: true,
                description: "String path to validate and convert".into(),
            }],
            outputs: vec![TypedPin {
                name: "path".into(),
                pin_type: PinType::FilePath,
                required: true,
                description: "Validated FilePath".into(),
            }],
            requires_network: false,
            requires_fs_write: false,
        }
    }

    async fn execute(&self, input: NodeInput) -> Result<NodeOutput> {
        let raw = input
            .get("path")
            .or_else(|| input.get("text"))
            .ok_or_else(|| anyhow!("string_to_path requires a 'path' input"))?
            .as_str()
            .ok_or_else(|| anyhow!("'path' input must be a string"))?;

        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(anyhow!("Path cannot be empty"));
        }

        let mut output = HashMap::new();
        output.insert("path".to_string(), json!(trimmed));
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn converts_valid_path() {
        let node = StringToPathNode;
        let mut input = HashMap::new();
        input.insert("path".to_string(), json!(" /tmp/test.txt "));
        let out = node.execute(input).await.unwrap();
        assert_eq!(out["path"], json!("/tmp/test.txt"));
    }

    #[tokio::test]
    async fn rejects_empty_path() {
        let node = StringToPathNode;
        let mut input = HashMap::new();
        input.insert("path".to_string(), json!("   "));
        let err = node.execute(input).await.unwrap_err().to_string();
        assert!(err.contains("Path cannot be empty"), "{err}");
    }
}
