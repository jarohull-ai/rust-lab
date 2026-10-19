use anyhow::Result;
use lab_node_sdk::{NodeInput, NodeOutput, NodeSpec, PinType, TypedPin, ViperNode};
use serde_json::json;
use std::collections::HashMap;
use std::fs;

pub struct FsReadNode;

pub fn fs_read(path: &str) -> Result<String> {
    println!("   Reading file: {}", path);
    let content = fs::read_to_string(path)?;
    Ok(content)
}

#[async_trait::async_trait]
impl ViperNode for FsReadNode {
    fn spec(&self) -> NodeSpec {
        NodeSpec {
            name: "fs_read".into(),
            version: "0.1.0".into(),
            description: "Reads content from a file".into(),
            inputs: vec![TypedPin {
                name: "path".into(),
                pin_type: PinType::FilePath,
                required: true,
                description: "Path to the file".into(),
            }],
            outputs: vec![TypedPin {
                name: "content".into(),
                pin_type: PinType::String,
                required: true,
                description: "File content".into(),
            }],
            requires_network: false,
            requires_fs_write: false,
        }
    }

    async fn execute(&self, input: NodeInput) -> Result<NodeOutput, anyhow::Error> {
        let path = input
            .get("path")
            .or_else(|| input.get("file_path"))
            .or_else(|| input.get("filepath"))
            .ok_or_else(|| anyhow::anyhow!("fs_read node requires a 'path' parameter"))?
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("path parameter must be a string"))?;

        let content = fs_read(path)?;

        let mut output = HashMap::new();
        output.insert("content".to_string(), json!(content));
        Ok(output)
    }
}

