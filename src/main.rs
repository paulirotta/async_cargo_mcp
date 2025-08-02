//! Model Control Protocol (MCP) for Cargo with asynchronous respon handling to allow the LLM to continue processing while waiting for responses.

//mod cargo_command;
mod cargo_tools;

use crate::cargo_tools::AsyncCargo;
use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::{self, EnvFilter};

/// npx @modelcontextprotocol/inspector cargo run -p async_cargo_mcp
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the tracing subscriber with file and stdout logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting MCP server");

    // Create an instance of our counter router
    let service = AsyncCargo::new().serve(stdio()).await.inspect_err(|e| {
        tracing::error!("serving error: {:?}", e);
    })?;

    service.waiting().await?;
    Ok(())
}
