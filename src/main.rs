//! Model Control Protocol (MCP) for Cargo with asynchronous respon handling to allow the LLM to continue processing while waiting for responses.

use anyhow::Result;
use async_cargo_mcp::cargo_tools::AsyncCargo;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::{self, EnvFilter};

/// npx @modelcontextprotocol/inspector cargo run -p async_cargo_mcp
#[tokio::main]
async fn main() -> Result<()> {
    // Write to a log file to prove we started
    std::fs::write(
        "/tmp/async_cargo_mcp_started.log",
        format!(
            "async_cargo_mcp started at {:?}\n",
            std::time::SystemTime::now()
        ),
    )
    .ok();

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
