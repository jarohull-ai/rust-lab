use anyhow::{anyhow, Result};
use async_trait::async_trait;
use lab_node_sdk::{NodeInput, NodeOutput, NodeSpec, PinType, TypedPin, ViperNode};
use serde_json::{json, Value};
use std::collections::HashMap;

pub struct JsonToBoolNode;

#[async_trait]
impl ViperNode for JsonToBoolNode {
    fn spec(&self) -> NodeSpec {
        NodeSpec {
            name: "json_to_bool".into(),
            version: "0.1.0".into(),
            description: "Converts a JSON value to a Boolean".into(),
            inputs: vec![TypedPin {
                name: "value".into(),
                pin_type: PinType::Json,
                required: true,
                description: "JSON value to parse as boolean".into(),
            }],
            outputs: vec![TypedPin {
                name: "value".into(),
                pin_type: PinType::Boolean,
                required: true,
                description: "Parsed boolean value".into(),
            }],
            requires_network: false,
            requires_fs_write: false,
        }
    }

    async fn execute(&self, input: NodeInput) -> Result<NodeOutput> {
        let val = input
            .get("value")
            .ok_or_else(|| anyhow!("json_to_bool requires a 'value' input"))?;

        let boolean_value = match val {
            Value::Bool(b) => *b,
            Value::String(s) => match s.trim().to_lowercase().as_str() {
                "true" | "1" | "yes" => true,
                "false" | "0" | "no" => false,
                other => return Err(anyhow!("Cannot convert string '{other}' to boolean")),
            },
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    match i {
                        1 => true,
                        0 => false,
                        other => return Err(anyhow!("Cannot convert number {other} to boolean (expected 0 or 1)")),
                    }
                } else {
                    return Err(anyhow!("Cannot convert floating number to boolean"));
                }
            }
            other => return Err(anyhow!("Cannot convert JSON value {other} to boolean")),
        };

        let mut output = HashMap::new();
        output.insert("value".to_string(), json!(boolean_value));
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn converts_bool_values() {
        let node = JsonToBoolNode;

        let mut input = HashMap::new();
        input.insert("value".to_string(), json!(true));
        let out = node.execute(input).await.unwrap();
        assert_eq!(out["value"], json!(true));

        let mut input2 = HashMap::new();
        input2.insert("value".to_string(), json!(false));
        let out2 = node.execute(input2).await.unwrap();
        assert_eq!(out2["value"], json!(false));
    }

    #[tokio::test]
    async fn converts_string_and_number_representations() {
        let node = JsonToBoolNode;

        let mut input_str = HashMap::new();
        input_str.insert("value".to_string(), json!("true"));
        let out = node.execute(input_str).await.unwrap();
        assert_eq!(out["value"], json!(true));

        let mut input_num = HashMap::new();
        input_num.insert("value".to_string(), json!(1));
        let out_num = node.execute(input_num).await.unwrap();
        assert_eq!(out_num["value"], json!(true));

        let mut input_zero = HashMap::new();
        input_zero.insert("value".to_string(), json!(0));
        let out_zero = node.execute(input_zero).await.unwrap();
        assert_eq!(out_zero["value"], json!(false));
    }

    #[tokio::test]
    async fn rejects_invalid_types() {
        let node = JsonToBoolNode;

        let mut input = HashMap::new();
        input.insert("value".to_string(), json!("not_a_bool"));
        assert!(node.execute(input).await.is_err());

        let mut input_obj = HashMap::new();
        input_obj.insert("value".to_string(), json!({"foo": "bar"}));
        assert!(node.execute(input_obj).await.is_err());
    }
}
