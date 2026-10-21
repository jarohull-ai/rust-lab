use anyhow::Context;
use api::{app, AppState};
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
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

    let api_token = match std::env::var("RUST_LAB_API_TOKEN") {
        Ok(token) if token.len() >= 32 => token,
        Ok(_) => anyhow::bail!("RUST_LAB_API_TOKEN must contain at least 32 characters"),
        Err(_) => api::generate_api_token().context("could not generate the local API token")?,
    };
    let state = AppState::new(PathBuf::from("target/rust-lab-workspaces"), api_token);
    println!("🔐 Rust Lab local API token: {}", state.api_token());
    let router = app(state);

    let host: IpAddr = std::env::var("RUST_LAB_HOST")
        .unwrap_or_else(|_| "127.0.0.1".to_string())
        .parse()
        .context("RUST_LAB_HOST must be a valid IP address")?;
    let host = require_loopback(host)?;
    let addr = SocketAddr::new(host, port);
    println!("🦀 Rust Lab API Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}

fn require_loopback(host: IpAddr) -> anyhow::Result<IpAddr> {
    if !host.is_loopback() {
        anyhow::bail!(
            "RUST_LAB_HOST must be a loopback address; the local session API cannot be exposed to a network"
        );
    }
    Ok(host)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_api_rejects_network_bindings() {
        assert!(require_loopback("127.0.0.1".parse().unwrap()).is_ok());
        assert!(require_loopback("::1".parse().unwrap()).is_ok());
        assert!(require_loopback("0.0.0.0".parse().unwrap()).is_err());
        assert!(require_loopback("192.168.1.20".parse().unwrap()).is_err());
    }
}
