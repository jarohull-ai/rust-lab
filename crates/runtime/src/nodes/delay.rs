use anyhow::{anyhow, Result};
use async_trait::async_trait;
use lab_node_sdk::{NodeInput, NodeOutput, NodeSpec, PinType, TypedPin, ViperNode};
use std::time::Duration;

/// Upper bound so a typo like "3000m" cannot hang a lesson for days.
pub const MAX_DELAY: Duration = Duration::from_secs(600);

/// Pauses execution for a human-readable duration: `150ms`, `3s`, `2m`.
pub struct DelayNode;

pub fn parse_duration(text: &str) -> Result<Duration> {
    let text = text.trim();
    let split = text
        .find(|c: char| !c.is_ascii_digit())
        .ok_or_else(|| anyhow!("Duration '{text}' needs a unit: ms, s or m"))?;
    let (number, unit) = text.split_at(split);
    let amount: u64 = number
        .parse()
        .map_err(|_| anyhow!("Duration '{text}' must start with a whole number"))?;

    let duration = match unit {
        "ms" => Duration::from_millis(amount),
        "s" => Duration::from_secs(amount),
        "m" => Duration::from_secs(amount.saturating_mul(60)),
        other => return Err(anyhow!("Unknown duration unit '{other}' (use ms, s or m)")),
    };
    if duration > MAX_DELAY {
        return Err(anyhow!(
            "Duration '{text}' exceeds the {}s limit",
            MAX_DELAY.as_secs()
        ));
    }
    Ok(duration)
}

pub async fn delay(duration_str: &str) -> Result<()> {
    let duration = parse_duration(duration_str)?;
    println!("   Sleeping for {duration:?}");
    tokio::time::sleep(duration).await;
    Ok(())
}

#[async_trait]
impl ViperNode for DelayNode {
    fn spec(&self) -> NodeSpec {
        NodeSpec {
            name: "delay".into(),
            version: "0.1.0".into(),
            description: "Wait for a duration such as '150ms', '3s' or '2m'".into(),
            inputs: vec![TypedPin {
                name: "duration".into(),
                pin_type: PinType::String,
                required: true,
                description: "How long to wait (ms, s, m)".into(),
            }],
            outputs: vec![],
            requires_network: false,
            requires_fs_write: false,
        }
    }

    async fn execute(&self, input: NodeInput) -> Result<NodeOutput> {
        let text = input
            .get("duration")
            .ok_or_else(|| anyhow!("delay requires a 'duration' input"))?
            .as_str()
            .ok_or_else(|| anyhow!("'duration' must be a string like '3s'"))?;
        delay(text).await?;
        Ok(NodeOutput::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_units() {
        assert_eq!(parse_duration("150ms").unwrap(), Duration::from_millis(150));
        assert_eq!(parse_duration("3s").unwrap(), Duration::from_secs(3));
        assert_eq!(parse_duration(" 2m ").unwrap(), Duration::from_secs(120));
    }

    #[test]
    fn rejects_bad_input() {
        for bad in ["", "3", "s", "3h", "-1s", "1.5s", "11m"] {
            assert!(parse_duration(bad).is_err(), "{bad} should fail");
        }
    }
}
