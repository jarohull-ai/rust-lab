use anyhow::Context;
use api::{app, AppState};
use std::net::{IpAddr, SocketAddr};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3030);

    let state = AppState::default();
    let router = app(state);

    let host: IpAddr = std::env::var("RUST_LAB_HOST")
        .unwrap_or_else(|_| "127.0.0.1".to_string())
        .parse()
        .context("RUST_LAB_HOST must be a valid IP address")?;
    if !host.is_loopback() {
        eprintln!(
            "⚠️  Rust Lab is listening outside this computer. Use only on a trusted network."
        );
    }
    let addr = SocketAddr::new(host, port);
    println!("🦀 Rust Lab API Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
