//! Model Control Protocol (MCP) for Cargo with asynchronous respon handling to allow the LLM to continue processing while waiting for responses.

use anyhow::Result;
use async_cargo_mcp::{
    cargo_tools::AsyncCargo,
    operation_monitor::{MonitorConfig, OperationMonitor},
};
use clap::Parser;
use rmcp::{ServiceExt, transport::stdio};
use std::sync::Arc;
use std::time::Duration;
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
                  Features pre-warmed shell pools for 10x faster cargo command execution.\n\n\
                  For more information, visit: https://github.com/paulirotta/async_cargo_mcp"
)]
struct Args {
    /// Override default timeout in seconds (default: 300)
    #[arg(
        long,
        value_name = "SECONDS",
        help = "Set default timeout in seconds for cargo operations (default: 300)"
    )]
    timeout: Option<u64>,

    /// Number of shells per working directory (default: 2)
    #[arg(
        long,
        value_name = "COUNT",
        help = "Number of pre-warmed shells per working directory for faster command execution"
    )]
    shell_pool_size: Option<usize>,

    /// Maximum total number of shells across all pools (default: 20)
    #[arg(
        long,
        value_name = "COUNT",
        help = "Maximum total number of shells across all working directories"
    )]
    max_shells: Option<usize>,

    /// Disable shell pools and use direct command spawning
    #[arg(
        long,
        help = "Disable shell pools and use direct tokio::process::Command spawning"
    )]
    disable_shell_pools: bool,

    /// Force synchronous operation mode (disables async callbacks for all operations)
    #[arg(
        long,
        help = "Force synchronous execution of all operations, disabling async callbacks and notifications"
    )]
    synchronous: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize the tracing subscriber with improved formatting
    /* verbose in terminal
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .with_target(false) // Hide target module names
        .compact() // Use compact format for cleaner output
        .init();
    */
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info")) // force info+ (ignores RUST_LOG)
        .with_writer(std::io::stderr)
        .with_ansi(true)
        .with_target(false)
        //.compact()
        .init();

    info!("Starting MCP server");

    // Create and run the operation monitor with custom timeout if provided
    let monitor_config = match args.timeout {
        Some(timeout_secs) => {
            info!("Using custom timeout: {} seconds", timeout_secs);
            MonitorConfig::with_timeout(Duration::from_secs(timeout_secs))
        }
        None => {
            info!("Using default timeout: 300 seconds");
            MonitorConfig::default()
        }
    };
    let monitor = Arc::new(OperationMonitor::new(monitor_config));

    // Create shell pool manager with CLI configuration
    use async_cargo_mcp::shell_pool::{ShellPoolConfig, ShellPoolManager};
    let mut shell_pool_config = ShellPoolConfig::default();

    // Allow disabling shell pools via environment variable for test isolation / debugging
    if std::env::var("ASYNC_CARGO_MCP_DISABLE_SHELL_POOL").is_ok() {
        shell_pool_config.enabled = false;
        info!("Shell pools disabled via ASYNC_CARGO_MCP_DISABLE_SHELL_POOL env var");
    }

    // Apply CLI overrides
    if let Some(pool_size) = args.shell_pool_size {
        info!(
            "Using custom shell pool size: {} shells per directory",
            pool_size
        );
        shell_pool_config.shells_per_directory = pool_size;
    }

    if let Some(max_shells) = args.max_shells {
        info!("Using custom max shells: {} total shells", max_shells);
        shell_pool_config.max_total_shells = max_shells;
    }

    if args.disable_shell_pools {
        shell_pool_config.enabled = false;
        info!("Shell pools disabled via CLI flag - using direct command spawning");
    }

    if shell_pool_config.enabled {
        info!(
            "Shell pools enabled - {} shells per directory, {} max total",
            shell_pool_config.shells_per_directory, shell_pool_config.max_total_shells
        );
    } else {
        info!("Shell pools disabled");
    }

    let synchronous_mode = args.synchronous;
    if synchronous_mode {
        info!("Synchronous mode enabled - async callbacks disabled for all operations");
    } else {
        info!("Async mode enabled - operations can use async callbacks and notifications");
    }

    let shell_pool_manager = Arc::new(ShellPoolManager::new(shell_pool_config));

    // Start background health monitoring and cleanup tasks
    shell_pool_manager.clone().start_background_tasks();

    // Create an instance of our cargo tool service
    let service =
        AsyncCargo::new_with_config(monitor.clone(), shell_pool_manager, synchronous_mode)
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
