use anyhow::Result;
use lab_node_sdk::{NodeInput, NodeOutput, NodeSpec, PinType, TypedPin, ViperNode};
use serde_json::json;
use std::collections::HashMap;

pub struct LogOutputNode;

#[async_trait::async_trait]
impl ViperNode for LogOutputNode {
    fn spec(&self) -> NodeSpec {
        NodeSpec {
            name: "log_output".into(),
            version: "0.1.0".into(),
            description: "Prints input to console".into(),
            inputs: vec![TypedPin {
                name: "text".into(),
                pin_type: PinType::Any,
                required: true,
                description: "Value to log".into(),
            }],
            outputs: vec![],
            requires_network: false,
            requires_fs_write: false,
        }
    }

    async fn execute(&self, input: NodeInput) -> Result<NodeOutput, anyhow::Error> {
        let text = input.get("text").cloned().unwrap_or(json!("No input"));
        println!("   Log Output: {}", text);
        Ok(HashMap::new())
    }
}
