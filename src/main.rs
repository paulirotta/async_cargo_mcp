//! Model Control Protocol (MCP) for Cargo with asynchronous respon handling to allow the LLM to continue processing while waiting for responses.

use anyhow::Result;
use async_cargo_mcp::cargo_tools::AsyncCargo;
use clap::Parser;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::{self, EnvFilter};

/// Model Context Protocol server for Cargo operations with async support
#[derive(Parser)]
#[command(
    name = "async_cargo_mcp",
    version = env!("CARGO_PKG_VERSION"),
    about = "MCP server providing async Cargo operations for AI assistants",
    long_about = "A Model Context Protocol (MCP) server that provides asynchronous Cargo operations. \
                  This allows AI assistants to manage Rust projects with build, test, and dependency \
                  management capabilities while maintaining responsive interaction.\n\n\
                  For more information, visit: https://github.com/paulirotta/async_cargo_mcp"
)]
struct Args {
    // No direct arguments currently, but the struct is ready for future expansion
}

/// npx @modelcontextprotocol/inspector cargo run -p async_cargo_mcp
#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let _args = Args::parse();

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
