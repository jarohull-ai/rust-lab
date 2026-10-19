use anyhow::Result;
use lab_node_sdk::{NodeInput, NodeOutput, NodeSpec, PinType, TypedPin, ViperNode};
use serde_json::json;
use std::collections::HashMap;

pub struct HttpGetNode;

pub async fn http_get(url: &str) -> Result<String> {
    println!("   Fetching: {}", url);
    let client = reqwest::Client::builder()
        .user_agent("RustLab-PoC/0.0.2")
        .build()?;
    let resp = client.get(url).send().await?.text().await?;
    Ok(resp)
}

#[async_trait::async_trait]
impl ViperNode for HttpGetNode {
    fn spec(&self) -> NodeSpec {
        NodeSpec {
            name: "http_get".into(),
            version: "0.1.0".into(),
            description: "HTTP GET request".into(),
            inputs: vec![TypedPin {
                name: "url".into(),
                pin_type: PinType::String,
                required: true,
                description: "URL to fetch".into(),
            }],
            outputs: vec![TypedPin {
                name: "body".into(),
                pin_type: PinType::String,
                required: true,
                description: "Response body".into(),
            }],
            requires_network: true,
            requires_fs_write: false,
        }
    }

    async fn execute(&self, input: NodeInput) -> Result<NodeOutput, anyhow::Error> {
        let url = input
            .get("url")
            .ok_or_else(|| anyhow::anyhow!("HTTP Get node requires a 'url' parameter in input"))?
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("URL parameter must be a string"))?;

        let body = http_get(url).await?;

        let mut output = HashMap::new();
        output.insert("body".to_string(), json!(body));
        Ok(output)
    }
}

