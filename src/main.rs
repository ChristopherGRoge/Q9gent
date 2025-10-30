mod agent;
mod api;
mod config;
mod error;
mod session;

use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Server bind address
    #[arg(short, long, default_value = "127.0.0.1")]
    host: String,

    /// Server port
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// Session storage directory
    #[arg(short, long, default_value = "./sessions")]
    session_dir: String,

    /// Path to claude CLI executable
    #[arg(short, long, default_value = "claude")]
    claude_path: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "q9gent=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    tracing::info!("🎯 Q9gent v{} starting...", env!("CARGO_PKG_VERSION"));
    tracing::info!("📂 Session directory: {}", args.session_dir);
    tracing::info!("🔧 Claude CLI path: {}", args.claude_path);

    let config = Arc::new(config::ServerConfig {
        claude_path: args.claude_path,
        session_dir: args.session_dir,
    });

    // Create session directory if it doesn't exist
    tokio::fs::create_dir_all(&config.session_dir).await?;
    tracing::debug!("✓ Session directory ready");

    let addr = format!("{}:{}", args.host, args.port);

    api::serve(&addr, config).await?;

    Ok(())
}
