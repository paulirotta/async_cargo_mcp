//! # Async Cargo MCP
//!
//! A Model Context Protocol (MCP) server that provides asynchronous access to Cargo commands.
//! This crate allows clients (such as language models and development tools) to execute Rust
//! build system commands like `build`, `test`, `run`, `doc`, and dependency management operations
//! through a standardized protocol interface.
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use async_cargo_mcp::test_doc_functionality;
//! use anyhow::Result;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Test documentation generation
//!     let results = test_doc_functionality().await?;
//!     println!("Test results: {results}");
//!     Ok(())
//! }
//! ```

pub mod callback_system;
pub mod cargo_tools;
pub mod command_registry;
pub mod mcp_callback;
pub mod operation_monitor;
pub mod test_cargo_tools;

use anyhow::Result;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use tokio::process::Command;

/// Test documentation generation functionality
///
/// Tests the `doc` command to verify MCP server integration works correctly.
pub async fn test_doc_functionality() -> Result<String> {
    let client = ()
        .serve(TokioChildProcess::new(Command::new("cargo").configure(
            |cmd| {
                cmd.arg("run").arg("--bin").arg("async_cargo_mcp");
            },
        ))?)
        .await?;

    // Test doc command
    let doc_result = client
        .call_tool(CallToolRequestParam {
            name: "doc".into(),
            arguments: Some(object!({
                "working_directory": std::env::current_dir()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            })),
        })
        .await
        .map_err(|e| anyhow::anyhow!("Doc command failed: {}", e))?;

    // Store the result before canceling the client
    let result = format!("Documentation generation test results:\n- Doc result: {doc_result:?}");

    // Cancel the client - ignore errors since transport might already be closed
    let _ = client.cancel().await;

    Ok(result)
}
