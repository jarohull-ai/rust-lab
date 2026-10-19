use anyhow::{Context, Result};
use runtime::{codegen, nodes, Executor, NodeRegistry, Validator};
use std::env;
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage:");
        println!("  Run graph:    cargo run -p runtime -- <graph.json>");
        println!("  Export Cargo: cargo run -p runtime -- --export <output_dir> <graph.json>");
        std::process::exit(1);
    }

    let is_export = args[1] == "--export";
    let (graph_path, export_dir) = if is_export {
        if args.len() < 4 {
            println!("Usage: cargo run -p runtime -- --export <output_dir> <graph.json>");
            std::process::exit(1);
        }
        (&args[3], Some(Path::new(&args[2])))
    } else {
        (&args[1], None)
    };

    let content = fs::read_to_string(graph_path)
        .with_context(|| format!("Could not read file: {}", graph_path))?;
    let graph: lab_node_sdk::GraphSpec =
        serde_json::from_str(&content).context("Invalid graph JSON format")?;

    // Setup Registry & Validate
    let mut registry = NodeRegistry::new();
    nodes::register_all(&mut registry);

    let validator = Validator::new(&registry);
    if let Err(e) = validator.validate(&graph) {
        eprintln!("❌ Graph Validation Failed: {}", e);
        std::process::exit(1);
    }
    println!("✅ Graph Validated Successfully.");

    // Handle Construction Mode Export
    if let Some(out_dir) = export_dir {
        println!("🔨 Construction Mode: Exporting Cargo project...");
        let project_name = out_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("rust_lab_app");
        let safe_name = codegen::sanitize_ident(project_name);
        codegen::export_project(&graph, out_dir, &safe_name)?;
        println!("🎉 Project successfully generated in: {}", out_dir.display());
        println!("👉 To build and run:");
        println!("   cd {} && cargo run", out_dir.display());
        return Ok(());
    }

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
