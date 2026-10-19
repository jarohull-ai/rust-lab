use anyhow::{anyhow, Result};
use async_trait::async_trait;
use lab_node_sdk::{NodeInput, NodeOutput, NodeSpec, PinType, TypedPin, ViperNode};
use serde_json::{json, Value};

/// Reserved output key: id of the node the executor should run next.
pub const NEXT_KEY: &str = "__next__";
/// Reserved output key: id of the branch the executor must not run.
pub const SKIP_KEY: &str = "__skip__";

/// Routes control flow: picks `true_branch` or `false_branch` from a boolean.
/// It produces no data — only the reserved `__next__` / `__skip__` keys.
pub struct ConditionNode;

fn branch<'a>(input: &'a NodeInput, name: &str) -> Result<&'a str> {
    input
        .get(name)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("condition requires a non-empty '{name}' node id"))
}

#[async_trait]
impl ViperNode for ConditionNode {
    fn spec(&self) -> NodeSpec {
        NodeSpec {
            name: "condition".into(),
            version: "0.1.0".into(),
            description: "Choose the next node based on a boolean".into(),
            inputs: vec![
                TypedPin {
                    name: "value".into(),
                    pin_type: PinType::Boolean,
                    required: true,
                    description: "Condition to evaluate".into(),
                },
                TypedPin {
                    name: "true_branch".into(),
                    pin_type: PinType::String,
                    required: true,
                    description: "Node id to run when value is true".into(),
                },
                TypedPin {
                    name: "false_branch".into(),
                    pin_type: PinType::String,
                    required: true,
                    description: "Node id to run when value is false".into(),
                },
            ],
            outputs: vec![],
            requires_network: false,
            requires_fs_write: false,
        }
    }

    async fn execute(&self, input: NodeInput) -> Result<NodeOutput> {
        let value = input
            .get("value")
            .ok_or_else(|| anyhow!("condition requires a 'value' input"))?
            .as_bool()
            .ok_or_else(|| anyhow!("'value' must be a boolean"))?;
        let on_true = branch(&input, "true_branch")?;
        let on_false = branch(&input, "false_branch")?;

        let (next, skip) = if value {
            (on_true, on_false)
        } else {
            (on_false, on_true)
        };
        println!("   Condition = {value} → {next}");

        let mut out = NodeOutput::new();
        out.insert(NEXT_KEY.into(), json!(next));
        out.insert(SKIP_KEY.into(), json!(skip));
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn run(value: Value) -> Result<NodeOutput> {
        let input = [
            ("value", value),
            ("true_branch", json!("yes")),
            ("false_branch", json!("no")),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();
        ConditionNode.execute(input).await
    }

    #[tokio::test]
    async fn selects_branch() {
        let t = run(json!(true)).await.unwrap();
        assert_eq!(t[NEXT_KEY], json!("yes"));
        assert_eq!(t[SKIP_KEY], json!("no"));
        let f = run(json!(false)).await.unwrap();
        assert_eq!(f[NEXT_KEY], json!("no"));
    }

    #[tokio::test]
    async fn rejects_non_boolean() {
        assert!(run(json!("true")).await.is_err());
    }
}
