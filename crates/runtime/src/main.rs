mod executor;
mod nodes;
mod plan;
mod registry;
mod validator;

use anyhow::{Context, Result};
use executor::Executor;
use registry::NodeRegistry;
use std::env;
use std::fs;
use validator::Validator;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Load graph from args
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run -p runtime -- <graph.json>");
        std::process::exit(1);
    }
    let path = &args[1];
    let content =
        fs::read_to_string(path).with_context(|| format!("Could not read file: {}", path))?;
    let graph: lab_node_sdk::GraphSpec =
        serde_json::from_str(&content).context("Invalid graph JSON format")?;

    // 2. Setup Registry
    let mut registry = NodeRegistry::new();
    nodes::register_all(&mut registry);

    // 3. Validation
    let validator = Validator::new(&registry);
    if let Err(e) = validator.validate(&graph) {
        eprintln!("❌ Graph Validation Failed: {}", e);
        std::process::exit(1);
    }
    println!("✅ Graph Validated Successfully.");

    // 4. Execution
    let executor = Executor::new(registry);
    println!("Starting Rust Lab Runtime v0.0.2...");
    let report = executor.execute(graph).await?;
    println!(
        "Execution finished successfully! ({} executed, {} skipped)",
        report.executed.len(),
        report.skipped.len()
    );

    Ok(())
}
