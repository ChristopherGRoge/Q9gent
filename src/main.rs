mod agent;
mod api;
mod error;
mod session;
mod config;

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
                .unwrap_or_else(|_| "q9gent=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    let config = Arc::new(config::ServerConfig {
        claude_path: args.claude_path,
        session_dir: args.session_dir,
    });

    // Create session directory if it doesn't exist
    tokio::fs::create_dir_all(&config.session_dir).await?;

    let addr = format!("{}:{}", args.host, args.port);
    
    tracing::info!("Starting Q9gent server on {}", addr);
    
    api::serve(&addr, config).await?;

    Ok(())
}
