use anyhow::Result;
use lab_node_sdk::{NodeInput, NodeOutput, NodeSpec, PinType, TypedPin, ViperNode};

use std::collections::HashMap;

pub struct FsWriteNode;

pub fn fs_write(path: &str, content: &str) -> Result<()> {
    println!("   Writing to file: {}", path);
    if let Some(parent) = std::path::Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(path, content)?;
    Ok(())
}

#[async_trait::async_trait]
impl ViperNode for FsWriteNode {
    fn spec(&self) -> NodeSpec {
        NodeSpec {
            name: "fs_write".into(),
            version: "0.1.0".into(),
            description: "Writes content to a file".into(),
            inputs: vec![
                TypedPin {
                    name: "path".into(),
                    pin_type: PinType::FilePath,
                    required: true,
                    description: "Path to the file".into(),
                },
                TypedPin {
                    // This field name must match what the executor looks for
                    name: "content".into(),
                    pin_type: PinType::String,
                    required: true,
                    description: "Content to write".into(),
                },
            ],
            outputs: vec![],
            requires_network: false,
            requires_fs_write: true,
        }
    }

    async fn execute(&self, input: NodeInput) -> Result<NodeOutput, anyhow::Error> {
        let path = input
            .get("path")
            .or_else(|| input.get("file_path"))
            .or_else(|| input.get("filepath"))
            .ok_or_else(|| anyhow::anyhow!("fs_write node requires a 'path' parameter"))?
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("path parameter must be a string"))?;

        let content = input
            .get("content")
            .ok_or_else(|| anyhow::anyhow!("fs_write node requires a 'content' parameter"))?
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("content parameter must be a string"))?;

        fs_write(path, content)?;

        Ok(HashMap::new())
    }
}

