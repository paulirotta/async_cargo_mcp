//! Model Control Protocol (MCP) for Cargo with asynchronous respon handling to allow the LLM to continue processing while waiting for responses.

use anyhow::Result;
use async_cargo_mcp::{
    cargo_tools::AsyncCargo,
    operation_monitor::{MonitorConfig, OperationMonitor},
};
use clap::Parser;
use rmcp::{ServiceExt, transport::stdio};
use std::sync::Arc;
use tracing::info;
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

    // Initialize the tracing subscriber with file and stdout logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    info!("Starting MCP server");

    // Create and run the operation monitor
    let monitor_config = MonitorConfig::default();
    let monitor = Arc::new(OperationMonitor::new(monitor_config));

    // Create an instance of our cargo tool service
    let service = AsyncCargo::new(monitor.clone())
        .serve(stdio())
        .await
        .inspect_err(|e| {
            tracing::error!("serving error: {:?}", e);
        })?;

    // Wait for the service to finish
    service.waiting().await?;

    // Shutdown the monitor
    monitor.shutdown().await;

    Ok(())
}
