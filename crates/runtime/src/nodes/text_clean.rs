use anyhow::{anyhow, Result};
use async_trait::async_trait;
use lab_node_sdk::{NodeInput, NodeOutput, NodeSpec, PinType, TypedPin, ViperNode};
use serde_json::Value;

/// Cleans text with four simple flags, applied in a fixed order:
/// `trim` → `remove_extra_whitespace` → `to_lower` / `to_upper`.
pub struct TextCleanNode;

fn flag_pin(name: &str, description: &str) -> TypedPin {
    TypedPin {
        name: name.into(),
        pin_type: PinType::Boolean,
        required: false,
        description: description.into(),
    }
}

fn flag(input: &NodeInput, name: &str, default: bool) -> Result<bool> {
    match input.get(name) {
        None | Some(Value::Null) => Ok(default),
        Some(Value::Bool(b)) => Ok(*b),
        Some(_) => Err(anyhow!("'{name}' must be a boolean")),
    }
}

#[async_trait]
impl ViperNode for TextCleanNode {
    fn spec(&self) -> NodeSpec {
        NodeSpec {
            name: "text_clean".into(),
            version: "0.1.0".into(),
            description: "Clean text: trim, collapse whitespace, change case".into(),
            inputs: vec![
                TypedPin {
                    name: "text".into(),
                    pin_type: PinType::String,
                    required: true,
                    description: "Text to clean".into(),
                },
                flag_pin("trim", "Trim surrounding whitespace (default: true)"),
                flag_pin(
                    "remove_extra_whitespace",
                    "Collapse runs of whitespace into one space (default: false)",
                ),
                flag_pin("to_lower", "Convert to lowercase (default: false)"),
                flag_pin("to_upper", "Convert to uppercase (default: false)"),
            ],
            outputs: vec![TypedPin {
                name: "cleaned".into(),
                pin_type: PinType::String,
                required: true,
                description: "Cleaned text".into(),
            }],
            requires_network: false,
            requires_fs_write: false,
        }
    }

    async fn execute(&self, input: NodeInput) -> Result<NodeOutput> {
        let text = input
            .get("text")
            .ok_or_else(|| anyhow!("text_clean requires a 'text' input"))?
            .as_str()
            .ok_or_else(|| anyhow!("'text' must be a string"))?;

        let trim = flag(&input, "trim", true)?;
        let collapse = flag(&input, "remove_extra_whitespace", false)?;
        let to_lower = flag(&input, "to_lower", false)?;
        let to_upper = flag(&input, "to_upper", false)?;

        if to_lower && to_upper {
            return Err(anyhow!("'to_lower' and 'to_upper' cannot both be true"));
        }

        let mut result = text.to_string();
        if trim {
            result = result.trim().to_string();
        }
        if collapse {
            // Keep a single leading/trailing space when trim is off.
            let lead = !trim && result.starts_with(char::is_whitespace);
            let trail = !trim && result.ends_with(char::is_whitespace);
            let core = result.split_whitespace().collect::<Vec<_>>().join(" ");
            result = format!(
                "{}{}{}",
                if lead { " " } else { "" },
                core,
                if trail && !core.is_empty() { " " } else { "" }
            );
        }
        if to_lower {
            result = result.to_lowercase();
        }
        if to_upper {
            result = result.to_uppercase();
        }

        let mut out = NodeOutput::new();
        out.insert("cleaned".into(), Value::String(result));
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    async fn run(pairs: &[(&str, Value)]) -> Result<String> {
        let input = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect();
        let out = TextCleanNode.execute(input).await?;
        Ok(out["cleaned"].as_str().unwrap().to_string())
    }

    #[tokio::test]
    async fn trims_by_default() {
        assert_eq!(run(&[("text", json!("  Hi  "))]).await.unwrap(), "Hi");
    }

    #[tokio::test]
    async fn applies_all_flags_in_order() {
        let cleaned = run(&[
            ("text", json!("  Hello \n\t  WORLD  ")),
            ("remove_extra_whitespace", json!(true)),
            ("to_lower", json!(true)),
        ])
        .await
        .unwrap();
        assert_eq!(cleaned, "hello world");
    }

    #[tokio::test]
    async fn rejects_conflicting_case_flags_and_bad_types() {
        assert!(run(&[
            ("text", json!("x")),
            ("to_lower", json!(true)),
            ("to_upper", json!(true))
        ])
        .await
        .is_err());
        assert!(run(&[("text", json!("x")), ("trim", json!("yes"))])
            .await
            .is_err());
    }
}
