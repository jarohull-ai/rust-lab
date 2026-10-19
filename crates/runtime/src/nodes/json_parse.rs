use anyhow::{anyhow, Result};
use async_trait::async_trait;
use lab_node_sdk::{NodeInput, NodeOutput, NodeSpec, PinType, TypedPin, ViperNode};
use serde_json::Value;

/// Parses a JSON string into a structured `Value`, optionally drilling into it
/// with a dot-notation path (`data.items.0.name`). Numeric segments index arrays.
pub struct JsonParseNode;

/// Walks `value` along a dot-notation `path`.
pub fn select_path(value: &Value, path: &str) -> Result<Value> {
    let mut current = value;
    for segment in path.split('.').filter(|s| !s.is_empty()) {
        current = match current {
            Value::Object(map) => map.get(segment),
            Value::Array(items) => segment.parse::<usize>().ok().and_then(|i| items.get(i)),
            _ => None,
        }
        .ok_or_else(|| anyhow!("Path segment '{segment}' not found in '{path}'"))?;
    }
    Ok(current.clone())
}

/// Parses a JSON string, optionally selecting a nested dot-notation path.
pub fn json_parse(raw: &str, path: Option<&str>) -> Result<Value> {
    let parsed: Value = serde_json::from_str(raw).map_err(|e| anyhow!("Invalid JSON: {e}"))?;
    match path.filter(|p| !p.is_empty()) {
        Some(p) => select_path(&parsed, p),
        None => Ok(parsed),
    }
}

#[async_trait]
impl ViperNode for JsonParseNode {
    fn spec(&self) -> NodeSpec {
        NodeSpec {
            name: "json_parse".into(),
            version: "0.1.0".into(),
            description: "Parse a JSON string into a structured value".into(),
            inputs: vec![
                TypedPin {
                    name: "json_str".into(),
                    pin_type: PinType::String,
                    required: true,
                    description: "JSON text to parse".into(),
                },
                TypedPin {
                    name: "path".into(),
                    pin_type: PinType::String,
                    required: false,
                    description: "Optional dot-notation path (e.g. 'data.items.0')".into(),
                },
            ],
            outputs: vec![TypedPin {
                name: "value".into(),
                pin_type: PinType::Json,
                required: true,
                description: "Parsed JSON value".into(),
            }],
            requires_network: false,
            requires_fs_write: false,
        }
    }

    async fn execute(&self, input: NodeInput) -> Result<NodeOutput> {
        let raw = input
            .get("json_str")
            .ok_or_else(|| anyhow!("json_parse requires a 'json_str' input"))?
            .as_str()
            .ok_or_else(|| anyhow!("'json_str' must be a string"))?;

        let path = input.get("path").and_then(Value::as_str);
        let value = json_parse(raw, path)?;

        let mut out = NodeOutput::new();
        out.insert("value".into(), value);
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn input(pairs: &[(&str, Value)]) -> NodeInput {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()
    }

    #[tokio::test]
    async fn parses_whole_document() {
        let out = JsonParseNode
            .execute(input(&[("json_str", json!(r#"{"a":1}"#))]))
            .await
            .unwrap();
        assert_eq!(out["value"], json!({"a": 1}));
    }

    #[tokio::test]
    async fn follows_path_through_objects_and_arrays() {
        let doc = r#"{"data":{"items":[{"name":"x"},{"name":"y"}]}}"#;
        let out = JsonParseNode
            .execute(input(&[
                ("json_str", json!(doc)),
                ("path", json!("data.items.1.name")),
            ]))
            .await
            .unwrap();
        assert_eq!(out["value"], json!("y"));
    }

    #[tokio::test]
    async fn rejects_invalid_json_and_missing_path() {
        assert!(JsonParseNode
            .execute(input(&[("json_str", json!("{oops"))]))
            .await
            .is_err());
        assert!(JsonParseNode
            .execute(input(&[
                ("json_str", json!("{}")),
                ("path", json!("missing"))
            ]))
            .await
            .is_err());
    }
}
