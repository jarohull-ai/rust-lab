use anyhow::{anyhow, Result};
use async_trait::async_trait;
use lab_node_sdk::{NodeInput, NodeOutput, NodeSpec, PinType, TypedPin, ViperNode};
use serde_json::{json, Value};
use std::collections::HashMap;

pub struct JsonToStringNode;

#[async_trait]
impl ViperNode for JsonToStringNode {
    fn spec(&self) -> NodeSpec {
        NodeSpec {
            name: "json_to_string".into(),
            version: "0.1.0".into(),
            description: "Converts a JSON value to a String".into(),
            inputs: vec![TypedPin {
                name: "value".into(),
                pin_type: PinType::Json,
                required: true,
                description: "JSON value to stringify".into(),
            }],
            outputs: vec![TypedPin {
                name: "text".into(),
                pin_type: PinType::String,
                required: true,
                description: "Converted string representation".into(),
            }],
            requires_network: false,
            requires_fs_write: false,
        }
    }

    async fn execute(&self, input: NodeInput) -> Result<NodeOutput> {
        let val = input
            .get("value")
            .ok_or_else(|| anyhow!("json_to_string requires a 'value' input"))?;

        let string_repr = match val {
            Value::String(s) => s.clone(),
            _ => serde_json::to_string(val)?,
        };

        let mut output = HashMap::new();
        output.insert("text".to_string(), json!(string_repr));
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn converts_string_without_extra_quotes() {
        let node = JsonToStringNode;
        let mut input = HashMap::new();
        input.insert("value".to_string(), json!("hello world"));
        let out = node.execute(input).await.unwrap();
        assert_eq!(out["text"], json!("hello world"));
    }

    #[tokio::test]
    async fn converts_numbers_and_objects() {
        let node = JsonToStringNode;

        let mut input = HashMap::new();
        input.insert("value".to_string(), json!(42));
        let out = node.execute(input).await.unwrap();
        assert_eq!(out["text"], json!("42"));

        let mut input2 = HashMap::new();
        input2.insert("value".to_string(), json!({"a": 1}));
        let out2 = node.execute(input2).await.unwrap();
        assert_eq!(out2["text"], json!("{\"a\":1}"));
    }
}
